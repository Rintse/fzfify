use crate::util::{get_call_args, input_stdout};
use anyhow::Context;
use log::debug;
use regex::Regex;
use serde::Deserialize;
use std::process::Stdio;

/// An fzf binding that also has a description to explain the binding
#[derive(Debug, Deserialize, Clone)]
pub struct Bind {
    pub key: String,
    pub event: String,
    pub description: Option<String>,
}

/// Specifies a the behaviour for an fzf view
#[derive(Debug, Deserialize, Clone)]
pub struct Action {
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

pub struct Substitutor<'a> {
    this: &'a str,
    args: &'a [String],
    script_arg_re: Regex,
}

impl<'a> Substitutor<'a> {
    fn new(this: &'a str, args: &'a [String]) -> Self {
        let script_arg_re = Regex::new(r"\{\{(\d*)\}\}").unwrap();
        Self { this, args, script_arg_re }
    }

    /// The entire evocation
    fn evocation() -> String {
        std::env::args().collect::<Vec<String>>().join(" ")
    }

    fn get_arg(&self, n: usize) -> anyhow::Result<String> {
        if n == 0 {
            // Just the fzfify evocation (no script_args)
            Ok(self.this.to_owned())
        } else {
            // The `idx`-th script arg
            self.args
                .get(n - 1)
                .cloned()
                .context(format!("Invalid script argument index: {n}"))
        }
    }

    /// Finds all substrings to be substituted and performs the matching sub
    fn do_sub(&self, s: &str) -> anyhow::Result<String> {
        let mut replaced = s.to_string();

        while let Some(caps) = self.script_arg_re.captures(&replaced) {
            let m = caps.get(0).unwrap();
            let replacement = {
                if caps[1].is_empty() {
                    Self::evocation()
                } else {
                    let idx: usize =
                        caps[1].parse().expect("Regex allows only ints");
                    self.get_arg(idx)?
                }
            };
            debug!(
                "Substituting script argument: {} <- {}",
                &replaced[m.start()..m.end()],
                &replacement
            );
            // Dont care about the cost of repeated cloning here
            replaced.replace_range(m.start()..m.end(), &replacement);
        }

        Ok(replaced)
    }
}

impl Action {
    /// Substitues in all the runtime information for this action
    pub fn with_args(&self, args: &[String]) -> anyhow::Result<Self> {
        let this = get_call_args(args);
        let subber = Substitutor::new(&this, args);

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
            input_cmd,
            show_binds,
            preview,
            header_lines,
            binds,
            extra_fzf_args,
        })
    }

    pub fn run(&self) -> anyhow::Result<()> {
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
