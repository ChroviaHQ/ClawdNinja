use std::process::Command;

use clawdninja_core::variables;

use super::{Spec, State, chromium_arguments, proxy_argument, stderr};
use crate::error::CommandError;
use crate::system::{hidden, system32};

const EXIT_READY: i32 = 0;
const EXIT_MISSING: i32 = 2;
const EXIT_RUNNING: i32 = 3;

fn run_script(app: &Spec, url: Option<&str>) -> Result<State, CommandError> {
    let code = if url.is_none() { "probe_failed" } else { "launch_failed" };
    let mut command = Command::new(system32(r"WindowsPowerShell\v1.0\powershell.exe"));
    hidden(&mut command)
        .args(["-NoProfile", "-NonInteractive", "-Command", include_str!("launch.ps1")])
        .env("CLAWDNINJA_PACKAGE", app.locator.package)
        .env("CLAWDNINJA_ROOTS", app.locator.roots.join(";"))
        .env("CLAWDNINJA_EXECUTABLE", app.locator.executable);
    if let Some(url) = url {
        command
            .env("CLAWDNINJA_ARGUMENTS", chromium_arguments(app, url).join(" "))
            .env("CLAWDNINJA_PROXY_ARGUMENT", proxy_argument(url))
            .env("CLAWDNINJA_NEW_WINDOW_ARGUMENT", app.new_window_argument.unwrap_or_default())
            .envs(variables(url));
    }
    let output = command.output().map_err(|error| CommandError::new(code, error))?;
    match output.status.code() {
        Some(EXIT_READY) => Ok(State::Ready),
        Some(EXIT_MISSING) => Ok(State::Missing),
        Some(EXIT_RUNNING) => Ok(State::Running(())),
        _ => Err(CommandError::new(code, stderr(&output))),
    }
}

pub(super) fn installed(app: &Spec) -> Result<bool, CommandError> {
    Ok(!matches!(run_script(app, None)?, State::Missing))
}

pub(super) fn launch(app: &Spec, url: &str) -> Result<(), CommandError> {
    run_script(app, Some(url))?.require_ready()
}
