use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager, WebviewWindow, WebviewWindowBuilder};

use crate::preference::Preference;
use placement::popover_position;

#[cfg(target_os = "macos")]
mod macos;
mod placement;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(windows)]
use windows as platform;

#[cfg(target_os = "macos")]
pub use macos::{TitleBarMetrics, title_bar_metrics};

const MAIN_WINDOW_LABEL: &str = "main";
const DISMISS_SETTLE: Duration = Duration::from_millis(300);

pub struct MainWindow {
    popover: Preference<bool>,
    dismissed_at: Mutex<Option<Instant>>,
}

impl MainWindow {
    pub fn load(path: PathBuf) -> Self {
        Self { popover: Preference::load(path, true), dismissed_at: Mutex::new(None) }
    }

    pub fn is_popover(&self) -> bool {
        self.popover.get()
    }

    fn mark_dismissed(&self) {
        *self.dismissed_at.lock().unwrap_or_else(PoisonError::into_inner) = Some(Instant::now());
    }

    fn dismissed_recently(&self) -> bool {
        self.dismissed_at
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some_and(|at| at.elapsed() < DISMISS_SETTLE)
    }
}

pub fn launch_main_window(app: &AppHandle) -> tauri::Result<()> {
    let window_configs = &app.config().app.windows;
    let Some(config) = window_configs.iter().find(|config| config.label == MAIN_WINDOW_LABEL)
    else {
        return Ok(());
    };
    let mut config = config.clone();
    platform::configure_main_window(&mut config);
    let window = WebviewWindowBuilder::from_config(app, &config)?.build()?;
    let popover = app.state::<MainWindow>().is_popover();
    apply_popover(&window, popover);
    if !popover {
        show_main_window(app);
    }
    Ok(())
}

pub fn show_main_window(app: &AppHandle) {
    let Some(window) = main_window(app) else {
        return;
    };
    let popover = app.state::<MainWindow>().is_popover();
    if popover && let Some((position, scale)) = popover_position(app, &window) {
        platform::set_position(&window, position, scale);
    }
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    if !popover {
        platform::publish_title_bar_metrics(&window);
    }
}

pub fn toggle_main_window(app: &AppHandle) {
    let state = app.state::<MainWindow>();
    if state.is_popover() && (state.dismissed_recently() || is_visible(app)) {
        dismiss_popover(app);
    } else {
        show_main_window(app);
    }
}

pub fn dismiss_popover(app: &AppHandle) {
    let state = app.state::<MainWindow>();
    if !state.is_popover() {
        return;
    }
    if let Some(window) = main_window(app) {
        let _ = window.hide();
    }
    state.mark_dismissed();
}

pub fn set_popover(app: &AppHandle, popover: bool) {
    app.state::<MainWindow>().popover.set(popover);
    let Some(window) = main_window(app) else {
        return;
    };
    apply_popover(&window, popover);
    if window.is_visible().unwrap_or(false) {
        show_main_window(app);
    }
}

fn main_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(MAIN_WINDOW_LABEL)
}

fn is_visible(app: &AppHandle) -> bool {
    main_window(app).is_some_and(|window| window.is_visible().unwrap_or(false))
}

fn apply_popover(window: &WebviewWindow, popover: bool) {
    let _ = window.set_always_on_top(popover);
    let _ = window.set_skip_taskbar(popover);
    let _ = window.set_visible_on_all_workspaces(popover);
    platform::apply_popover_chrome(window, popover);
    let _ = window.emit("window://popover", popover);
}
