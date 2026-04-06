// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::models::{ForwardRule, Host, SshConfig};
use crate::utils::expand_tilde;

/// Parsea un fichero ssh_config desde disco.
///
/// Si el fichero no existe, devuelve una `SshConfig` vacía (ssh funciona sin config).
/// Si el fichero existe pero no se puede leer, devuelve error.
pub fn parse_ssh_config(path: &Path) -> Result<SshConfig> {
    if !path.exists() {
        let mut config = SshConfig::new();
        config.source_path = Some(path.to_path_buf());
        return Ok(config);
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("No se pudo leer el fichero SSH config: {}", path.display()))?;

    parse_ssh_config_str(&content, Some(path.to_path_buf()))
}

/// Parsea el contenido de un ssh_config desde un string.
///
/// Útil para testing sin necesidad de ficheros en disco.
pub fn parse_ssh_config_str(content: &str, source_path: Option<PathBuf>) -> Result<SshConfig> {
    let mut config = SshConfig::new();
    config.source_path = source_path;

    let mut current_host: Option<Host> = None;

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed
        let trimmed = line.trim();

        // Ignorar líneas vacías y comentarios
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parsear directiva
        let (key, value) = match parse_directive(trimmed) {
            Some(kv) => kv,
            None => continue,
        };

        let key_lower = key.to_lowercase();

        // Directiva Include: warning a stderr (no soportada)
        if key_lower == "include" {
            eprintln!(
                "sshr: advertencia: directiva 'Include' no soportada (línea {line_num}), se ignora"
            );
            continue;
        }

        // Directiva Match: se ignora silenciosamente
        if key_lower == "match" {
            continue;
        }

        // Nuevo bloque Host
        if key_lower == "host" {
            // Guardar host anterior
            if let Some(host) = current_host.take() {
                config.hosts.push(host);
            }

            let patterns: Vec<String> = value.split_whitespace().map(String::from).collect();
            if patterns.is_empty() {
                bail!("Línea {line_num}: bloque Host sin nombre");
            }

            let name = patterns[0].clone();
            current_host = Some(Host::new(&name, patterns));
            continue;
        }

        // Aplicar directiva al host actual o a opciones globales
        match current_host.as_mut() {
            Some(host) => apply_directive(host, &key_lower, &value, line_num)?,
            None => {
                // Opciones globales (antes del primer Host)
                config.global_options.insert(key_lower, value);
            }
        }
    }

    // No olvidar el último host
    if let Some(host) = current_host {
        config.hosts.push(host);
    }

    Ok(config)
}

/// Separa una línea de directiva en clave y valor.
///
/// Soporta ambos formatos SSH:
/// - `Key Value` (separado por espacio)
/// - `Key=Value` (separado por `=`)
fn parse_directive(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // El key es el primer token (hasta espacio o =)
    match line.find(|c: char| c == '=' || c.is_whitespace()) {
        Some(key_end) => {
            let key = line[..key_end].to_string();
            let rest = line[key_end..].trim_start();

            // Saltar separador '=' opcional
            let value = rest.strip_prefix('=').map_or(rest, |s| s.trim_start());

            if value.is_empty() {
                return Some((key, String::new()));
            }

            Some((key, value.to_string()))
        }
        // Solo keyword, sin separador ni valor (e.g., "Host" sin nombre)
        None => Some((line.to_string(), String::new())),
    }
}

/// Aplica una directiva parseada a un Host.
fn apply_directive(host: &mut Host, key: &str, value: &str, line_num: usize) -> Result<()> {
    match key {
        "hostname" => host.hostname = Some(value.to_string()),
        "user" => host.user = Some(value.to_string()),
        "port" => {
            host.port = Some(
                value
                    .parse::<u16>()
                    .with_context(|| format!("Línea {line_num}: puerto inválido '{value}'"))?,
            );
        }
        "identityfile" => {
            host.identity_file = Some(PathBuf::from(expand_tilde(value)));
        }
        "proxyjump" => host.proxy_jump = Some(value.to_string()),
        "proxycommand" => host.proxy_command = Some(value.to_string()),
        "forwardagent" => host.forward_agent = Some(parse_yes_no(value, line_num)?),
        "localforward" => {
            if let Some(rule) = ForwardRule::parse(value) {
                host.local_forwards.push(rule);
            }
        }
        "remoteforward" => {
            if let Some(rule) = ForwardRule::parse(value) {
                host.remote_forwards.push(rule);
            }
        }
        "serveraliveinterval" => {
            host.server_alive_interval = Some(value.parse::<u64>().with_context(|| {
                format!("Línea {line_num}: ServerAliveInterval inválido '{value}'")
            })?);
        }
        "serveralivecountmax" => {
            host.server_alive_count_max = Some(value.parse::<u64>().with_context(|| {
                format!("Línea {line_num}: ServerAliveCountMax inválido '{value}'")
            })?);
        }
        // Opciones desconocidas se preservan en extra_options
        _ => {
            host.extra_options
                .insert(key.to_string(), value.to_string());
        }
    }
    Ok(())
}

