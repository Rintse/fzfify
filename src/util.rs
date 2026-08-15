use std::process::Stdio;

use anyhow::Context;
use log::debug;

/// Reconstructs the invocation of this program, excluding `script_args`
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

/// Launches input command and returns a handle to its stdout
pub fn start_script(cmd: &str) -> anyhow::Result<std::process::ChildStdout> {
    debug!("Launching input cmd: {cmd}");
    let shell =
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let mut cmd = std::process::Command::new(shell)
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .spawn()?;
    cmd.stdout.take().context("Failed to take stdout of input command")
}
