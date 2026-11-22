use objc2::MainThreadMarker;
use objc2_app_kit::{NSScreen, NSWindow, NSWindowButton};
use objc2_foundation::{NSPoint, NSProcessInfo};
use serde::Serialize;
use tauri::utils::config::WindowConfig;
use tauri::{Emitter, PhysicalPosition, WebviewWindow};

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

pub(super) fn apply_popover_chrome(window: &WebviewWindow, popover: bool) {
    with_ns_window(window, move |ns_window, _| {
        ns_window.setMovable(!popover);
        for kind in [
            NSWindowButton::CloseButton,
            NSWindowButton::MiniaturizeButton,
            NSWindowButton::ZoomButton,
        ] {
            if let Some(button) = ns_window.standardWindowButton(kind) {
                button.setHidden(popover);
            }
        }
    });
}

pub(super) fn set_position(window: &WebviewWindow, position: PhysicalPosition<f64>, scale: f64) {
    let position = position.to_logical::<f64>(scale);
    with_ns_window(window, move |ns_window, mtm| {
        if let Some(primary) = NSScreen::screens(mtm).firstObject() {
            ns_window.setFrameTopLeftPoint(NSPoint::new(
                position.x,
                primary.frame().size.height - position.y,
            ));
        }
    });
}

fn with_ns_window(
    window: &WebviewWindow,
    action: impl FnOnce(&NSWindow, MainThreadMarker) + Send + 'static,
) {
    let target = window.clone();
    let _ = window.run_on_main_thread(move || {
        if let (Some(mtm), Ok(ns_window)) = (MainThreadMarker::new(), target.ns_window()) {
            action(unsafe { &*ns_window.cast::<NSWindow>() }, mtm);
        }
    });
}
