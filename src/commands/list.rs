use crate::{
    cli::{Cli, ListArgs, OutputFormat},
    config::{models::Host, parser::parse_ssh_config},
    utils::color,
};
use anyhow::Result;

pub fn execute(cli: &Cli, args: &ListArgs) -> Result<()> {
    let config = parse_ssh_config(&cli.config_file)?;

    // Filtrar hosts concretos (sin wildcards)
    let mut hosts: Vec<&Host> = config.concrete_hosts();

    // Filtrar por tag si se especificó
    if !args.tag.is_empty() {
        hosts.retain(|h| {
            println!("Host {} tags: {:?}", h.name, h.tags);
            args.tag.iter().any(|t| h.tags.contains(t))
        });
    }

    // Ordenar
    sort_hosts(&mut hosts, args);

    if hosts.is_empty() {
        println!("{}", color::dimmed("No hay hosts configurados."));
        println!(
            "{}",
            color::dimmed("Usa 'sshm add <nombre> <hostname>' para añadir uno.")
        );
        return Ok(());
    }

    match cli.format {
        OutputFormat::Text => {
            if args.verbose {
                print_verbose(&hosts);
            } else {
                print_table(&hosts);
            }
        }
        OutputFormat::Json => print_json(&hosts)?,
        OutputFormat::Csv => print_csv(&hosts),
    }

    Ok(())
}

fn sort_hosts(hosts: &mut [&Host], args: &ListArgs) {
    hosts.sort_by(|a, b| {
        use crate::cli::SortField;
        let cmp = match args.sort {
            SortField::Name => a.name.cmp(&b.name),
            SortField::Hostname => a.effective_hostname().cmp(b.effective_hostname()),
            SortField::User => a.effective_user().cmp(&b.effective_user()),
            SortField::Port => a.effective_port().cmp(&b.effective_port()),
            SortField::LastUsed => std::cmp::Ordering::Equal, // TODO: falta por implementar
        };
        if args.reverse { cmp.reverse() } else { cmp }
    });
}

/// Rellena un string coloreado con espacios hasta alcanzar `width` caracteres visibles.
fn pad_right(colored: String, visible_len: usize, width: usize) -> String {
    if visible_len >= width {
        colored
    } else {
        format!("{}{}", colored, " ".repeat(width - visible_len))
    }
}

fn print_table(hosts: &[&Host]) {
    // Calcular anchos de columna basados en longitud visible (sin códigos ANSI)
    let name_width = hosts.iter().map(|h| h.name.len()).max().unwrap_or(4).max(4);
    let host_width = hosts
        .iter()
        .map(|h| h.effective_hostname().len())
        .max()
        .unwrap_or(8)
        .max(8);

    // Cabecera — los códigos ANSI no afectan al padding porque usamos pad_right
    println!(
        "  {}  {}  {:>5}  {}",
        pad_right(color::header("NAME"), 4, name_width),
        pad_right(color::header("HOSTNAME"), 8, host_width),
        color::header("PORT"),
        color::header("USER"),
    );

    // Separador
    println!(
        "  {}  {}  {}  {}",
        color::dimmed(&"─".repeat(name_width)),
        color::dimmed(&"─".repeat(host_width)),
        color::dimmed("─────"),
        color::dimmed("────────"),
    );

    // Filas
    for host in hosts {
        let port_str = host.effective_port().to_string();
        let user = host.effective_user();

        println!(
            "  {}  {}  {:>5}  {}",
            pad_right(color::host_name(&host.name), host.name.len(), name_width),
            pad_right(
                color::hostname_addr(host.effective_hostname()),
                host.effective_hostname().len(),
                host_width
            ),
            color::port_number(&port_str),
            color::user_name(&user),
        );
    }

    // Resumen
    println!("\n{}", color::dimmed(&format!("{} hosts", hosts.len())));
}

fn print_verbose(hosts: &[&Host]) {
    for (i, host) in hosts.iter().enumerate() {
        if i > 0 {
            println!();
        }

        println!("{} {}", color::dimmed("●"), color::host_name(&host.name));
        println!(
            "  Hostname:  {}",
            color::hostname_addr(host.effective_hostname())
        );
        println!("  User:      {}", color::user_name(&host.effective_user()));
        println!(
            "  Port:      {}",
            color::port_number(&host.effective_port().to_string())
        );

        if let Some(ref key) = host.identity_file {
            println!("  Key:       {}", key.display());
        }
        if let Some(ref jump) = host.proxy_jump {
            println!("  ProxyJump: {}", jump);
        }
        if !host.tags.is_empty() {
            let tags_str: Vec<String> = host
                .tags
                .iter()
                .map(|t| color::tag(&format!("[{}]", t)))
                .collect();
            println!("  Tags:      {}", tags_str.join(" "));
        }
        if !host.local_forwards.is_empty() {
            for lf in &host.local_forwards {
                println!("  Forward:   L {} -> {}", lf.port, lf);
            }
        }
    }
}

fn print_json(hosts: &[&Host]) -> Result<()> {
    let json = serde_json::to_string_pretty(hosts)?;
    println!("{}", json);
    Ok(())
}

fn print_csv(hosts: &[&Host]) {
    println!("name,hostname,port,user,proxy_jump,tags");
    for h in hosts {
        println!(
            "{},{},{},{},{},\"{}\"",
            h.name,
            h.effective_hostname(),
            h.effective_port(),
            h.effective_user(),
            h.proxy_jump.as_deref().unwrap_or(""),
            h.tags.join(";"),
        );
    }
}
