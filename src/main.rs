#![warn(clippy::pedantic)]

mod descriptor;
mod util;

use crate::{descriptor::split_action, util::start_script};
use clap::Parser;
use log::{LevelFilter, debug};
use std::io::BufReader;

#[derive(Parser)]
#[command(about, long_about = None)]
struct CliArgs {
    /// Enable verbose mode
    #[arg(short, long)]
    verbose: bool,
    /// The script and its arguments that output options and fzf data
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    script: Vec<String>,
}

impl CliArgs {
    fn get_level(&self) -> LevelFilter {
        if self.verbose { LevelFilter::Debug } else { LevelFilter::Info }
    }
}

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    env_logger::Builder::new().filter_level(args.get_level()).init();

    let stdout = start_script(&args.script)?;
    let (action, reader) = split_action(BufReader::new(stdout))?;
    debug!("Parsed action:\n{action:#?}");

    let action = action.with_args(&args.script)?;
    action.run(reader)
}
