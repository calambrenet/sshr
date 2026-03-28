# sshr

[![CI](https://github.com/calambrenet/sshr/actions/workflows/ci.yml/badge.svg)](https://github.com/calambrenet/sshr/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/sshr.svg)](https://crates.io/crates/sshr)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/calambrenet/sshr#license)

**SSH Reimagined.** A modern SSH connection manager written in Rust.

`sshr` reads your existing `~/.ssh/config` and extends it with fuzzy search, key auditing, tunnel management, and more — without replacing your workflow.

> **Status:** Early development. Core CLI structure is in place; commands are being implemented incrementally.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Building from Source](#building-from-source)
- [Running Tests](#running-tests)
- [Contributing](#contributing)
- [Roadmap](#roadmap)
- [Author](#author)
- [License](#license)

## Features

- **Drop-in SSH compatibility** — `sshr host` works just like `ssh host`, no subcommand required
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

### From crates.io

```bash
cargo install sshr
```

### From source

```bash
git clone https://github.com/calambrenet/sshr.git
cd sshr
cargo install --path .
```

### Requirements

- Rust 1.93+ (edition 2024)
- OpenSSH client installed on your system

## Usage

### Connecting to hosts

`sshr` is compatible with the SSH invocation style — just pass the host directly:

```bash
# These are equivalent:
sshr myserver
sshr connect myserver
ssh myserver

# With user and flags:
sshr root@192.168.1.100
sshr -p 2222 myserver
sshr myserver -- -L 8080:localhost:80
```

The explicit `connect` subcommand is still available and supports aliases:

```bash
sshr connect myserver
sshr c myserver -p 2222 -u root
```

> **Note:** If a host has the same name as a subcommand (e.g., `Host list` in your config), use `sshr connect list` explicitly.

### Managing hosts

```bash
# List all configured SSH hosts
sshr list
sshr ls                        # alias

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
```

### Key and tunnel management

```bash
# Manage SSH keys
sshr keys list
sshr keys audit
sshr keys generate mykey

# Manage tunnels
sshr tunnel add myserver -l 8080 -r localhost:80 --background
sshr tunnel list
sshr tunnel stop myserver
```

### Other commands

```bash
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

## Building from Source

```bash
git clone https://github.com/calambrenet/sshr.git
cd sshr
cargo build --release
```

The binary will be at `target/release/sshr`.

## Running Tests

```bash
cargo test                     # All tests (unit + integration)
cargo clippy                   # Lint
cargo fmt --check              # Check formatting
```

## Contributing

Contributions are welcome! Whether it's bug reports, feature requests, documentation improvements, or code — all help is appreciated.

> **Important:** AI-generated code (LLMs, Copilot, etc.) is not accepted.
> We value human-written contributions. See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on:

- How to set up your development environment
- The pull request process
- Code style guidelines

### Quick start for contributors

```bash
# Fork and clone
git clone https://github.com/<your-username>/sshr.git
cd sshr

# Create a branch
git checkout -b my-feature

# Make changes, then verify
cargo fmt --check
cargo clippy -- -D warnings
cargo test

# Push and open a pull request
```

## Roadmap

This project is in active development. Here's what's implemented and what's planned:

- [x] CLI structure with all subcommands
- [x] SSH config parsing (`~/.ssh/config`)
- [x] `connect` command with SSH process delegation
- [x] Implicit connect (`sshr host` without subcommand)
- [x] Shell completions generation
- [ ] `list` — Host listing with filtering and sorting
- [ ] `search` — Fuzzy search across hosts
- [ ] `add` / `remove` — Config file modification
- [ ] `show` — Detailed host information
- [ ] `lint` — Config validation and error detection
- [ ] `keys` — Key listing, auditing, and generation
- [ ] `tunnel` — Tunnel creation and management
- [ ] `history` — Connection history and auditing
- [ ] `transfer` — SCP/SFTP file transfer
- [ ] `status` — Active connection monitoring
- [ ] `Include` directive support in config parser

## Author

**Jose Luis Castro Sola** ([@calambrenet](https://github.com/calambrenet))

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution licensing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
