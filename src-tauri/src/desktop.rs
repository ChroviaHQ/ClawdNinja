use std::process::Command;

use clawdninja_core::variables;
use serde::Deserialize;

use crate::error::CommandError;
#[cfg(target_os = "windows")]
use crate::system::{hidden, system32};

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum App {
    Claude,
    Science,
    Chatgpt,
    Vscode,
}

impl App {
    fn spec(self) -> &'static Spec {
        match self {
            App::Claude => &CLAUDE,
            App::Science => &SCIENCE,
            App::Chatgpt => &CHATGPT,
            App::Vscode => &VSCODE,
        }
    }

    pub fn download_page(self) -> &'static str {
        self.spec().download
    }
}

struct Spec {
    download: &'static str,
    locator: Locator,
    chromium: bool,
}

#[cfg(target_os = "macos")]
struct Locator {
    bundle_id: &'static str,
    background: Option<&'static str>,
}

#[cfg(target_os = "windows")]
struct Locator {
    package: &'static str,
    roots: &'static [&'static str],
    executable: &'static str,
}

const CLAUDE: Spec = Spec {
    download: "https://claude.com/download",
    #[cfg(target_os = "macos")]
    locator: Locator { bundle_id: "com.anthropic.claudefordesktop", background: None },
    #[cfg(target_os = "windows")]
    locator: Locator {
        package: "Claude",
        roots: &[r"%LOCALAPPDATA%\AnthropicClaude"],
        executable: "claude.exe",
    },
    chromium: true,
};

const SCIENCE: Spec = Spec {
    download: "https://claude.com/product/claude-science",
    #[cfg(target_os = "macos")]
    locator: Locator {
        bundle_id: "com.anthropic.operon",
        background: Some("(^|/)claude-science serve( |$)"),
    },
    #[cfg(target_os = "windows")]
    locator: Locator {
        package: "",
        roots: &[r"%LOCALAPPDATA%\Programs\ClaudeScience"],
        executable: "claude-science.exe",
    },
    chromium: false,
};

const CHATGPT: Spec = Spec {
    download: "https://chatgpt.com/download",
    #[cfg(target_os = "macos")]
    locator: Locator { bundle_id: "com.openai.codex", background: None },
    #[cfg(target_os = "windows")]
    locator: Locator { package: "OpenAI.Codex", roots: &[], executable: "ChatGPT.exe" },
    chromium: true,
};

const VSCODE: Spec = Spec {
    download: "https://code.visualstudio.com/download",
    #[cfg(target_os = "macos")]
    locator: Locator { bundle_id: "com.microsoft.VSCode", background: None },
    #[cfg(target_os = "windows")]
    locator: Locator {
        package: "",
        roots: &[r"%LOCALAPPDATA%\Programs\Microsoft VS Code", r"%ProgramFiles%\Microsoft VS Code"],
        executable: "Code.exe",
    },
    chromium: true,
};

fn chromium_arguments(app: &Spec, url: &str) -> Vec<String> {
    if !app.chromium {
        return Vec::new();
    }
    vec![
        format!("--proxy-server={url}"),
        "--force-webrtc-ip-handling-policy=disable_non_proxied_udp".to_owned(),
    ]
}

enum State {
    Missing,
    Ready,
    Running(&'static str),
}

impl State {
    fn ready(self) -> Result<(), CommandError> {
        match self {
            State::Missing => Err(CommandError::code("app_missing")),
            State::Running(code) => Err(CommandError::code(code)),
            State::Ready => Ok(()),
        }
    }
}

#[cfg(target_os = "windows")]
fn probe_failed(error: impl ToString) -> CommandError {
    CommandError::new("probe_failed", error)
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_owned()
}

#[cfg(target_os = "macos")]
fn checked_output(command: &mut Command, code: &'static str) -> Result<Vec<u8>, CommandError> {
    let output = command.output().map_err(|error| CommandError::new(code, error))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(CommandError::new(code, stderr(&output)))
    }
}

#[cfg(target_os = "macos")]
fn lookup(app: &Spec) -> Result<State, CommandError> {
    let script = format!(
        r#"ObjC.import("AppKit"); const id = "{}"; const url = $.NSWorkspace.sharedWorkspace.URLForApplicationWithBundleIdentifier(id); url.isNil() || !$.NSFileManager.defaultManager.fileExistsAtPath(url.path) ? "missing" : $.NSRunningApplication.runningApplicationsWithBundleIdentifier(id).count > 0 ? "running" : "ready""#,
        app.locator.bundle_id
    );
    let stdout = checked_output(
        Command::new("/usr/bin/osascript").args(["-l", "JavaScript", "-e", &script]),
        "probe_failed",
    )?;
    Ok(match stdout.trim_ascii() {
        b"missing" => State::Missing,
        b"running" => State::Running("app_running"),
        _ => State::Ready,
    })
}

#[cfg(target_os = "macos")]
fn background_running(pattern: &str) -> bool {
    Command::new("/usr/bin/pgrep")
        .args(["-f", pattern])
        .stdout(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(target_os = "macos")]
pub fn launch(app: App, url: &str) -> Result<(), CommandError> {
    let app = app.spec();
    lookup(app)?.ready()?;
    if app.locator.background.is_some_and(background_running) {
        return Err(CommandError::code("background_running"));
    }
    let mut open = Command::new("/usr/bin/open");
    open.args(["-b", app.locator.bundle_id]);
    for (key, value) in variables(url) {
        open.arg("--env").arg(format!("{key}={value}"));
    }
    let arguments = chromium_arguments(app, url);
    if !arguments.is_empty() {
        open.arg("--args").args(arguments);
    }
    checked_output(&mut open, "launch_failed").map(|_| ())
}

#[cfg(target_os = "windows")]
fn run_script(app: &Spec, launch: Option<&str>) -> Result<State, CommandError> {
    let result = hidden(Command::new(system32(r"WindowsPowerShell\v1.0\powershell.exe")).args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        include_str!("launch.ps1"),
    ]))
    .env("CLAWDNINJA_DETECT", if launch.is_none() { "1" } else { "0" })
    .env("CLAWDNINJA_PACKAGE", app.locator.package)
    .env("CLAWDNINJA_ROOTS", app.locator.roots.join(";"))
    .env("CLAWDNINJA_EXECUTABLE", app.locator.executable)
    .env(
        "CLAWDNINJA_ARGUMENT",
        launch.map(|url| chromium_arguments(app, url).join(" ")).unwrap_or_default(),
    )
    .envs(launch.into_iter().flat_map(variables))
    .output()
    .map_err(probe_failed)?;
    match result.status.code() {
        Some(0) => Ok(State::Ready),
        Some(2) => Ok(State::Missing),
        Some(3) => Ok(State::Running("app_running")),
        _ => Err(CommandError::new(
            if launch.is_none() { "probe_failed" } else { "launch_failed" },
            stderr(&result),
        )),
    }
}

#[cfg(target_os = "windows")]
fn lookup(app: &Spec) -> Result<State, CommandError> {
    run_script(app, None)
}

#[cfg(target_os = "windows")]
pub fn launch(app: App, url: &str) -> Result<(), CommandError> {
    run_script(app.spec(), Some(url))?.ready()
}

pub fn installed(app: App) -> Result<bool, CommandError> {
    Ok(!matches!(lookup(app.spec())?, State::Missing))
}
