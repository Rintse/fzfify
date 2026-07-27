use std::process::Stdio;

use anyhow::Context;
use log::debug;
use regex::Regex;

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

/// Finds `{{k}}` in `line` and replaces it with the k-th `SCRIPT_ARG`, with the
/// 0-th argument being the program itself, from `get_call_args()`
pub fn arg_replace(
    this: &str,
    args: &[String],
    line: &str,
) -> anyhow::Result<String> {
    let match_arg_re = Regex::new(r"\{\{(\d+)\}\}").unwrap();
    let mut replaced = line.to_string();

    while let Some(caps) = match_arg_re.captures(&replaced) {
        let m = caps.get(0).unwrap();
        let idx: usize = caps[1].parse().context("Invalid match argument")?;
        let arg = match idx {
            0 => &this.to_string(),
            n => args
                .get(n - 1)
                .context(format!("Invalid match argument index: {idx}"))?,
        };
        // Dont care about the cost of repeated cloning here
        replaced.replace_range(m.start()..m.end(), arg);
    }

    Ok(replaced)
}

/// Launches input command and returns a handle to its stdout
pub fn input_stdout(cmd: &str) -> anyhow::Result<std::process::ChildStdout> {
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
