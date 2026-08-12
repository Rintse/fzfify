fn SSH_DIR() {
    env("HOME") + "/.ssh"
}

fn show_hosts() {
    const SSH_DIR = SSH_DIR();

    let input_cmd = `grep -r "Host " ~/.ssh | grep -v "*" | cut -d " " -f 2`;

    let preview = `
    shopt -s nullglob; \
    sed -n '/^Host {1}\\($\\| \\)/,/^Host \\|^$/p' \
        ${SSH_DIR}/config \
        ${SSH_DIR}/config.d/* \
        | sed '$d' \
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

fn scp() {
    let scp_cmd = `become(
        save_path=$(vifm --choose-dir -);
        scp -r "{{2}}":{} "$save_path/$(basename {})"
    )`;

    let binds = [
        #{
            description: "Copy to this host",
            key: "enter",
            event: scp_cmd,
        },
    ];

    #{
        input_cmd: `ssh {{2}} "find -type f"`,
        preview: `ssh {{2}} "stat {}"`,
        show_binds: true,
        binds: binds,
    }
}

if ARGS.len() == 0 {
    show_hosts()
}
else {
    switch ARGS[0] {
        "cp" => scp(),
        _ => print("Invalid arguments")
    }
}

