mod deadlines;
mod git;
mod model;
mod rank;
mod vault;

use chrono::Local;
use model::DayView;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

fn compute_day(home: std::path::PathBuf, data_dir: Option<std::path::PathBuf>) -> DayView {
    let now = Local::now();
    let today = now.date_naive();
    let mut warnings = Vec::new();

    let repos = git::scan(&home.join("Documents").join("GitHub"), now.timestamp());
    let notes = vault::project_note_names(
        &home.join("Documents").join("Vault").join("_brain").join("projects"),
    );

    let deadlines = match data_dir {
        Some(dir) => {
            let (d, w) = deadlines::load(&dir.join("deadlines.yaml"));
            warnings.extend(w);
            d
        }
        None => {
            warnings.push("app data folder unavailable".to_string());
            vec![]
        }
    };

    rank::build_day(today, &deadlines, &repos, &notes, warnings)
}

// async + spawn_blocking keeps the git scan off the main thread, so the window and tray stay alive.
#[tauri::command]
async fn get_day(app: tauri::AppHandle) -> Result<DayView, String> {
    let home = app.path().home_dir().unwrap_or_default();
    let data_dir = app.path().app_data_dir().ok();
    tauri::async_runtime::spawn_blocking(move || compute_day(home, data_dir))
        .await
        .map_err(|e| e.to_string())
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .invoke_handler(tauri::generate_handler![get_day])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let auto = CheckMenuItem::with_id(
                app, "autostart", "Start with Windows", true,
                app.autolaunch().is_enabled().unwrap_or(false), None::<&str>,
            )?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &auto, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, ev| match ev.id().as_ref() {
                    "open" => show_main(app),
                    "quit" => app.exit(0),
                    "autostart" => {
                        let al = app.autolaunch();
                        if al.is_enabled().unwrap_or(false) {
                            let _ = al.disable();
                        } else {
                            let _ = al.enable();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, ev| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = ev
                    {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
