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
    /// Enable verbose mode
    #[arg(short, long)]
    verbose: bool,
    /// The rhai script specifying fzf behaviour
    script: PathBuf,
    /// The arguments passed as `match_args` in the toml descriptor
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    script_args: Vec<String>,
}

impl CliArgs {
    fn get_level(&self) -> LevelFilter {
        if self.verbose { LevelFilter::Debug } else { LevelFilter::Info }
    }
}

fn run_script(script: &str, script_args: &[String]) -> anyhow::Result<Action> {
    let rhai_args: rhai::Array =
        script_args.iter().cloned().map(Dynamic::from).collect();
    let mut scope = Scope::new();
    scope.push_constant("ARGS", rhai_args);

    let mut engine = Engine::new();
    engine.register_fn("env", |name: &str| -> Dynamic {
        match std::env::var(name) {
            Ok(v) => v.into(),
            Err(_) => Dynamic::UNIT,
        }
    });

    let result: Dynamic = engine
        .eval_with_scope(&mut scope, script)
        .context("Error evaluating script")?;

    let action: Action = rhai::serde::from_dynamic(&result)
        .context("Script did not return action")?;

    Ok(action)
}

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    env_logger::Builder::new().filter_level(args.get_level()).init();

    let script = fs::read_to_string(&args.script)?;
    let action = run_script(&script, &args.script_args)?;
    let action = action.with_args_vars(&args.script_args, &HashMap::new())?;

    action.run()
}
