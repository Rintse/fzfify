#!/bin/env bash

SSH_DIR="$HOME/.ssh"
PAHTPICKER="vifm --choose-dir -"

set_keybind() {
    echo -ne "\x1ebind\x1f${1}\x1f${2}\x1f${3}"
}


edit_ssh_cfg() {
    rg -l 'Host {1}' ~/.ssh | xargs \"$EDITOR\" '+/Host {r1}'
}

show_host_cfg="\
shopt -s nullglob ; \
sed -n '/^Host {1}\($\| \)/,/^Host \|^$/p' \
$SSH_DIR/config $SSH_DIR/config.d/* \
| sed '\$d' \
"

show_hosts() {
    echo -ne "\x1eshow_binds\x1ftrue"
    echo -ne "\x1epreview\x1f$show_host_cfg"

    # set_keybind "Output host to stdout" \
    #     "ctrl-p" \
    #     "accept"

    # set_keybind "SSH into host" \
    #     "enter" \
    #     "become(ssh {1})"

    # set_keybind "Copy files" \
    #     "ctrl-f" \
    #     "become({{0}} cp {})"

    # edit_config_cmd="execute('eval $(declare -f edit_ssh_cfg)')"
    # set_keybind "Edit SSH host config" \
    #     "ctrl-e" \
    #     "$edit_config_cmd"

    # set_keybind "SSH into host after editing ssh command" \ 
    #     "ctrl-space" \
    #     "become(bash <(echo 'ssh {1}' | vipe | tee /dev/tty))"

    echo "printing sep" 1>&2
    echo -ne "\x1d"
    grep -r "Host " ~/.ssh | grep -v "*" | cut -d " " -f 2
}

if [[ -z "$1" ]]; then
    show_hosts
fi
