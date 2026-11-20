use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use clawdninja_core::variables;

use crate::error::CommandError;
use crate::system::hidden;

const FAILURE_TAIL_CHARS: usize = 700;

const CLI_SETUP_MARKER: &str = "Setting up Claude Code";

pub fn install_failed(error: impl ToString) -> CommandError {
    CommandError::new("install_failed", error)
}

fn failure(status: ExitStatus, detail: &str) -> CommandError {
    let start =
        detail.char_indices().rev().nth(FAILURE_TAIL_CHARS - 1).map_or(0, |(index, _)| index);
    install_failed(format!("{status}: {}", detail[start..].trim()))
}

fn run_with_lines(cmd: &mut Command, mut on_line: impl FnMut(&str)) -> Result<(), CommandError> {
    let mut child = hidden(cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(install_failed)?;
    let stdout = child.stdout.take().expect("stdout is piped");
    let mut output = String::new();
    for line in BufReader::new(stdout).split(b'\n') {
        let line = line.map_err(install_failed)?;
        let line = String::from_utf8_lossy(&line);
        on_line(&line);
        output.push_str(&line);
        output.push('\n');
    }
    let status = child.wait().map_err(install_failed)?;
    if status.success() { Ok(()) } else { Err(failure(status, &output)) }
}

#[cfg(windows)]
fn curl() -> Command {
    let mut cmd = Command::new(crate::system::system32("curl.exe"));
    cmd.args(["-q", "--ssl-revoke-best-effort"]);
    cmd
}

#[cfg(unix)]
fn curl() -> Command {
    let mut cmd = Command::new("/usr/bin/curl");
    cmd.arg("-q");
    cmd
}

fn download(proxy: &str, source: &str, path: &Path) -> Result<(), CommandError> {
    let output = hidden(curl().args([
        "--fail",
        "--location",
        "--silent",
        "--show-error",
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--connect-timeout",
        "15",
        "--max-time",
        "300",
        "--retry",
        "2",
        "--proxy",
        proxy,
        "--noproxy",
        "",
        "--output",
    ]))
    .arg(path)
    .arg(source)
    .output()
    .map_err(install_failed)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(failure(output.status, &String::from_utf8_lossy(&output.stderr)))
    }
}

#[cfg(unix)]
fn official_script() -> (&'static str, Command) {
    let mut cmd = Command::new("/bin/bash");
    cmd.args(["-c", r#"exec /bin/bash "$1" 2>&1"#, "bash", "install.sh"]);
    ("install.sh", cmd)
}

#[cfg(windows)]
fn official_script() -> (&'static str, Command) {
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new(crate::system::system32("cmd.exe"));
    cmd.raw_arg(r#"/d /c "install.cmd 2>&1""#);
    ("install.cmd", cmd)
}

pub fn install_cli(proxy: &str, report: impl Fn(&'static str)) -> Result<(), CommandError> {
    let temp = tempfile::Builder::new().prefix("ClawdNinja-").tempdir().map_err(install_failed)?;
    let (name, mut script) = official_script();
    report("script");
    download(proxy, &format!("https://claude.ai/{name}"), &temp.path().join(name))?;
    report("download");
    let mut setting_up = false;
    run_with_lines(script.current_dir(temp.path()).envs(variables(proxy)), |line| {
        if !setting_up && line.contains(CLI_SETUP_MARKER) {
            setting_up = true;
            report("setup");
        }
    })
}
