use fileprism_core;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, core_name])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
