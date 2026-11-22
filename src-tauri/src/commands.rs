use std::num::NonZeroU16;

use clawdninja_core::settings_path;
use tauri::{AppHandle, Emitter, State};

use crate::desktop::App;
use crate::egress::Egress;
use crate::error::CommandError;
use crate::update::{PendingUpdate, update_failed};
use crate::window::MainWindow;
use crate::{cli, desktop, install, system};

#[tauri::command]
pub fn get_port(egress: State<'_, Egress>) -> NonZeroU16 {
    egress.port()
}

#[tauri::command]
pub async fn is_app_installed(app: App) -> Result<bool, CommandError> {
    desktop::installed(app)
}

#[tauri::command]
pub async fn launch_app(
    egress: State<'_, Egress>,
    app: App,
    port: NonZeroU16,
) -> Result<(), CommandError> {
    desktop::launch(app, &egress.connect(port)?)
}

#[tauri::command]
pub async fn open_app_download(app: App) -> Result<String, CommandError> {
    let page = app.download_page();
    system::open_url(page)?;
    Ok(page.to_owned())
}

#[tauri::command]
pub async fn is_cli_installed() -> bool {
    cli::locate().is_some()
}

#[tauri::command]
pub async fn launch_cli(egress: State<'_, Egress>, port: NonZeroU16) -> Result<(), CommandError> {
    let url = egress.connect(port)?;
    cli::open_terminal(&url, &cli::locate().ok_or_else(|| CommandError::code("cli_missing"))?)
}

#[tauri::command]
pub async fn install_cli(
    app: AppHandle,
    egress: State<'_, Egress>,
    port: NonZeroU16,
) -> Result<(), CommandError> {
    let url = egress.connect(port)?;
    tauri::async_runtime::spawn_blocking(move || {
        install::install_cli(&url, |stage| {
            let _ = app.emit("install://stage", stage);
        })
    })
    .await
    .map_err(install::install_failed)??;
    cli::locate().map(|_| ()).ok_or_else(|| CommandError::code("install_incomplete"))
}

#[tauri::command]
pub async fn is_proxy_applied() -> Result<bool, CommandError> {
    Ok(clawdninja_core::is_configured(&settings_path()?)?)
}

#[tauri::command]
pub async fn apply_proxy(
    egress: State<'_, Egress>,
    port: NonZeroU16,
) -> Result<String, CommandError> {
    let url = egress.connect(port)?;
    let path = settings_path()?;
    clawdninja_core::write(&path, &url)?;
    Ok(path.display().to_string())
}

#[tauri::command]
pub async fn remove_proxy() -> Result<(), CommandError> {
    Ok(clawdninja_core::remove(&settings_path()?)?)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn get_title_bar_metrics(
    window: tauri::WebviewWindow,
) -> Option<crate::window::TitleBarMetrics> {
    crate::window::title_bar_metrics(&window)
}

#[tauri::command]
pub fn is_popover(main_window: State<'_, MainWindow>) -> bool {
    main_window.is_popover()
}

#[tauri::command]
pub fn get_pending_update(pending: State<'_, PendingUpdate>) -> Option<String> {
    pending.0.lock().ok()?.as_ref().map(|update| update.version.clone())
}

#[tauri::command]
pub async fn install_update(
    app: AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<(), CommandError> {
    let update = pending
        .0
        .lock()
        .ok()
        .and_then(|pending| pending.clone())
        .ok_or_else(|| update_failed("no update available"))?;
    let bytes = update.download(|_, _| {}, || {}).await.map_err(update_failed)?;
    update.install(bytes).map_err(update_failed)?;
    app.request_restart();
    Ok(())
}
