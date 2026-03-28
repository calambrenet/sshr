// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;

/// Regla de reenvío de puertos (local o remoto).
///
/// Formato SSH: `[bind_address:]port:host:hostport`
#[derive(Debug, Clone, PartialEq)]
pub struct ForwardRule {
    pub bind_address: Option<String>,
    pub port: u16,
    pub host: String,
    pub host_port: u16,
}

impl ForwardRule {
    /// Parsea una regla de reenvío desde el formato SSH.
    ///
    /// Formatos soportados:
    /// - `port:host:hostport`
    /// - `bind_address:port:host:hostport`
    pub fn parse(s: &str) -> Option<ForwardRule> {
        let parts: Vec<&str> = s.split(':').collect();
        match parts.len() {
            // port:host:hostport
            3 => {
                let port = parts[0].parse().ok()?;
                let host = parts[1].to_string();
                let host_port = parts[2].parse().ok()?;
                Some(ForwardRule {
                    bind_address: None,
                    port,
                    host,
                    host_port,
                })
            }
            // bind_address:port:host:hostport
            4 => {
                let bind_address = Some(parts[0].to_string());
                let port = parts[1].parse().ok()?;
                let host = parts[2].to_string();
                let host_port = parts[3].parse().ok()?;
                Some(ForwardRule {
                    bind_address,
                    port,
                    host,
                    host_port,
                })
            }
            _ => None,
        }
    }
}

/// Representa un bloque Host del fichero ssh_config.
///
/// Limitaciones conocidas:
/// - Solo un `IdentityFile` por host (OpenSSH permite múltiples; eventualmente será `Vec<PathBuf>`)
/// - Sin soporte para bloques `Match` (se ignoran silenciosamente)
/// - Sin expansión de `~user/` (solo `~/`)
#[derive(Debug, Clone)]
pub struct Host {
    /// Nombre principal del host (primer patrón)
    pub name: String,
    /// Patrones del bloque Host (puede incluir wildcards)
    pub patterns: Vec<String>,
    /// Hostname o IP real
    pub hostname: Option<String>,
    /// Usuario SSH
    pub user: Option<String>,
    /// Puerto SSH
    pub port: Option<u16>,
    /// Fichero de clave privada
    pub identity_file: Option<PathBuf>,
    /// ProxyJump host
    pub proxy_jump: Option<String>,
    /// ProxyCommand
    pub proxy_command: Option<String>,
    /// ForwardAgent (yes/no)
    pub forward_agent: Option<bool>,
    /// Reenvíos locales (LocalForward)
    pub local_forwards: Vec<ForwardRule>,
    /// Reenvíos remotos (RemoteForward)
    pub remote_forwards: Vec<ForwardRule>,
    /// ServerAliveInterval en segundos
    pub server_alive_interval: Option<u64>,
    /// ServerAliveCountMax
    pub server_alive_count_max: Option<u64>,
    /// Opciones adicionales no mapeadas explícitamente
    pub extra_options: HashMap<String, String>,
}

impl Host {
    pub fn new(name: &str, patterns: Vec<String>) -> Self {
        Host {
            name: name.to_string(),
            patterns,
            hostname: None,
            user: None,
            port: None,
            identity_file: None,
            proxy_jump: None,
            proxy_command: None,
            forward_agent: None,
            local_forwards: Vec::new(),
            remote_forwards: Vec::new(),
            server_alive_interval: None,
            server_alive_count_max: None,
            extra_options: HashMap::new(),
        }
    }

    /// Devuelve el hostname efectivo (hostname configurado o el nombre del host).
    pub fn effective_hostname(&self) -> &str {
        self.hostname.as_deref().unwrap_or(&self.name)
    }

    /// Devuelve el puerto efectivo (configurado o 22 por defecto).
    pub fn effective_port(&self) -> u16 {
        self.port.unwrap_or(22)
    }

    /// Devuelve el usuario efectivo (configurado o el usuario actual del sistema).
    pub fn effective_user(&self) -> String {
        self.user.clone().unwrap_or_else(|| {
            std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .unwrap_or_else(|_| String::from("root"))
        })
    }

    /// Devuelve true si alguno de los patrones contiene wildcards (`*` o `?`).
    pub fn is_pattern(&self) -> bool {
        self.patterns
            .iter()
            .any(|p| p.contains('*') || p.contains('?'))
    }

    /// Devuelve la cadena de conexión en formato `user@hostname:port`.
    pub fn connection_string(&self) -> String {
        let user = self.effective_user();
        let hostname = self.effective_hostname();
        let port = self.effective_port();
        if port == 22 {
            format!("{user}@{hostname}")
        } else {
            format!("{user}@{hostname}:{port}")
        }
    }
}

/// Configuración SSH completa parseada de un fichero.
#[derive(Debug)]
pub struct SshConfig {
    /// Opciones globales (antes del primer bloque Host)
    pub global_options: HashMap<String, String>,
    /// Bloques Host parseados
    pub hosts: Vec<Host>,
    /// Ruta del fichero fuente (si se parseó desde fichero)
    pub source_path: Option<PathBuf>,
}

impl SshConfig {
    pub fn new() -> Self {
        SshConfig {
            global_options: HashMap::new(),
            hosts: Vec::new(),
            source_path: None,
        }
    }

    /// Busca un host por nombre exacto (case-insensitive).
    pub fn find_host(&self, name: &str) -> Option<&Host> {
        let name_lower = name.to_lowercase();
        self.hosts
            .iter()
            .find(|h| h.name.to_lowercase() == name_lower)
    }

    /// Devuelve solo los hosts concretos (sin wildcards).
    pub fn concrete_hosts(&self) -> Vec<&Host> {
        self.hosts.iter().filter(|h| !h.is_pattern()).collect()
    }
}
