# Contributing to sshr

Thank you for your interest in contributing to sshr!

## Getting started

1. Fork the repository and clone your fork
2. Create a new branch: `git checkout -b my-feature`
3. Make your changes
4. Run the checks:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```
5. Commit your changes and push to your fork
6. Open a pull request

## Development setup

- Rust 1.93+ (edition 2024)
- Run `cargo build` to verify everything compiles

## Guidelines

- Run `cargo fmt` before committing
- Ensure `cargo clippy -- -D warnings` passes with no warnings
- Add tests for new functionality
- Keep pull requests focused — one feature or fix per PR
- Update documentation if your change affects user-facing behavior

## AI-generated code policy

**Do not use AI code generators (LLMs, Copilot, ChatGPT, etc.) for your contributions.**

This project is a learning space. We want to see *your* code — your thinking,
your mistakes, your growth. That's how we all learn. A pull request full of
AI-generated code teaches no one anything.

We review contributions carefully. PRs that appear to be AI-generated will be
closed without merge.

Write it yourself. Make mistakes. Ask questions. Learn with us.

## Reporting bugs

Open an issue at <https://github.com/calambrenet/sshr/issues> with:
- Steps to reproduce the bug
- Expected vs actual behavior
- Your OS, Rust version (`rustc --version`), and sshr version (`sshr --version`)

## License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license.
