# sshr

[![CI](https://github.com/calambrenet/sshr/actions/workflows/ci.yml/badge.svg)](https://github.com/calambrenet/sshr/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/sshr.svg)](https://crates.io/crates/sshr)
[![License](https://img.shields.io/crates/l/sshr.svg)](https://github.com/calambrenet/sshr#license)

**SSH Reimagined.** A modern SSH connection manager written in Rust.

`sshr` reads your existing `~/.ssh/config` and extends it with fuzzy search, key auditing, tunnel management, and more — without replacing your workflow.

> **Status:** Early development. Core CLI structure is in place; commands are being implemented incrementally.

## Features

- **List & search** hosts from your SSH config with fuzzy matching
- **Connect** with overrides (port, user, verbose, persistent reconnect)
- **Add & remove** hosts directly from the command line
- **Key management** — list, audit, and generate SSH keys
- **Tunnel management** — create, list, and stop SSH port-forwarding tunnels
- **Config linting** — validate your SSH config for common errors
- **Known hosts management** — manage trust and fingerprint history
- **File transfer** — copy files via SCP/SFTP with progress and resume
- **Connection status** — view active SSH sessions and tunnels
- **Shell completions** — generate completions for bash, zsh, fish, and more
- **Multiple output formats** — text, JSON, and CSV

## Installation

### From source

```bash
cargo install sshr
```

### From git

```bash
git clone https://github.com/calambrenet/sshr.git
cd sshr
cargo install --path .
```

### Requirements

- Rust 1.93+ (edition 2024)
- OpenSSH client installed on your system

## Usage

```bash
# List all configured SSH hosts
sshr list
sshr ls                        # alias

# Connect to a host
sshr connect myserver
sshr c myserver -p 2222 -u root  # with overrides

# Fuzzy search across hosts
sshr search prod

# Add a new host
sshr add myserver 192.168.1.100 -u deploy -p 22 --tags web,production

# Remove a host
sshr rm myserver

# Show detailed host info
sshr show myserver

# Lint your SSH config
sshr lint --warnings

# Manage SSH keys
sshr keys list
sshr keys audit
sshr keys generate mykey

# Manage tunnels
sshr tunnel add myserver -l 8080 -r localhost:80 --background
sshr tunnel list
sshr tunnel stop myserver

# File transfer
sshr cp local.txt myserver:/tmp/ --progress

# View connection status
sshr status

# Generate shell completions
sshr completions bash > ~/.bash_completion.d/sshr
sshr completions zsh > ~/.zfunc/_sshr
sshr completions fish > ~/.config/fish/completions/sshr.fish
```

### Global options

```
-F, --config-file <PATH>   Path to SSH config file [default: ~/.ssh/config] [env: SSHR_CONFIG]
    --no-color             Disable colored output [env: NO_COLOR]
    --format <FORMAT>      Output format: text, json, csv [default: text]
```

## Building from source

```bash
git clone https://github.com/calambrenet/sshr.git
cd sshr
cargo build --release
```

The binary will be at `target/release/sshr`.

## Running tests

```bash
cargo test
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
