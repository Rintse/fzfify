#![warn(clippy::pedantic)]

mod descriptor;
mod util;

use crate::{descriptor::Action, util::start_script};
use anyhow::bail;
use clap::Parser;
use log::{LevelFilter, debug};
use std::io::{BufRead, BufReader};

#[derive(Parser)]
#[command(about, long_about = None)]
struct CliArgs {
    /// Enable verbose mode
    #[arg(short, long)]
    verbose: bool,
    /// The rhai script specifying fzf behaviour
    script: String,
    /// The arguments passed as `ARGS` to the script
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    script_args: Vec<String>,
}

impl CliArgs {
    fn get_level(&self) -> LevelFilter {
        if self.verbose { LevelFilter::Debug } else { LevelFilter::Info }
    }
}

fn parse_action(
    mut reader: impl BufRead,
) -> anyhow::Result<(Action, impl BufRead)> {
    let mut options_buf = Vec::new();
    let _ = reader.read_until(0x1e, &mut options_buf);

    if options_buf.last() != Some(&0x1e) {
        bail!("Did not find a data separator '0x1e'")
    }

    let action = Action::from_option_data(&options_buf)?;
    Ok((action, reader))
}

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    env_logger::Builder::new().filter_level(args.get_level()).init();

    let stdout = start_script(&args.script)?;
    let (action, reader) = parse_action(BufReader::new(stdout))?;
    debug!("Parsed action:\n{action:?}");

    let action = action.with_args(&args.script_args)?;
    action.run(reader)
}
