use serde::Deserialize;

use crate::error::CommandError;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(windows)]
use windows as platform;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum App {
    Claude,
    Science,
    Chatgpt,
    Vscode,
    Antigravity,
}

impl App {
    fn spec(self) -> &'static Spec {
        match self {
            App::Claude => &CLAUDE,
            App::Science => &SCIENCE,
            App::Chatgpt => &CHATGPT,
            App::Vscode => &VSCODE,
            App::Antigravity => &ANTIGRAVITY,
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
    new_window_argument: Option<&'static str>,
}

#[cfg(target_os = "macos")]
struct Locator {
    bundle_id: &'static str,
    background: Option<&'static str>,
}

#[cfg(windows)]
struct Locator {
    package: &'static str,
    roots: &'static [&'static str],
    executable: &'static str,
}

const CLAUDE: Spec = Spec {
    download: "https://claude.com/download",
    #[cfg(target_os = "macos")]
    locator: Locator { bundle_id: "com.anthropic.claudefordesktop", background: None },
    #[cfg(windows)]
    locator: Locator {
        package: "Claude",
        roots: &[r"%LOCALAPPDATA%\AnthropicClaude"],
        executable: "claude.exe",
    },
    chromium: true,
    new_window_argument: None,
};

const SCIENCE: Spec = Spec {
    download: "https://claude.com/product/claude-science",
    #[cfg(target_os = "macos")]
    locator: Locator {
        bundle_id: "com.anthropic.operon",
        background: Some("(^|/)claude-science serve( |$)"),
    },
    #[cfg(windows)]
    locator: Locator {
        package: "",
        roots: &[r"%LOCALAPPDATA%\Programs\ClaudeScience"],
        executable: "claude-science.exe",
    },
    chromium: false,
    new_window_argument: None,
};

const CHATGPT: Spec = Spec {
    download: "https://chatgpt.com/download",
    #[cfg(target_os = "macos")]
    locator: Locator { bundle_id: "com.openai.codex", background: None },
    #[cfg(windows)]
    locator: Locator { package: "OpenAI.Codex", roots: &[], executable: "ChatGPT.exe" },
    chromium: true,
    new_window_argument: None,
};

const VSCODE: Spec = Spec {
    download: "https://code.visualstudio.com/download",
    #[cfg(target_os = "macos")]
    locator: Locator { bundle_id: "com.microsoft.VSCode", background: None },
    #[cfg(windows)]
    locator: Locator {
        package: "",
        roots: &[r"%LOCALAPPDATA%\Programs\Microsoft VS Code", r"%ProgramFiles%\Microsoft VS Code"],
        executable: "Code.exe",
    },
    chromium: true,
    new_window_argument: Some("--new-window"),
};

const ANTIGRAVITY: Spec = Spec {
    download: "https://antigravity.google/download",
    #[cfg(target_os = "macos")]
    locator: Locator { bundle_id: "com.google.antigravity", background: None },
    #[cfg(windows)]
    locator: Locator {
        package: "",
        roots: &[r"%LOCALAPPDATA%\Programs\Antigravity", r"%ProgramFiles%\Antigravity"],
        executable: "Antigravity.exe",
    },
    chromium: true,
    new_window_argument: None,
};

fn proxy_argument(url: &str) -> String {
    format!("--proxy-server={url}")
}

fn chromium_arguments(app: &Spec, url: &str) -> Vec<String> {
    if !app.chromium {
        return Vec::new();
    }
    vec![
        proxy_argument(url),
        "--force-webrtc-ip-handling-policy=disable_non_proxied_udp".to_owned(),
    ]
}

enum State<Processes = ()> {
    Missing,
    Ready,
    Running(Processes),
}

impl<Processes> State<Processes> {
    fn require_ready(self) -> Result<(), CommandError> {
        match self {
            State::Missing => Err(CommandError::code("app_missing")),
            State::Ready => Ok(()),
            State::Running(_) => Err(CommandError::code("app_running")),
        }
    }
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_owned()
}

pub fn launch(app: App, url: &str) -> Result<(), CommandError> {
    platform::launch(app.spec(), url)
}

pub fn installed(app: App) -> Result<bool, CommandError> {
    platform::installed(app.spec())
}
