use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::num::NonZeroU16;
use std::time::Duration;

use clawdninja_core::{proxy_url, settings_path};
use tauri::{AppHandle, Emitter, State};

use crate::desktop::App;
use crate::error::CommandError;
use crate::update::PendingUpdate;
use crate::{cli, desktop, install, system};

const PROXY_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

fn reachable_proxy(port: NonZeroU16) -> Result<String, CommandError> {
    TcpStream::connect_timeout(
        &SocketAddr::from((Ipv4Addr::LOCALHOST, port.get())),
        PROXY_CONNECT_TIMEOUT,
    )
    .map_err(|error| CommandError::new("egress_unreachable", error))?;
    Ok(proxy_url(port))
}

#[tauri::command]
pub async fn is_app_installed(app: App) -> Result<bool, CommandError> {
    desktop::installed(app)
}

#[tauri::command]
pub async fn launch_app(app: App, port: NonZeroU16) -> Result<(), CommandError> {
    desktop::launch(app, &reachable_proxy(port)?)
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
pub async fn launch_cli(port: NonZeroU16) -> Result<(), CommandError> {
    let url = reachable_proxy(port)?;
    cli::open_terminal(&url, &cli::locate().ok_or_else(|| CommandError::code("cli_missing"))?)
}

#[tauri::command]
pub async fn install_cli(app: AppHandle, port: NonZeroU16) -> Result<(), CommandError> {
    let url = reachable_proxy(port)?;
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
pub async fn apply_proxy(port: NonZeroU16) -> Result<String, CommandError> {
    let path = settings_path()?;
    clawdninja_core::write(&path, &proxy_url(port))?;
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

fn update_failed(error: impl ToString) -> CommandError {
    CommandError::new("update_failed", error)
}
