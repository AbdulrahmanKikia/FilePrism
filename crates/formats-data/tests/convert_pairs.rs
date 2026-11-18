//! Integration tests for M3 data conversion pairs.

use std::path::{Path, PathBuf};

use fileprism_core::{CancellationToken, Converter, Job, JobOptions, Pipeline};
use fileprism_formats_data::DataConverter;
use serde_json::Value;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fileprism-m3-data-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn job(id: &str, input: &Path, output: &Path, from: &str, to: &str) -> Job {
    Job {
        id: id.into(),
        input_path: input.to_string_lossy().into(),
        output_path: output.to_string_lossy().into(),
        input_format: from.into(),
        output_format: to.into(),
        options: JobOptions {
            quality: Some(50),
            width: Some(100),
            height: Some(80),
            strip_metadata: Some(true),
            ..Default::default()
        },
    }
}

fn run(job: &Job) -> fileprism_core::Output {
    let mut pipeline = Pipeline::new();
    pipeline.register(DataConverter::new());
    pipeline
        .run_silent(job, &CancellationToken::new())
        .unwrap_or_else(|e| {
            panic!(
                "convert {}→{} failed: {e}",
                job.input_format, job.output_format
            )
        })
}

fn sample_json_table() -> &'static str {
    r#"[
  {"name": "Ada", "score": 95, "active": true},
  {"name": "Grace", "score": 88, "active": false}
]"#
}

#[test]
fn json_to_csv_and_back() {
    let dir = temp_dir("json-csv");
    let json_in = dir.join("in.json");
    let csv = dir.join("out.csv");
    let json_out = dir.join("roundtrip.json");
    std::fs::write(&json_in, sample_json_table()).unwrap();

    let out = run(&job("json-csv", &json_in, &csv, "json", "csv"));
    assert!(out.bytes_written > 0);
    let csv_text = std::fs::read_to_string(&csv).unwrap();
    assert!(csv_text.contains("name"));
    assert!(csv_text.contains("Ada"));
    assert!(csv_text.contains("Grace"));

    let out = run(&job("csv-json", &csv, &json_out, "csv", "json"));
    assert!(out.bytes_written > 0);
    let value: Value = serde_json::from_str(&std::fs::read_to_string(&json_out).unwrap()).unwrap();
    let arr = value.as_array().expect("array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["name"], "Ada");
    assert_eq!(arr[1]["score"], 88);
}

#[test]
fn json_to_yaml_and_back() {
    let dir = temp_dir("json-yaml");
    let json_in = dir.join("in.json");
    let yaml = dir.join("out.yaml");
    let json_out = dir.join("roundtrip.json");
    std::fs::write(
        &json_in,
        r#"{"title": "FilePrism", "version": 3, "tags": ["data", "m3"]}"#,
    )
    .unwrap();

    let out = run(&job("json-yaml", &json_in, &yaml, "json", "yaml"));
    assert!(out.bytes_written > 0);
    let yaml_text = std::fs::read_to_string(&yaml).unwrap();
    assert!(yaml_text.contains("FilePrism"));

    let out = run(&job("yaml-json", &yaml, &json_out, "yml", "json"));
    assert!(out.bytes_written > 0);
    let value: Value = serde_json::from_str(&std::fs::read_to_string(&json_out).unwrap()).unwrap();
    assert_eq!(value["title"], "FilePrism");
    assert_eq!(value["version"], 3);
    assert_eq!(value["tags"][0], "data");
}

#[test]
fn csv_to_xlsx_and_back() {
    let dir = temp_dir("csv-xlsx");
    let csv_in = dir.join("in.csv");
    let xlsx = dir.join("out.xlsx");
    let csv_out = dir.join("roundtrip.csv");
    std::fs::write(
        &csv_in,
        "name,qty,price\nwidget,3,1.5\ngadget,10,2.25\n",
    )
    .unwrap();

    let out = run(&job("csv-xlsx", &csv_in, &xlsx, "csv", "xlsx"));
    assert!(out.bytes_written > 0);
    assert!(xlsx.is_file());

    let out = run(&job("xlsx-csv", &xlsx, &csv_out, "xlsx", "csv"));
    assert!(out.bytes_written > 0);
    let text = std::fs::read_to_string(&csv_out).unwrap();
    assert!(text.contains("name"));
    assert!(text.contains("widget"));
    assert!(text.contains("gadget"));
    assert!(text.contains("2.25") || text.contains("2.250"));
}

#[test]
fn supports_expected_pairs() {
    let c = DataConverter::new();
    assert!(c.supports("json", "csv"));
    assert!(c.supports("csv", "json"));
    assert!(c.supports("json", "yaml"));
    assert!(c.supports("yml", "json"));
    assert!(c.supports("YAML", "JSON"));
    assert!(c.supports("csv", "xlsx"));
    assert!(c.supports("xlsx", "csv"));
    assert!(!c.supports("json", "xlsx"));
    assert!(!c.supports("yaml", "csv"));
    assert!(!c.supports("xml", "json"));
}

#[test]
fn rejects_non_object_json_array_for_csv() {
    let dir = temp_dir("bad-json");
    let json_in = dir.join("bad.json");
    let csv = dir.join("out.csv");
    std::fs::write(&json_in, r#"[1, 2, 3]"#).unwrap();

    let mut pipeline = Pipeline::new();
    pipeline.register(DataConverter::new());
    let err = pipeline
        .run_silent(
            &job("bad", &json_in, &csv, "json", "csv"),
            &CancellationToken::new(),
        )
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("object") || msg.contains("Validation"),
        "unexpected error: {msg}"
    );
}
