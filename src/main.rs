mod cli;
mod commands;
mod config;
mod utils;
use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use cli::{Cli, Command};

/// Global flags that consume a separate value.
const GLOBAL_FLAGS_WITH_VALUE: &[&str] = &["-F", "--config-file", "--format"];

/// Preprocesses argv to inject "connect" when invoked without a subcommand.
///
/// If the first positional argument (after the binary and global flags) is not
/// a known subcommand, it is interpreted as the target host and "connect" is
/// injected at position 1.
fn preprocess_args(args: &[String]) -> Vec<String> {
    // Get known subcommands from clap (names + aliases + "help")
    let cmd = Cli::command();
    let known: std::collections::HashSet<&str> = cmd
        .get_subcommands()
        .flat_map(|sub| std::iter::once(sub.get_name()).chain(sub.get_all_aliases()))
        .chain(std::iter::once("help"))
        .collect();

    let mut has_subcommand = false;
    let mut has_positional = false;
    let mut i = 1;

    while i < args.len() {
        let arg = &args[i];

        // "--" -> stop. Do NOT mark as positional (host must come before --)
        if arg == "--" {
            break;
        }

        if arg.starts_with('-') {
            if GLOBAL_FLAGS_WITH_VALUE.contains(&arg.as_str()) && i + 1 < args.len() {
                i += 2; // skip flag + value
            } else {
                i += 1;
            }
            continue;
        }

        // Arg posicional
        if known.contains(arg.as_str()) {
            has_subcommand = true;
        } else {
            has_positional = true;
        }
        break;
    }

    if !has_subcommand && has_positional {
        let mut result = Vec::with_capacity(args.len() + 1);
        result.push(args[0].clone());
        result.push("connect".to_string());
        result.extend(args[1..].iter().cloned());
        result
    } else {
        args.to_vec()
    }
}

fn main() -> Result<()> {
    let raw_args: Vec<String> = std::env::args().collect();
    let args = preprocess_args(&raw_args);
    let mut cli = Cli::try_parse_from(&args).unwrap_or_else(|e| e.exit());

    // Expand ~ in the config path (clap receives it as a literal string)
    let config_str = cli.config_file.to_string_lossy().to_string();
    cli.config_file = PathBuf::from(utils::expand_tilde(&config_str));

    // Configurar colores globalmente
    if cli.no_color || std::env::var("NO_COLOR").is_ok() {
        // Disable colors (to be implemented later)
    }
    match &cli.command {
        Command::List(args) => commands::list::execute(&cli, args),
        Command::Connect(args) => commands::connect::execute(&cli, args),
        Command::Add(args) => commands::add::execute(&cli, args),
        Command::Remove(args) => commands::remove::execute(&cli, args),
        Command::Search(args) => commands::search::execute(&cli, args),
        Command::Show(args) => commands::show::execute(&cli, args),
        Command::Lint(args) => commands::lint::execute(&cli, args),
        Command::Keys(args) => commands::keys::execute(&cli, args),
        Command::Trust(args) => commands::trust::execute(&cli, args),
        Command::Tunnel(args) => commands::tunnel::execute(&cli, args),
        Command::History(args) => commands::history::execute(&cli, args),
        Command::Transfer(args) => commands::transfer::execute(&cli, args),
        Command::Status(args) => commands::status::execute(&cli, args),
        Command::Completions(args) => {
            let mut cmd = <Cli as clap::CommandFactory>::command();
            clap_complete::generate(args.shell, &mut cmd, "sshr", &mut std::io::stdout());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    // --- Casos que inyectan "connect" ---

    #[test]
    fn test_inject_bare_host() {
        assert_eq!(preprocess_args(&a("sshr host")), a("sshr connect host"));
    }

    #[test]
    fn test_inject_user_at_host() {
        assert_eq!(
            preprocess_args(&a("sshr root@host")),
            a("sshr connect root@host")
        );
    }

    #[test]
    fn test_inject_with_connect_flags() {
        assert_eq!(
            preprocess_args(&a("sshr -p 22 host")),
            a("sshr connect -p 22 host")
        );
    }

    #[test]
    fn test_inject_verbose_and_port() {
        assert_eq!(
            preprocess_args(&a("sshr -vvv -p 22 root@host")),
            a("sshr connect -vvv -p 22 root@host")
        );
    }

    #[test]
    fn test_inject_global_flag_f_with_value() {
        assert_eq!(
            preprocess_args(&a("sshr -F /path host")),
            a("sshr connect -F /path host")
        );
    }

    #[test]
    fn test_inject_global_flag_f_equals() {
        assert_eq!(
            preprocess_args(&a("sshr -F=/path host")),
            a("sshr connect -F=/path host")
        );
    }

    #[test]
    fn test_inject_global_long_flag_with_value() {
        assert_eq!(
            preprocess_args(&a("sshr --config-file /path host")),
            a("sshr connect --config-file /path host")
        );
    }

    #[test]
    fn test_inject_global_format_equals() {
        assert_eq!(
            preprocess_args(&a("sshr --format=json host")),
            a("sshr connect --format=json host")
        );
    }

    #[test]
    fn test_inject_no_color_flag() {
        assert_eq!(
            preprocess_args(&a("sshr --no-color host")),
            a("sshr connect --no-color host")
        );
    }

    #[test]
    fn test_inject_host_before_double_dash() {
        let input = a("sshr host -- -L 8080:lo:80");
        let expected = a("sshr connect host -- -L 8080:lo:80");
        assert_eq!(preprocess_args(&input), expected);
    }

    // --- Casos que NO inyectan ---

    #[test]
    fn test_no_inject_subcommand() {
        assert_eq!(preprocess_args(&a("sshr list")), a("sshr list"));
    }

    #[test]
    fn test_no_inject_alias() {
        assert_eq!(preprocess_args(&a("sshr ls")), a("sshr ls"));
    }

    #[test]
    fn test_no_inject_explicit_connect() {
        assert_eq!(
            preprocess_args(&a("sshr connect host")),
            a("sshr connect host")
        );
    }

    #[test]
    fn test_no_inject_help_subcommand() {
        assert_eq!(
            preprocess_args(&a("sshr help connect")),
            a("sshr help connect")
        );
    }

    #[test]
    fn test_no_inject_help_flag() {
        assert_eq!(preprocess_args(&a("sshr --help")), a("sshr --help"));
    }

    #[test]
    fn test_no_inject_version_flag() {
        assert_eq!(preprocess_args(&a("sshr --version")), a("sshr --version"));
    }

    #[test]
    fn test_no_inject_no_args() {
        assert_eq!(preprocess_args(&a("sshr")), a("sshr"));
    }

    #[test]
    fn test_no_inject_bare_double_dash() {
        assert_eq!(preprocess_args(&a("sshr -- host")), a("sshr -- host"));
    }

    #[test]
    fn test_no_inject_global_flag_then_subcommand() {
        assert_eq!(
            preprocess_args(&a("sshr -F /path list")),
            a("sshr -F /path list")
        );
    }
}
