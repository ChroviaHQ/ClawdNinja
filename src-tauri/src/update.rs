use std::sync::Mutex;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::locale::labels;
use crate::tray::Tray;
use crate::window::show_main_window;

const FIRST_CHECK_DELAY: Duration = Duration::from_secs(30);
const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Default)]
pub struct PendingUpdate(pub Mutex<Option<Update>>);

pub fn spawn_update_checker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let start = tokio::time::Instant::now() + FIRST_CHECK_DELAY;
        let mut interval = tokio::time::interval_at(start, CHECK_INTERVAL);
        loop {
            interval.tick().await;
            if matches!(check(&app).await, Ok(true)) {
                break;
            }
        }
    });
}

pub fn spawn_manual_update_check(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let tray = app.state::<Tray>();
        tray.set_checking_update(true);
        let result = check(&app).await;
        tray.set_checking_update(false);
        match result {
            Ok(true) => show_main_window(&app),
            Ok(false) => show_message(
                &app,
                labels().up_to_date.replace("{version}", &app.package_info().version.to_string()),
            ),
            Err(error) => {
                show_message(&app, labels().check_failed.replace("{detail}", &error.to_string()))
            }
        }
    });
}

fn show_message(app: &AppHandle, message: String) {
    app.dialog().message(message).title("ClawdNinja").show(|_| {});
}

async fn check(app: &AppHandle) -> tauri_plugin_updater::Result<bool> {
    let pending = app.state::<PendingUpdate>();
    if pending.0.lock().is_ok_and(|update| update.is_some()) {
        return Ok(true);
    }
    let result = async { app.updater()?.check().await }.await;
    let Some(update) = result.inspect_err(|error| tracing::warn!(%error, "update check failed"))?
    else {
        return Ok(false);
    };
    let version = update.version.clone();
    if let Ok(mut pending) = pending.0.lock() {
        *pending = Some(update);
    }
    let _ = app.emit("app://update", version);
    Ok(true)
}
