use std::process::Command;

use crate::error::CommandError;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
fn windows_dir() -> std::path::PathBuf {
    std::env::var_os("SystemRoot").map_or_else(|| r"C:\Windows".into(), Into::into)
}

#[cfg(windows)]
pub fn system32(tool: &str) -> std::path::PathBuf {
    windows_dir().join("System32").join(tool)
}

pub fn hidden(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

pub fn open_url(url: &str) -> Result<(), CommandError> {
    #[cfg(target_os = "macos")]
    let mut opener = Command::new("/usr/bin/open");
    #[cfg(windows)]
    let mut opener = Command::new(windows_dir().join("explorer.exe"));
    opener.arg(url).spawn().map(|_| ()).map_err(|error| CommandError::new("browser_failed", error))
}
