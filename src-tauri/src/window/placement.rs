use tauri::{AppHandle, Monitor, PhysicalPosition, PhysicalSize, Rect, WebviewWindow};

use crate::tray::TRAY_ID;

const POPOVER_MARGIN: f64 = 6.0;

#[derive(Clone, Copy)]
struct Bounds {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl Bounds {
    fn new(position: PhysicalPosition<f64>, size: PhysicalSize<f64>) -> Self {
        Self {
            left: position.x,
            top: position.y,
            right: position.x + size.width,
            bottom: position.y + size.height,
        }
    }

    fn center(self) -> (f64, f64) {
        ((self.left + self.right) / 2.0, (self.top + self.bottom) / 2.0)
    }

    fn contains(self, (x, y): (f64, f64)) -> bool {
        (self.left..self.right).contains(&x) && (self.top..self.bottom).contains(&y)
    }

    fn inset(self, amount: f64) -> Self {
        Self {
            left: self.left + amount,
            top: self.top + amount,
            right: self.right - amount,
            bottom: self.bottom - amount,
        }
    }
}

pub(super) fn popover_position(
    app: &AppHandle,
    window: &WebviewWindow,
) -> Option<(PhysicalPosition<f64>, f64)> {
    let anchor = tray_bounds(app).or_else(|| cursor_bounds(app))?;
    let center = anchor.center();
    let monitor = monitor_containing(window, center)?;
    let scale = monitor.scale_factor();
    let work = monitor.work_area();
    let area = Bounds::new(work.position.cast(), work.size.cast()).inset(POPOVER_MARGIN * scale);
    let PhysicalSize { width, height } = window
        .outer_size()
        .ok()?
        .to_logical::<f64>(window.scale_factor().ok()?)
        .to_physical::<f64>(scale);
    let x = center.0 - width / 2.0;
    let y = if center.1 < area.center().1 { anchor.bottom } else { anchor.top - height };
    let position = PhysicalPosition::new(
        x.min(area.right - width).max(area.left).round(),
        y.min(area.bottom - height).max(area.top).round(),
    );
    Some((position, scale))
}

fn tray_bounds(app: &AppHandle) -> Option<Bounds> {
    let Rect { position, size } = app.tray_by_id(TRAY_ID)?.rect().ok()??;
    Some(Bounds::new(position.to_physical(1.0), size.to_physical(1.0)))
}

fn cursor_bounds(app: &AppHandle) -> Option<Bounds> {
    Some(Bounds::new(app.cursor_position().ok()?, PhysicalSize::new(0.0, 0.0)))
}

fn monitor_containing(window: &WebviewWindow, point: (f64, f64)) -> Option<Monitor> {
    window
        .available_monitors()
        .ok()?
        .into_iter()
        .find(|monitor| {
            Bounds::new(monitor.position().cast(), monitor.size().cast()).contains(point)
        })
        .or_else(|| window.primary_monitor().ok().flatten())
}
