use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use clawdninja_core::variables;

use crate::error::CommandError;

#[cfg(windows)]
const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

fn terminal_failed(error: impl ToString) -> CommandError {
    CommandError::new("terminal_failed", error)
}

fn executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata().is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(windows)]
    {
        path.is_file()
    }
}

pub fn locate() -> Option<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/bin"));
        dirs.push(home.join(".volta/bin"));
        #[cfg(target_os = "macos")]
        {
            let nvm =
                env::var_os("NVM_DIR").map(PathBuf::from).unwrap_or_else(|| home.join(".nvm"));
            if let Ok(versions) = std::fs::read_dir(nvm.join("versions/node")) {
                let mut bins =
                    versions.flatten().map(|entry| entry.path().join("bin")).collect::<Vec<_>>();
                bins.sort_by_cached_key(|p| {
                    std::cmp::Reverse(p.metadata().and_then(|m| m.modified()).ok())
                });
                dirs.extend(bins);
            }
        }
    }
    #[cfg(windows)]
    if let Some(appdata) = env::var_os("APPDATA") {
        dirs.push(PathBuf::from(appdata).join("npm"));
    }
    #[cfg(windows)]
    if let Some(local) = env::var_os("LOCALAPPDATA") {
        dirs.push(PathBuf::from(local).join("Microsoft/WinGet/Links"));
    }
    #[cfg(unix)]
    dirs.extend(["/opt/homebrew/bin", "/usr/local/bin"].map(PathBuf::from));
    dirs.extend(
        env::split_paths(&env::var_os("PATH").unwrap_or_default()).filter(|p| p.is_absolute()),
    );
    #[cfg(windows)]
    let names = ["claude.exe", "claude.cmd", "claude.bat"];
    #[cfg(unix)]
    let names = ["claude"];
    dirs.iter().flat_map(|dir| names.iter().map(move |name| dir.join(name))).find(|p| executable(p))
}

#[cfg(target_os = "macos")]
pub fn open_terminal(url: &str, cli: &Path) -> Result<(), CommandError> {
    use std::{io::Write, os::unix::fs::PermissionsExt};
    let mut script = tempfile::Builder::new()
        .prefix("ClawdNinja-")
        .suffix(".command")
        .tempfile()
        .map_err(terminal_failed)?;
    let quote = |s: &str| format!("'{}'", s.replace('\'', "'\\''"));
    let mut body = String::from("#!/bin/sh\n/bin/rm -f -- \"$0\"\ncd \"$HOME\" || exit 1\n");
    for (key, value) in variables(url) {
        body.push_str(&format!("export {key}={}\n", quote(value)));
    }
    if let Some(parent) = cli.parent().and_then(Path::to_str) {
        body.push_str(&format!("export PATH={}:\"$PATH\"\n", quote(parent)));
    }
    body.push_str("export BASH_SILENCE_DEPRECATION_WARNING=1\nprintf '\\033[H\\033[2J\\033[3J'\n");
    body.push_str("exec \"${SHELL:-/bin/zsh}\" -l\n");
    script.write_all(body.as_bytes()).map_err(terminal_failed)?;
    script
        .as_file()
        .set_permissions(std::fs::Permissions::from_mode(0o700))
        .map_err(terminal_failed)?;
    let (_, path) = script.keep().map_err(terminal_failed)?;
    let result =
        Command::new("/usr/bin/open").args(["-b", "com.apple.Terminal"]).arg(&path).status();
    if result.is_ok_and(|s| s.success()) {
        Ok(())
    } else {
        let _ = std::fs::remove_file(path);
        Err(CommandError::code("terminal_failed"))
    }
}

#[cfg(windows)]
pub fn open_terminal(url: &str, cli: &Path) -> Result<(), CommandError> {
    use std::os::windows::process::CommandExt;
    let mut paths: Vec<PathBuf> = cli.parent().map(Path::to_path_buf).into_iter().collect();
    paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
    let mut cmd = Command::new(crate::system::system32("cmd.exe"));
    cmd.creation_flags(CREATE_NEW_CONSOLE)
        .envs(variables(url))
        .env("PATH", env::join_paths(paths).map_err(terminal_failed)?)
        .raw_arg("/D /V:OFF /K title ClawdNinja");
    if let Some(home) = dirs::home_dir() {
        cmd.current_dir(home);
    }
    cmd.spawn().map(|_| ()).map_err(terminal_failed)
}
