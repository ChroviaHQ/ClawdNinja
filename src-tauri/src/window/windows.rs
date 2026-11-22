use tauri::utils::config::WindowConfig;
use tauri::{PhysicalPosition, WebviewWindow};

pub(super) fn configure_main_window(config: &mut WindowConfig) {
    config.decorations = false;
    config.shadow = true;
}

pub(super) fn publish_title_bar_metrics(_window: &WebviewWindow) {}

pub(super) fn apply_popover_chrome(_window: &WebviewWindow, _popover: bool) {}

pub(super) fn set_position(window: &WebviewWindow, position: PhysicalPosition<f64>, _scale: f64) {
    let _ = window.set_position(position.cast::<i32>());
}
