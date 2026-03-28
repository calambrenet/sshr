mod cli;
mod commands;
mod config;
mod utils;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};

fn main() -> Result<()> {
    let mut cli = Cli::parse();

    // Expandir ~ en la ruta de configuración (clap la recibe como literal)
    let config_str = cli.config_file.to_string_lossy().to_string();
    cli.config_file = PathBuf::from(utils::expand_tilde(&config_str));

    // Configurar colores globalmente
    if cli.no_color || std::env::var("NO_COLOR").is_ok() {
        // Desactivamos colores (lo implementaremos después)
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
