#![warn(clippy::pedantic)]

mod descriptor;
mod util;

use crate::descriptor::Action;
use anyhow::Context;
use clap::Parser;
use log::LevelFilter;
use rhai::{Dynamic, Engine, Scope};
use std::{collections::HashMap, fs, path::PathBuf};

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

    let mut engine = Engine::new();
    engine.register_fn("env", |name: &str| -> Dynamic {
        match std::env::var(name) {
            Ok(v) => v.into(),
            Err(_) => Dynamic::UNIT,
        }
    });

    let rhai_args: rhai::Array =
        args.script_args.iter().cloned().map(Dynamic::from).collect();
    let mut scope = Scope::new();
    scope.push_constant("ARGS", rhai_args);

    let script = fs::read_to_string(&args.descriptor)?;
    let result: Dynamic = engine
        .eval_with_scope(&mut scope, &script)
        .context("Error evaluating script")?;

    let action: Action = rhai::serde::from_dynamic(&result)
        .context("Script did not return action")?;
    let action = action.with_args_vars(&args.script_args, &HashMap::new())?;
    action.run()

    // let toml_str = fs::read_to_string(&args.descriptor)?;
    // let cfg: Descriptor = toml::from_str(&toml_str)?;
    // cfg.run(&args.script_args)
}
