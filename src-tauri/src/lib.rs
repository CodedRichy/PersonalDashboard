mod deadlines;
mod git;
mod model;
mod rank;
mod vault;

use chrono::Local;
use model::DayView;
use tauri::Manager;

#[tauri::command]
fn get_day(app: tauri::AppHandle) -> DayView {
    let now = Local::now();
    let today = now.date_naive();
    let mut warnings = Vec::new();

    let home = app.path().home_dir().unwrap_or_default();
    let repos = git::scan(&home.join("Documents").join("GitHub"), now.timestamp());
    let notes = vault::project_note_names(
        &home.join("Documents").join("Vault").join("_brain").join("projects"),
    );

    let deadlines = match app.path().app_data_dir() {
        Ok(dir) => {
            let (d, w) = deadlines::load(&dir.join("deadlines.yaml"));
            warnings.extend(w);
            d
        }
        Err(e) => {
            warnings.push(format!("app data folder unavailable: {e}"));
            vec![]
        }
    };

    rank::build_day(today, &deadlines, &repos, &notes, warnings)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_day])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
