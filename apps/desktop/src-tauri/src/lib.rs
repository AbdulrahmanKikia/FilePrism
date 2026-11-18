//! FilePrism desktop shell — Tauri commands wired to core / history / formats (M5).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use fileprism_core::{CancellationToken, Job, JobOptions, Output, Pipeline};
use fileprism_formats_archive::ArchiveConverter;
use fileprism_formats_data::DataConverter;
use fileprism_formats_doc::DocConverter;
use fileprism_formats_image::ImageConverter;
use fileprism_history::{HistoryStore, JobRecord, JobStatus, Setting};
use serde::Deserialize;
use tauri::Manager;

struct AppState {
    history: Mutex<HistoryStore>,
}

fn build_pipeline() -> Pipeline {
    let mut pipeline = Pipeline::new();
    pipeline.register(ImageConverter);
    pipeline.register(DocConverter);
    pipeline.register(DataConverter);
    pipeline.register(ArchiveConverter);
    pipeline
}

fn next_job_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("job-{ms}-{n}")
}

fn run_and_record(state: &AppState, job: Job) -> Result<Output, String> {
    let history = state.history.lock().map_err(|e| e.to_string())?;
    let record = history.history_enabled().unwrap_or(true);
    if record {
        history
            .record_job_start(&job, JobStatus::Running)
            .map_err(|e| e.to_string())?;
    }
    drop(history);

    let pipeline = build_pipeline();
    let cancel = CancellationToken::new();
    let result = pipeline.run_silent(&job, &cancel);

    let history = state.history.lock().map_err(|e| e.to_string())?;
    match &result {
        Ok(output) => {
            if record {
                let _ = history.record_job_complete(output);
            }
        }
        Err(err) => {
            if record {
                let status = if matches!(err, fileprism_core::Error::Cancelled) {
                    JobStatus::Cancelled
                } else {
                    JobStatus::Failed
                };
                let _ = history.record_job_finish(&job.id, status, Some(&err.to_string()));
            }
        }
    }

    result.map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunJobArgs {
    input_path: String,
    output_path: String,
    input_format: String,
    output_format: String,
    #[serde(default)]
    options: JobOptions,
    /// Optional client-supplied id; generated when omitted.
    #[serde(default)]
    id: Option<String>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!(
        "Hello, {name}! FilePrism core `{}` is linked.",
        fileprism_core::crate_name()
    )
}

#[tauri::command]
fn core_name() -> String {
    fileprism_core::crate_name().to_string()
}

/// Run a conversion / archive job through the registered pipeline and record history.
#[tauri::command]
fn run_job(state: tauri::State<'_, AppState>, args: RunJobArgs) -> Result<Output, String> {
    let job = Job {
        id: args.id.unwrap_or_else(next_job_id),
        input_path: args.input_path,
        output_path: args.output_path,
        input_format: args.input_format,
        output_format: args.output_format,
        options: args.options,
    };
    run_and_record(&state, job)
}

/// Search or list recent history jobs.
#[tauri::command]
fn search_history(
    state: tauri::State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<JobRecord>, String> {
    let history = state.history.lock().map_err(|e| e.to_string())?;
    history
        .search_jobs(&query, limit.unwrap_or(100))
        .map_err(|e| e.to_string())
}

/// Fetch a single history job by id.
#[tauri::command]
fn get_history_job(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Option<JobRecord>, String> {
    let history = state.history.lock().map_err(|e| e.to_string())?;
    history.get_job(&id).map_err(|e| e.to_string())
}

/// Re-run a past job with a new id (same paths / formats / default options).
#[tauri::command]
fn rerun_job(state: tauri::State<'_, AppState>, id: String) -> Result<Output, String> {
    let record = {
        let history = state.history.lock().map_err(|e| e.to_string())?;
        history
            .get_job(&id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("job not found: {id}"))?
    };

    let job = Job {
        id: next_job_id(),
        input_path: record.input_path,
        output_path: record.output_path,
        input_format: record.input_format,
        output_format: record.output_format,
        options: JobOptions::default(),
    };
    run_and_record(&state, job)
}

#[tauri::command]
fn list_settings(state: tauri::State<'_, AppState>) -> Result<Vec<Setting>, String> {
    let history = state.history.lock().map_err(|e| e.to_string())?;
    history.list_settings().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_setting(state: tauri::State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    let history = state.history.lock().map_err(|e| e.to_string())?;
    history.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_setting(
    state: tauri::State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let history = state.history.lock().map_err(|e| e.to_string())?;
    history.set_setting(&key, &value).map_err(|e| e.to_string())
}

fn open_history_store(app: &tauri::App) -> Result<HistoryStore, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create app data dir: {e}"))?;
    let db = dir.join("history.db");
    HistoryStore::open(&db).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let store = open_history_store(app).unwrap_or_else(|err| {
                eprintln!("FilePrism: falling back to in-memory history ({err})");
                HistoryStore::open_in_memory().expect("in-memory history")
            });
            app.manage(AppState {
                history: Mutex::new(store),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            core_name,
            run_job,
            search_history,
            get_history_job,
            rerun_job,
            list_settings,
            get_setting,
            set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
