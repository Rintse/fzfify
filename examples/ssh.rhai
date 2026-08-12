const SSH_DIR = env("HOME") + "/.ssh";
const PAHTPICKER = "vifm --choose-dir -";

fn show_hosts() {
    let input_cmd = `grep -r "Host " ~/.ssh | grep -v "*" | cut -d " " -f 2`;
    let preview = `
    shopt -s nullglob; \
    sed -n '/^Host {1}\($\| \)/,/^Host \|^$/p' \
        ${global::SSH_DIR}/config \
        ${global::SSH_DIR}/config.d/* \
        | sed '$d'
    `;

    let ssh_cmd = "become(ssh {1})";
    let cp_cmd = "become({{0}} cp {})";
    let edit_config_cmd = `execute(
        rg -l 'Host {1}' ~/.ssh | xargs "$EDITOR" '+/Host {r1}'
    )`;
    let edit_cmd = "become(bash <(echo 'ssh {1}' | vipe | tee /dev/tty) )";

    let binds = [
        #{
            description: "Output host to stdout",
            key: "ctrl-p",
            event: "accept",
        },
        #{
            description: "SSH into host",
            key: "enter",
            event: ssh_cmd,
        },
        #{
            description: "Copy files",
            key: "ctrl-f",
            event: cp_cmd,
        },
        #{
            description: "Edit SSH host config",
            key: "ctrl-e",
            event: edit_config_cmd,
        },
        #{
            description: "SSH into host after editing ssh command",
            key: "ctrl-space",
            event: edit_cmd,
        },
    ];

    #{
        input_cmd: input_cmd,
        preview: preview,
        binds: binds,
        show_binds: true,
    }
}

fn scp(host) {
    let scp_cmd = `become(
        save_path=$(${global::PAHTPICKER});
        scp -r "${host}":{} "$save_path/$(basename {})"
    )`;

    let binds = [
        #{
            description: "Copy to this host",
            key: "enter",
            event: scp_cmd,
        },
    ];

    #{
        input_cmd: `ssh ${host} "find -type f" 2>/dev/null`,
        preview: `ssh ${host} "stat {}"`,
        show_binds: true,
        binds: binds,
    }
}

if ARGS.len() == 0 {
    show_hosts()
}
else {
    switch ARGS[0] {
        "cp" => scp(ARGS[1]),
        _ => throw "Invalid arguments: " + ARGS,
    }
}
