use std::process::{Command, Stdio};

use clawdninja_core::variables;

use super::{Spec, State, chromium_arguments, proxy_argument, stderr};
use crate::error::CommandError;

fn checked_output(command: &mut Command, code: &'static str) -> Result<Vec<u8>, CommandError> {
    let output = command.output().map_err(|error| CommandError::new(code, error))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(CommandError::new(code, stderr(&output)))
    }
}

fn lookup(app: &Spec) -> Result<State<String>, CommandError> {
    let stdout = checked_output(
        Command::new("/usr/bin/osascript").args([
            "-l",
            "JavaScript",
            "-e",
            include_str!("lookup.js"),
            app.locator.bundle_id,
        ]),
        "probe_failed",
    )?;
    let output = String::from_utf8_lossy(&stdout).trim().to_owned();
    Ok(match output.as_str() {
        "missing" => State::Missing,
        "ready" => State::Ready,
        _ => State::Running(output),
    })
}

pub(super) fn installed(app: &Spec) -> Result<bool, CommandError> {
    Ok(!matches!(lookup(app)?, State::Missing))
}

fn background_running(pattern: &str) -> bool {
    Command::new("/usr/bin/pgrep")
        .args(["-f", pattern])
        .stdout(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn running_with_proxy(pids: &str, url: &str) -> bool {
    let proxy = proxy_argument(url);
    checked_output(Command::new("/bin/ps").args(["-ww", "-o", "args=", "-p", pids]), "probe_failed")
        .is_ok_and(|stdout| {
            String::from_utf8_lossy(&stdout).split_whitespace().any(|token| token == proxy)
        })
}

pub(super) fn launch(app: &Spec, url: &str) -> Result<(), CommandError> {
    let new_window_argument = match (lookup(app)?, app.new_window_argument) {
        (State::Running(pids), Some(argument)) if running_with_proxy(&pids, url) => Some(argument),
        (state, _) => {
            state.require_ready()?;
            if app.locator.background.is_some_and(background_running) {
                return Err(CommandError::code("background_running"));
            }
            None
        }
    };
    let mut open = Command::new("/usr/bin/open");
    open.args(new_window_argument.map(|_| "-n")).args(["-b", app.locator.bundle_id]);
    for (key, value) in variables(url) {
        open.arg("--env").arg(format!("{key}={value}"));
    }
    let mut arguments = chromium_arguments(app, url);
    arguments.extend(new_window_argument.map(str::to_owned));
    if !arguments.is_empty() {
        open.arg("--args").args(arguments);
    }
    checked_output(&mut open, "launch_failed").map(|_| ())
}
