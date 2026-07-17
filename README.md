# fzfify

Turns a toml containing a description of fzf arguments into an fzf-based TUI,
if you're masochistic enough to figure out the toml string syntax. Mean mostly
as a cure for those noun-verb CLIs where you end up copying lots of output into
new commands.

## Installation

If `$HOME/.cargo/bin` is in your `$PATH`:

```
cargo install --path .
```


## Usage

```
fzfify <DESCRIPTOR_TOML> [SCRIPT_ARGS]...
```

## Example descriptors

See [`examples/`](examples/).

## The descriptor

The descriptor is a TOML file containing an array of actions at the top level,
where actions have the following fields:

- **`match_args`**: This is matched against `[SCRIPT_ARGS]` to select which
  action in the list must be run. Each item in the list is a regex. The first
  matching action is used.
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
- **`extra_fzf_args`**: Passed to fzf as-is with if the action is matched.

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
the fzfify program, minus the `SCRIPT_ARGS`.
