use crate::util::get_call_args;
use anyhow::{Context, bail};
use log::debug;
use regex::Regex;
use std::{io::BufRead, process::Stdio};

/// This goes between the action and the actual data
const ACTION_SEPARATOR: u8 = 0x1d; // ascii record separator
/// This goes between each of `show_binds`, `preview`, etc.
const OPTION_SEPARATOR: u8 = 0x1e; // ascii record separator
/// This goes between the option name and each of its parameters
const PARAM_SEPARATOR: u8 = 0x1f; // ascii unit separator

pub fn split_action(
    mut reader: impl BufRead,
) -> anyhow::Result<(Action, impl BufRead)> {
    let mut options_buf = Vec::new();
    let _ = reader.read_until(ACTION_SEPARATOR, &mut options_buf);

    if options_buf.last() != Some(&ACTION_SEPARATOR) {
        bail!("Did not find an action separator '{ACTION_SEPARATOR:#04x}'")
    }

    let action = Action::from_option_data(&options_buf)?;
    Ok((action, reader))
}

/// Specifies a the behaviour for an fzf view
#[derive(Default, Debug, Clone)]
pub struct Action {
    pub show_binds: bool,
    pub preview: Option<String>,
    pub header_lines: Vec<String>,
    pub binds: Vec<Bind>,
    pub extra_fzf_args: Vec<String>,
}

impl Action {
    pub fn from_option_data(data: &[u8]) -> anyhow::Result<Self> {
        let mut preview: Option<String> = None;
        let mut show_binds = false;
        let mut binds: Vec<Bind> = Vec::new();
        let mut header_lines: Vec<String> = Vec::new();
        let mut fzf_args: Vec<String> = Vec::new();

        let options: Vec<String> = data
            .split(|x| *x == OPTION_SEPARATOR)
            .map(|x| String::from_utf8_lossy(x).to_string())
            .collect();

        for opt in options {
            let mut s = opt.split(PARAM_SEPARATOR as char);

            if let Some(opt_name) = s.next() {
                match opt_name {
                    "show_binds" => match s.next() {
                        None => bail!("{opt_name} with no value"),
                        Some("true") => show_binds = true,
                        Some(_) => (),
                    },
                    "bind" => {
                        let bind = Bind::from_opt(&opt)
                            .context("Error parsing keybind")?;
                        binds.push(bind);
                    }
                    "preview" => match s.next() {
                        Some(cmd) => preview = Some(cmd.to_string()),
                        None => bail!("{opt_name} with no value"),
                    },
                    "header" => match s.next() {
                        Some(s) => header_lines.push(s.to_string()),
                        None => bail!("{opt_name} with no value"),
                    },
                    "fzf_arg" => match s.next() {
                        Some(s) => fzf_args.push(s.to_string()),
                        None => bail!("{opt_name} with no value"),
                    },
                    "" => (), // this allows starting with OPTION_SEPARATOR
                    _ => bail!("Unknown option: {opt_name}"),
                }
            }
        }

        Ok(Self {
            show_binds,
            preview,
            header_lines,
            binds,
            extra_fzf_args: fzf_args,
        })
    }

    /// Substitues in all the runtime information for this action
    pub fn with_args(&self, args: &[String]) -> anyhow::Result<Self> {
        let this = get_call_args(args);
        let subber = Substitutor::new(&this, args);

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

        Ok(Self { show_binds, preview, header_lines, binds, extra_fzf_args })
    }

    fn fzf_args(&self) -> Vec<String> {
        let mut args: Vec<String> = vec![];
        let mut header_lines = self.header_lines.clone();

        if self.show_binds {
            for Bind { key, description, .. } in &self.binds {
                if let Some(desc) = description {
                    header_lines.push(format!("[{key}] {desc}"));
                }
            }
        }

        if !header_lines.is_empty() {
            args.push("--header".to_string());
            args.push(header_lines.join("\n"));
        }

        if let Some(cmd) = &self.preview {
            args.push("--preview".to_string());
            args.push(cmd.clone());
        }

        for Bind { key, event, .. } in &self.binds {
            args.push("--bind".to_string());
            args.push(format!("{key}:{event}"));
        }

        args.extend_from_slice(&self.extra_fzf_args);
        args
    }

    pub fn run(&self, mut reader: impl BufRead) -> anyhow::Result<()> {
        let fzf_args = self.fzf_args();
        debug!("Passing to fzf:\n{}", fzf_args.join("\n"));

        let mut fzf = std::process::Command::new("fzf")
            .args(fzf_args)
            .stdin(Stdio::piped())
            .spawn()?;
        let mut stdin =
            fzf.stdin.take().context("Failed to take fzf's stdin")?;

        // Pipe the rest of the reader into stdin
        let _ = std::io::copy(&mut reader, &mut stdin);
        drop(stdin);
        fzf.wait()?;

        Ok(())
    }
}

/// An fzf binding that also has a description to explain the binding
#[derive(Debug, Clone)]
pub struct Bind {
    pub key: String,
    pub event: String,
    pub description: Option<String>,
}

impl Bind {
    fn from_opt(data: &str) -> anyhow::Result<Self> {
        let mut s = data.split(PARAM_SEPARATOR as char).skip(1);

        let description = s.next().context("Missing description")?.to_string();
        let key = s.next().context("Missing key")?.to_string();
        let event = s.next().context("Missing event")?.to_string();

        let description =
            if description.is_empty() { None } else { Some(description) };
        Ok(Self { key, event, description })
    }
}

pub struct Substitutor<'a> {
    this: &'a str,
    args: &'a [String],
    script_arg_re: Regex,
}

/// Utility struct that performs transformations based on evocation details
impl<'a> Substitutor<'a> {
    fn new(this: &'a str, args: &'a [String]) -> Self {
        let script_arg_re = Regex::new(r"\{\{(\d*)\}\}").unwrap();
        Self { this, args, script_arg_re }
    }

    /// The entire evocation of this program
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
