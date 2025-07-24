use tauri::ipc::Channel;
use tauri_plugin_updater::UpdaterExt;

use crate::types::{AppEvent, AppState};

pub async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        let mut downloaded = 0;

        // alternatively we could also call update.download() and update.install() separately
        update.download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    println!("downloaded {downloaded} from {content_length:?}");
                },
                || {
                println!("download finished");
                },
            )
            .await?;

        println!("update installed");
        app.restart();
    }

  Ok(())
}

#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<bool, String> {
    match app.updater() {
        Ok(updater) => {
            match updater.check().await {
                Ok(Some(_update)) => {
                    println!("Update available");
                    Ok(true)
                },
                Ok(None) => {
                    println!("No update available");
                    Ok(false)
                },
                Err(e) => {
                    eprintln!("Failed to check for updates: {}", e);
                    Err(format!("Failed to check for updates: {}", e))
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to initialize updater: {}", e);
            Err(format!("Failed to initialize updater: {}", e))
        }
    }
}
#[tauri::command]
pub async fn perform_update(app: tauri::AppHandle) -> Result<(), String> {
    match update(app).await {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Update failed: {}", e);
            Err(format!("Update failed: {}", e))
        }
    }
}
#[tauri::command]
pub async fn exit_app(app: tauri::AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}
#[tauri::command]
pub async fn restart_app(
    app: tauri::AppHandle
) -> Result<(), String> {
    app.restart();
}
#[tauri::command]
pub async fn start_channel_reads (
    state: tauri::State<'_, AppState>,
    channel: Channel<AppEvent>,
) -> Result<String, String> {
    while let Ok(event) = state.events_channel_reciever.lock().recv() {
        channel.send(event).expect("Failed to send event");
    }

    Err(String::from("Channel reads closed early?"))
}