/// Parsea un valor yes/no a bool.
fn parse_yes_no(value: &str, line_num: usize) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ => bail!("Línea {line_num}: se esperaba 'yes' o 'no', encontrado '{value}'"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_host_simple() {
        let content = "\
Host myserver
    HostName 192.168.1.100
    User admin
    Port 2222
";
        let config = parse_ssh_config_str(content, None).unwrap();
        assert_eq!(config.hosts.len(), 1);
        let host = &config.hosts[0];
        assert_eq!(host.name, "myserver");
        assert_eq!(host.hostname.as_deref(), Some("192.168.1.100"));
        assert_eq!(host.user.as_deref(), Some("admin"));
        assert_eq!(host.port, Some(2222));
    }

    #[test]
    fn test_parse_multiple_hosts() {
        let content = "\
Host server1
    HostName 10.0.0.1

Host server2
    HostName 10.0.0.2
";
        let config = parse_ssh_config_str(content, None).unwrap();
        assert_eq!(config.hosts.len(), 2);
        assert_eq!(config.hosts[0].name, "server1");
        assert_eq!(config.hosts[1].name, "server2");
    }

    #[test]
    fn test_global_options() {
        let content = "\
ServerAliveInterval 60

Host myserver
    HostName 10.0.0.1
";
        let config = parse_ssh_config_str(content, None).unwrap();
        assert_eq!(
            config.global_options.get("serveraliveinterval").unwrap(),
            "60"
        );
        assert_eq!(config.hosts.len(), 1);
    }

    #[test]
    fn test_case_insensitive_directives() {
        let content = "\
Host test
    HOSTNAME 10.0.0.1
";
        let config = parse_ssh_config_str(content, None).unwrap();
        assert_eq!(config.hosts[0].hostname.as_deref(), Some("10.0.0.1"));

        let content2 = "\
Host test
    hostname 10.0.0.2
";
        let config2 = parse_ssh_config_str(content2, None).unwrap();
        assert_eq!(config2.hosts[0].hostname.as_deref(), Some("10.0.0.2"));
    }

    #[test]
    fn test_equals_format() {
        let content = "\
Host test
    HostName=10.0.0.1
    Port=3333
";
        let config = parse_ssh_config_str(content, None).unwrap();
        let host = &config.hosts[0];
        assert_eq!(host.hostname.as_deref(), Some("10.0.0.1"));
        assert_eq!(host.port, Some(3333));
    }

    #[test]
    fn test_comments_and_empty_lines() {
        let content = "\
# This is a comment

Host test
    # Another comment
    HostName 10.0.0.1

    User admin
";
        let config = parse_ssh_config_str(content, None).unwrap();
        assert_eq!(config.hosts.len(), 1);
        assert_eq!(config.hosts[0].user.as_deref(), Some("admin"));
    }

    #[test]
    fn test_unknown_directives_preserved() {
        let content = "\
Host test
    HostName 10.0.0.1
    AddKeysToAgent yes
    Compression yes
";
        let config = parse_ssh_config_str(content, None).unwrap();
        let host = &config.hosts[0];
        assert_eq!(host.extra_options.get("addkeystoagent").unwrap(), "yes");
        assert_eq!(host.extra_options.get("compression").unwrap(), "yes");
    }

    #[test]
    fn test_forward_rule_parsing() {
        let rule = ForwardRule::parse("8080:localhost:80").unwrap();
        assert_eq!(rule.port, 8080);
        assert_eq!(rule.host, "localhost");
        assert_eq!(rule.host_port, 80);
        assert!(rule.bind_address.is_none());

        let rule = ForwardRule::parse("0.0.0.0:8080:localhost:80").unwrap();
        assert_eq!(rule.bind_address.as_deref(), Some("0.0.0.0"));
        assert_eq!(rule.port, 8080);
    }

    #[test]
    fn test_forward_rule_invalid() {
        assert!(ForwardRule::parse("invalid").is_none());
        assert!(ForwardRule::parse("a:b").is_none());
        assert!(ForwardRule::parse("a:b:c:d:e").is_none());
    }

    #[test]
    fn test_yes_no_values() {
        let content = "\
Host test1
    ForwardAgent yes
";
        let config = parse_ssh_config_str(content, None).unwrap();
        assert_eq!(config.hosts[0].forward_agent, Some(true));

        let content = "\
Host test2
    ForwardAgent no
";
        let config = parse_ssh_config_str(content, None).unwrap();
        assert_eq!(config.hosts[0].forward_agent, Some(false));
    }

    #[test]
    fn test_yes_no_invalid() {
        let content = "\
Host test
    ForwardAgent maybe
";
        let result = parse_ssh_config_str(content, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("yes"));
    }

    #[test]
    fn test_empty_config() {
        let config = parse_ssh_config_str("", None).unwrap();
        assert!(config.hosts.is_empty());
        assert!(config.global_options.is_empty());
    }

    #[test]
    fn test_wildcard_host() {
        let content = "\
Host *
    ServerAliveInterval 60
    ServerAliveCountMax 3
";
        let config = parse_ssh_config_str(content, None).unwrap();
        assert_eq!(config.hosts.len(), 1);
        assert!(config.hosts[0].is_pattern());
        assert_eq!(config.hosts[0].server_alive_interval, Some(60));
        assert_eq!(config.hosts[0].server_alive_count_max, Some(3));
    }

    #[test]
    fn test_invalid_port_error() {
        let content = "\
Host test
    Port notanumber
";
        let result = parse_ssh_config_str(content, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("puerto inválido"));
    }

    #[test]
    fn test_host_without_name_error() {
        let content = "Host\n    HostName 10.0.0.1";
        let result = parse_ssh_config_str(content, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("sin nombre"));
    }

    #[test]
    fn test_file_not_found_returns_empty() {
        let config = parse_ssh_config(Path::new("/nonexistent/path/ssh_config")).unwrap();
        assert!(config.hosts.is_empty());
        assert!(config.source_path.is_some());
    }

    #[test]
    fn test_local_and_remote_forwards() {
        let content = "\
Host test
    LocalForward 8080:localhost:80
    RemoteForward 0.0.0.0:9090:db:5432
";
        let config = parse_ssh_config_str(content, None).unwrap();
        let host = &config.hosts[0];
        assert_eq!(host.local_forwards.len(), 1);
        assert_eq!(host.remote_forwards.len(), 1);
        assert_eq!(host.local_forwards[0].port, 8080);
        assert_eq!(host.local_forwards[0].host, "localhost");
        assert_eq!(host.local_forwards[0].host_port, 80);
        assert_eq!(
            host.remote_forwards[0].bind_address.as_deref(),
            Some("0.0.0.0")
        );
        assert_eq!(host.remote_forwards[0].port, 9090);
    }

    #[test]
    fn test_identity_file_tilde_expansion() {
        let content = "\
Host test
    IdentityFile ~/.ssh/id_ed25519
";
        let config = parse_ssh_config_str(content, None).unwrap();
        let path = config.hosts[0].identity_file.as_ref().unwrap();
        // Debe haberse expandido ~ -> $HOME
        assert!(!path.to_string_lossy().starts_with('~'));
    }

    #[test]
    fn test_proxy_jump_and_command() {
        let content = "\
Host test
    ProxyJump bastion
    ProxyCommand ssh -W %h:%p gateway
";
        let config = parse_ssh_config_str(content, None).unwrap();
        let host = &config.hosts[0];
        assert_eq!(host.proxy_jump.as_deref(), Some("bastion"));
        assert_eq!(host.proxy_command.as_deref(), Some("ssh -W %h:%p gateway"));
    }

    #[test]
    fn test_multiple_patterns_in_host() {
        let content = "\
Host web1 web2 web3
    HostName 10.0.0.1
";
        let config = parse_ssh_config_str(content, None).unwrap();
        assert_eq!(config.hosts[0].name, "web1");
        assert_eq!(config.hosts[0].patterns, vec!["web1", "web2", "web3"]);
    }

    #[test]
    fn test_parse_directive_formats() {
        // Espacio simple
        let (k, v) = parse_directive("HostName 10.0.0.1").unwrap();
        assert_eq!(k, "HostName");
        assert_eq!(v, "10.0.0.1");

        // Con =
        let (k, v) = parse_directive("HostName=10.0.0.1").unwrap();
        assert_eq!(k, "HostName");
        assert_eq!(v, "10.0.0.1");

        // Con = y espacios
        let (k, v) = parse_directive("HostName = 10.0.0.1").unwrap();
        assert_eq!(k, "HostName");
        assert_eq!(v, "10.0.0.1");

        // Valor con espacios (ProxyCommand)
        let (k, v) = parse_directive("ProxyCommand ssh -W %h:%p gw").unwrap();
        assert_eq!(k, "ProxyCommand");
        assert_eq!(v, "ssh -W %h:%p gw");
    }
}
