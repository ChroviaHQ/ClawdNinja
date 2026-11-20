use tauri::WebviewWindow;
use tauri::utils::config::WindowConfig;

pub(super) fn configure_main_window(config: &mut WindowConfig) {
    config.decorations = false;
    config.shadow = true;
}

pub(super) fn publish_title_bar_metrics(_window: &WebviewWindow) {}
