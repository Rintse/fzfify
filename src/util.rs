use std::{path::Path, process::Stdio};

use anyhow::Context;
use log::debug;

/// Reconstructs the invocation of this program
pub fn get_call_args(script_args: &[String]) -> String {
    let mut args: Vec<String> = std::env::args().collect();
    let keep = args.len().saturating_sub(script_args.len());
    args.truncate(keep);

    if let Some(a) = args.last()
        && a == "--"
    {
        args.pop();
    }

    args.join(" ")
}

/// Required to pass a file in the pwd as a single argument without `./`
fn resolve_if_cwd_file(cmd: &[String]) -> Vec<String> {
    let mut cmd = cmd.to_vec();

    if let Some(f) = cmd.first_mut()
        && !f.contains('/')
        && Path::new(f).is_file()
    {
        *f = format!("./{f}");
    }

    cmd
}

/// Launches input command and returns a handle to its stdout
pub fn start_script(
    cmd: &[String],
) -> anyhow::Result<std::process::ChildStdout> {
    debug!("Launching input cmd: {cmd:?}");
    let shell =
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let cmd = resolve_if_cwd_file(cmd).join(" ");

    let mut p = std::process::Command::new(shell)
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .spawn()?;

    p.stdout.take().context("Failed to take stdout of input command")
}
