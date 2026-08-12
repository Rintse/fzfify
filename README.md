# fzfify

Turns a rhai script outputting a description of fzf arguments into an fzf-based
TUI, if you're masochistic enough to figure out rhai. Meant mostly as a cure
for those noun-verb CLIs where you end up copying lots of output into new
commands.

## Installation

If `$HOME/.cargo/bin` is in your `$PATH`:

```
cargo install --path .
```


## Usage

```
fzfify <RHAI_SCRIPT> [SCRIPT_ARGS]...
```

## Example scripts

See [`examples/`](examples/).

## The script

The script is a rhai file containing an array of actions at the top level,
where actions have the following fields:

- **`input_cmd`**: The command to pipe into to the fzf view.
- **`preview`**: Passed as `--preview` to fzf.
- **`header_lines`**: Passed to fzf with `--header`.
- **`binds`**: An array of objects that describe keybindings in the fzf view.
  Must have the following keys:
    - **`description`**
    - **`key`**
    - **`event`**
- **`show_binds`**: If true, additional header lines are added for each
  binding, showing the key and description fields.
- **`extra_fzf_args`**: Passed to fzf as-is.

## Argument templating

All of:
- `input_cmd` 
- `header_lines` 
- `preview`
- `binds.event` 
- `binds.description`
- `extra_fzf_args`

support the `{{n}}` syntax to be able to reference the n-th `SCRIPT_ARG`. This
enables previous selections to be passed as extra arguments which can be used
in various ways. The 0-th `SCRIPT_ARG` (`{{0}}`) is set to the invocation of
the fzfify program, minus the `SCRIPT_ARGS`. The empty `{{}}` will be
substituted for the entire evocation (so with `SCRIPT_ARGS`).
