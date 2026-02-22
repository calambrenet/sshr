# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- CLI structure with clap derive: global options, subcommands, and aliases
- Subcommands: list, connect, add, remove, search, show, lint, keys, trust, tunnel, history, transfer, status, completions
- Shell completion generation (bash, zsh, fish, PowerShell, elvish)
- Multiple output formats: text, JSON, CSV
- Integration tests with assert_cmd
