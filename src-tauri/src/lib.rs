mod cli;
mod commands;
mod desktop;
mod egress;
mod error;
mod install;
mod locale;
mod preference;
mod system;
mod tray;
mod update;
mod window;

#[cfg(target_os = "macos")]
use tauri::RunEvent;
use tauri::{Manager, WindowEvent};

use crate::egress::Egress;
use crate::tray::Tray;
use crate::update::{PendingUpdate, spawn_update_checker};
use crate::window::{MainWindow, dismiss_popover, launch_main_window, show_main_window};

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "clawdninja=info,clawdninja_core=info".into()),
        )
        .init();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_port,
            commands::is_app_installed,
            commands::launch_app,
            commands::open_app_download,
            commands::is_cli_installed,
            commands::launch_cli,
            commands::install_cli,
            commands::is_proxy_applied,
            commands::apply_proxy,
            commands::remove_proxy,
            #[cfg(target_os = "macos")]
            commands::get_title_bar_metrics,
            commands::is_popover,
            commands::get_pending_update,
            commands::install_update
        ])
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            app.manage(Egress::load(config_dir.join("egress-port")));
            app.manage(MainWindow::load(config_dir.join("tray-popover")));
            app.manage(Tray::new(app.handle())?);
            app.manage(PendingUpdate::default());
            spawn_update_checker(app.handle().clone());
            launch_main_window(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
            }
            WindowEvent::Focused(false) => dismiss_popover(window.app_handle()),
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("failed to build ClawdNinja");

    app.run(|app, event| {
        #[cfg(target_os = "macos")]
        if let RunEvent::Reopen { .. } = event {
            show_main_window(app);
        }
        #[cfg(not(target_os = "macos"))]
        let _ = (app, event);
    });
}
