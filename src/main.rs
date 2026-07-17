#![warn(clippy::pedantic)]

mod descriptor;
mod util;

use crate::descriptor::Descriptor;
use clap::Parser;
use log::LevelFilter;
use std::{fs, path::PathBuf};

#[derive(Parser)]
#[command(about, long_about = None)]
struct CliArgs {
    /// The TOML specifying fzf behaviour
    descriptor: PathBuf,
    /// Enable verbose mode
    #[arg(short, long)]
    verbose: bool,
    /// The arguments passed as `match_args` in the toml descriptor
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    script_args: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let log_level =
        if args.verbose { LevelFilter::Debug } else { LevelFilter::Info };
    env_logger::Builder::new().filter_level(log_level).init();

    let toml_str = fs::read_to_string(&args.descriptor)?;
    let cfg: Descriptor = toml::from_str(&toml_str)?;
    cfg.run(&args.script_args)
}
