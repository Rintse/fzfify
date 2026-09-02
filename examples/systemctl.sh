#!/bin/env bash

SCRIPT_DIR="$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")"
source "$SCRIPT_DIR/util.sh"

show_deps() {
    sudo systemctl list-dependencies "$1" --plain \
        | tail -n +2 \
        | awk '{$1=$1;print}'
}

show_services() {
    set_show_binds
    set_preview "systemctl status {1}"
    add_fzf_arg "--no-mouse"
    set_keybind "Start service" \
        "ctrl-space" \
        "execute(sudo systemctl start {1})"
    set_keybind "Stop service" \
        "ctrl-s" \
        "execute(sudo systemctl stop {1})"
    set_keybind "Restart service" \
        "ctrl-r" \
        "execute(sudo systemctl restart {1})"
    set_keybind "Edit service" \
        "ctrl-e" \
        "execute(sudo systemctl edit {1})"
    set_keybind "Show dependencies" \
        "ctrl-d" \
        "execute({{*}} show_deps)"
    options_end

    systemctl list-units --type=service --all --no-legend --plain
}


case "$1" in
"")
    show_services ;;
show_deps)
    show_deps "$2" ;;
*)
    echo "wat" ;;
esac
