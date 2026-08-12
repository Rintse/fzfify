use crate::util::{get_call_args, input_stdout};
use anyhow::{Context, anyhow};
use log::debug;
use regex::Regex;
use serde::Deserialize;
use std::{collections::HashMap, process::Stdio};

/// An fzf binding that also has a description to explain the binding
#[derive(Debug, Deserialize, Clone)]
pub struct Bind {
    pub key: String,
    pub event: String,
    pub description: Option<String>,
}

/// Specifies a the behaviour for an fzf view given `match_args` are provided
#[derive(Debug, Deserialize, Clone)]
pub struct Action {
    /// The arguments to match for this action (regex)
    #[serde(default)]
    pub match_args: Vec<String>,
    /// The command to run as input to the fzf window
    pub input_cmd: String,
    /// Shows the keybinds for which a description is set if true
    #[serde(default)]
    pub show_binds: bool,
    // fzf arguments
    pub preview: Option<String>,
    #[serde(default)]
    pub header_lines: Vec<String>,
    #[serde(default)]
    pub binds: Vec<Bind>,
    #[serde(default)]
    pub extra_fzf_args: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct Descriptor {
    pub actions: Vec<Action>,
    #[serde(default)]
    pub variables: HashMap<String, String>,
}

pub enum SubKey<'a> {
    /// The string `{{}}` is replaced by the evocation of fzfify
    All,
    /// The string `{{n}}` is replaced by the `n`-th match argument
    MatchArg(usize),
    /// The string `{{key}}` is replaced by the value in the top-level
    /// `variables` section of the descriptor
    Var(&'a str),
}

pub struct Substitutor<'a> {
    this: &'a str,
    args: &'a [String],
    vars: &'a HashMap<String, String>,
    all_arg_re: Regex,
    match_arg_re: Regex,
    var_re: Regex,
}

impl<'a> Substitutor<'a> {
    fn new(
        this: &'a str,
        args: &'a [String],
        vars: &'a HashMap<String, String>,
    ) -> Self {
        let all_arg_re = Regex::new(r"\{\{\}\}").unwrap();
        let match_arg_re = Regex::new(r"\{\{(\d+)\}\}").unwrap();
        let var_re = Regex::new(r"\{\{([a-zA-Z]\w*)\}\}").unwrap();
        Self { this, args, vars, all_arg_re, match_arg_re, var_re }
    }

    fn get_replacement(&self, key: &SubKey<'_>) -> anyhow::Result<String> {
        match key {
            SubKey::All => {
                Ok(std::env::args().collect::<Vec<String>>().join(" "))
            }
            SubKey::MatchArg(idx) => {
                if *idx == 0 {
                    Ok(self.this.to_string())
                } else {
                    self.args
                        .get(idx - 1)
                        .cloned()
                        .context(format!("Invalid match argument index: {idx}"))
                }
            }
            SubKey::Var(name) => self
                .vars
                .get(*name)
                .cloned()
                .context(format!("Undefined variable: {name}")),
        }
    }

    /// Finds all substrings to be substituted and performs the matching sub
    fn do_sub(&self, s: &str) -> anyhow::Result<String> {
        // Dont care about the cost of repeated cloning here
        let mut replaced = s.to_string();
        // TODO: lots of duplication below?

        while let Some(caps) = self.all_arg_re.captures(&replaced) {
            let m = caps.get(0).unwrap();
            let replacement = self.get_replacement(&SubKey::All)?;
            debug!(
                "Substituting evocation: {} <- {}",
                &replaced[m.start()..m.end()],
                &replacement
            );
            replaced.replace_range(m.start()..m.end(), &replacement);
        }

        while let Some(caps) = self.match_arg_re.captures(&replaced) {
            let m = caps.get(0).unwrap();
            let replacement = {
                let idx = caps[1].parse().expect("Regex allows only ints");
                self.get_replacement(&SubKey::MatchArg(idx))?
            };
            debug!(
                "Substituting match argument: {} <- {}",
                &replaced[m.start()..m.end()],
                &replacement
            );
            replaced.replace_range(m.start()..m.end(), &replacement);
        }

        while let Some(caps) = self.var_re.captures(&replaced) {
            let m = caps.get(0).unwrap();
            let replacement = self.get_replacement(&SubKey::Var(&caps[1]))?;
            debug!(
                "Substituting variable: {} <- {}",
                &replaced[m.start()..m.end()],
                &replacement
            );
            replaced.replace_range(m.start()..m.end(), &replacement);
        }

        Ok(replaced)
    }
}

