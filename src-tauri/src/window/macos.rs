use objc2::MainThreadMarker;
use objc2_app_kit::{NSWindow, NSWindowButton};
use objc2_foundation::NSProcessInfo;
use serde::Serialize;
use tauri::utils::config::WindowConfig;
use tauri::{Emitter, WebviewWindow};

const MACOS_TAHOE_MAJOR_VERSION: isize = 26;
const PRE_TAHOE_TRAFFIC_LIGHT_OFFSET: f64 = 4.0;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleBarMetrics {
    height: f64,
    controls_width: f64,
}

pub(super) fn configure_main_window(config: &mut WindowConfig) {
    if let Some(position) = &mut config.traffic_light_position
        && major_version() < MACOS_TAHOE_MAJOR_VERSION
    {
        position.y -= PRE_TAHOE_TRAFFIC_LIGHT_OFFSET;
    }
}

pub fn title_bar_metrics(window: &WebviewWindow) -> Option<TitleBarMetrics> {
    MainThreadMarker::new()?;
    let ns_window = unsafe { &*window.ns_window().ok()?.cast::<NSWindow>() };
    if !ns_window.isVisible() {
        return None;
    }
    ns_window.display();
    let content = ns_window.contentView()?;
    let close = ns_window.standardWindowButton(NSWindowButton::CloseButton)?;
    let zoom = ns_window.standardWindowButton(NSWindowButton::ZoomButton)?;
    let close = close.convertRect_toView(close.bounds(), Some(&content));
    let zoom = zoom.convertRect_toView(zoom.bounds(), Some(&content));
    let center = close.origin.y + close.size.height / 2.0;
    let center_from_top =
        if content.isFlipped() { center } else { content.bounds().size.height - center };
    Some(TitleBarMetrics {
        height: center_from_top * 2.0,
        controls_width: zoom.origin.x + zoom.size.width,
    })
}

pub(super) fn publish_title_bar_metrics(window: &WebviewWindow) {
    let target = window.clone();
    let _ = window.run_on_main_thread(move || {
        if let Some(metrics) = title_bar_metrics(&target) {
            let _ = target.emit("window://title-bar-metrics", metrics);
        }
    });
}

fn major_version() -> isize {
    NSProcessInfo::processInfo().operatingSystemVersion().majorVersion
}
