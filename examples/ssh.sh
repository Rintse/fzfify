#!/bin/env bash

SSH_DIR="$HOME/.ssh"
PAHTPICKER="vifm --choose-dir -"

set_keybind() {
    echo -ne "\x1ebind\x1f${1}\x1f${2}\x1f${3}"
}
edit_ssh_cfg() {
    rg -l "Host $1" ~/.ssh | xargs "$EDITOR" "+/Host $1"
}
edit_ssh_cmd() {
    cmd=$(echo "ssh $1" | vipe | tee /dev/tty)
    eval "$cmd"
}
show_host_cfg() {
    shopt -s nullglob
    sed -n "/^Host $1\($\| \)/,/^Host \|^$/p" \
        "$SSH_DIR/config" "$SSH_DIR"/config.d/* |
        sed '$d'
}
copy_from() {
    if [[ -z "$2" ]]; then
        # Pick file on remote
        echo -ne "\x1eshow_binds\x1ftrue"
        echo -ne "\x1epreview\x1fssh {{2}} stat {}"
        set_keybind "Copy file to local host" \
            "enter" \
            "become({{*}} {})"
        echo -ne "\x1d"

        ssh "$1" "find -type f"
    else
        # Pick path on local host and copy
        save_path=$($PAHTPICKER)
        scp -r "$1:$2" "$save_path/$(basename "$2")"
    fi
}
show_hosts() {
    echo -ne "\x1eshow_binds\x1ftrue"
    echo -ne "\x1epreview\x1f{{*}} preview_host {}"
    set_keybind "Edit SSH host config" \
        "ctrl-e" \
        "execute({{*}} edit_host {})"
    set_keybind "Output host to stdout" \
        "ctrl-p" \
        "accept"
    set_keybind "SSH into host" \
        "enter" \
        "become(ssh {1})"
    set_keybind "Copy files" \
        "ctrl-f" \
        "become({{}} copy_from {})"
    set_keybind "SSH into host after editing ssh command" \
        "ctrl-space" \
        "become({{*}} edit_cmd {})"
    echo -ne "\x1d"

    grep -r "Host " ~/.ssh | grep -v '*' | cut -d " " -f 2
}

if [[ -z "$1" ]]; then
    show_hosts
    exit 0
fi

case "$1" in
preview_host)
    show_host_cfg "$2"
    ;;
edit_host)
    edit_ssh_cfg "$2"
    ;;
edit_cmd)
    edit_ssh_cmd "$2"
    ;;
copy_from)
    shift
    copy_from "$@"
    ;;
*)
    echo "wat"
    ;;
esac
