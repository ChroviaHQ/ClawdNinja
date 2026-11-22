use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::locale::labels;
use crate::update::spawn_manual_update_check;
use crate::window::{MainWindow, set_popover, show_main_window, toggle_main_window};

pub const TRAY_ID: &str = "main";

#[cfg(target_os = "macos")]
const ICON: &[u8] = include_bytes!("../icons/tray.png");
#[cfg(not(target_os = "macos"))]
const ICON: &[u8] = include_bytes!("../icons/tray-color.png");
const ICON_IS_TEMPLATE: bool = cfg!(target_os = "macos");

pub struct Tray {
    popover: CheckMenuItem<Wry>,
    check_update: MenuItem<Wry>,
}

impl Tray {
    pub fn new(app: &AppHandle) -> tauri::Result<Self> {
        let labels = labels();
        let open = MenuItem::with_id(app, "open", &labels.open, true, None::<&str>)?;
        let popover = CheckMenuItem::with_id(
            app,
            "popover",
            &labels.popover,
            true,
            app.state::<MainWindow>().is_popover(),
            None::<&str>,
        )?;
        let check_update =
            MenuItem::with_id(app, "check_update", &labels.check_update, true, None::<&str>)?;
        let quit = MenuItem::with_id(app, "quit", &labels.quit, true, None::<&str>)?;
        let menu = Menu::with_items(
            app,
            &[&open, &PredefinedMenuItem::separator(app)?, &popover, &check_update, &quit],
        )?;

        TrayIconBuilder::with_id(TRAY_ID)
            .icon(Image::from_bytes(ICON)?)
            .icon_as_template(ICON_IS_TEMPLATE)
            .tooltip("ClawdNinja")
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(|app, event| match event.id().as_ref() {
                "open" => show_main_window(app),
                "popover" => {
                    if let Ok(checked) = app.state::<Tray>().popover.is_checked() {
                        set_popover(app, checked);
                    }
                }
                "check_update" => spawn_manual_update_check(app.clone()),
                "quit" => app.exit(0),
                _ => {}
            })
            .on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    toggle_main_window(tray.app_handle());
                }
            })
            .build(app)?;

        Ok(Self { popover, check_update })
    }

    pub fn set_checking_update(&self, checking: bool) {
        let labels = labels();
        let _ = self.check_update.set_text(if checking {
            &labels.checking_update
        } else {
            &labels.check_update
        });
        let _ = self.check_update.set_enabled(!checking);
    }
}