impl Action {
    /// Substitues in all the runtime information for this action
    pub fn with_args_vars(
        &self,
        args: &[String],
        vars: &HashMap<String, String>,
    ) -> anyhow::Result<Self> {
        let this = get_call_args(args);
        let subber = Substitutor::new(&this, args, vars);

        let match_args = self.match_args.clone();
        let input_cmd =
            subber.do_sub(&self.input_cmd).context("Error in input command")?;
        let show_binds = self.show_binds;

        let preview = self.preview.clone().map(|p| subber.do_sub(&p));
        let preview = preview.transpose().context("Error in preview")?;

        let header_lines = self
            .header_lines
            .iter()
            .map(|l| {
                subber.do_sub(l).context(format!("Error in header: `{l}`"))
            })
            .collect::<Result<_, _>>()?;

        let extra_fzf_args = self
            .extra_fzf_args
            .iter()
            .map(|l| {
                subber
                    .do_sub(l)
                    .context(format!("Error in extra_fzf_args: `{l}`"))
            })
            .collect::<Result<_, _>>()?;

        let binds: Vec<_> = self
            .binds
            .iter()
            .map(|Bind { key, event, description }| {
                let key = key.clone();
                let event = subber
                    .do_sub(event)
                    .context(format!("Error in bind event for `{key}`"))?;
                let description = description
                    .as_ref()
                    .map(|d| subber.do_sub(d))
                    .transpose()
                    .context(format!("Error in description for `{key}`"))?;
                Ok(Bind { key, event, description })
            })
            .collect::<anyhow::Result<_>>()?;

        Ok(Self {
            match_args,
            input_cmd,
            show_binds,
            preview,
            header_lines,
            binds,
            extra_fzf_args,
        })
    }

    fn run(&self) -> anyhow::Result<()> {
        let mut fzf_args: Vec<String> = vec![];

        let mut header_lines = self.header_lines.clone();
        if self.show_binds {
            for Bind { key, description, .. } in &self.binds {
                if let Some(desc) = description {
                    header_lines.push(format!("[{key}] {desc}"));
                }
            }
        }

        if !header_lines.is_empty() {
            fzf_args.push("--header".to_string());
            fzf_args.push(header_lines.join("\n"));
        }

        if let Some(cmd) = &self.preview {
            fzf_args.push("--preview".to_string());
            fzf_args.push(cmd.clone());
        }

        for Bind { key, event, .. } in &self.binds {
            fzf_args.push("--bind".to_string());
            fzf_args.push(format!("{key}:{event}"));
        }

        fzf_args.extend_from_slice(&self.extra_fzf_args);
        debug!("Passing to fzf:\n{}", fzf_args.join("\n"));

        let mut fzf = std::process::Command::new("fzf")
            .args(fzf_args)
            .stdin(Stdio::from(input_stdout(&self.input_cmd)?))
            .spawn()?;

        fzf.wait()?;
        Ok(())
    }
}

impl Descriptor {
    pub fn run(&self, args: &[String]) -> anyhow::Result<()> {
        let action: Option<&Action> = 'action_find: {
            'action_for: for action in &self.actions {
                if action.match_args.len() != args.len() {
                    continue;
                }

                for (sa, ma) in args.iter().zip(&action.match_args) {
                    let ma_re = Regex::new(ma)
                        .context(format!("Invalid regex: {ma}"))?;
                    if !ma_re.is_match(sa) {
                        continue 'action_for;
                    }
                }
                break 'action_find Some(action);
            }

            None
        };

        match action {
            Some(action) => {
                let action = action.with_args_vars(args, &self.variables)?;
                action.run()
            }
            None => Err(anyhow!("No actions match provided args {args:?}")),
        }
    }
}
