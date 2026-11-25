mod espn;
mod state;
mod storage;
mod tray;

use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime, WindowEvent};

use state::{AppState, FollowedTeam, GameDisplay, SharedState};

pub fn show_settings<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("settings") {
        #[cfg(target_os = "macos")]
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn hide_settings<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.hide();
    }
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
}

pub fn spawn_refresh<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = state::shared(&app);
        match state.refresh().await {
            Ok(()) => {}
            Err(e) => eprintln!("refresh failed: {e}"),
        }
        if let Err(e) = tray::rebuild(&app) {
            eprintln!("tray rebuild failed: {e}");
        }
    });
}

fn start_auto_refresh(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(800)).await;
        loop {
            let state = state::shared(&app);
            match state.refresh().await {
                Ok(()) => {}
                Err(e) => eprintln!("refresh failed: {e}"),
            }
            if let Err(e) = tray::rebuild(&app) {
                eprintln!("tray rebuild failed: {e}");
            }
            let secs = if state.has_live() { 30 } else { 120 };
            tokio::time::sleep(Duration::from_secs(secs)).await;
        }
    });
}

#[tauri::command]
fn get_followed_teams(state: tauri::State<'_, SharedState>) -> Vec<FollowedTeam> {
    state.teams()
}

#[tauri::command]
fn add_team(state: tauri::State<'_, SharedState>, team: FollowedTeam) -> Result<(), String> {
    state.add_team(team).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn remove_team(state: tauri::State<'_, SharedState>, team_id: String) -> Result<(), String> {
    state.remove_team(&team_id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_current_games(state: tauri::State<'_, SharedState>) -> Vec<GameDisplay> {
    state.games()
}

#[tauri::command]
async fn search_teams(
    state: tauri::State<'_, SharedState>,
    query: String,
) -> Result<Vec<espn::Team>, String> {
    state.search_teams(&query).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn open_settings(app: AppHandle) {
    show_settings(&app);
}

#[tauri::command]
fn close_settings(app: AppHandle) {
    hide_settings(&app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let state: SharedState = Arc::new(AppState::new()?);
            app.manage(state);

            tray::build(app.handle())?;
            start_auto_refresh(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
                #[cfg(target_os = "macos")]
                let _ = window
                    .app_handle()
                    .set_activation_policy(tauri::ActivationPolicy::Accessory);
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_followed_teams,
            add_team,
            remove_team,
            get_current_games,
            search_teams,
            open_settings,
            close_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
