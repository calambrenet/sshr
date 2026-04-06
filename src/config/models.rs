// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// Port forwarding rule (local or remote).
///
/// SSH format: `[bind_address:]port:host:hostport`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForwardRule {
    pub bind_address: Option<String>,
    pub port: u16,
    pub host: String,
    pub host_port: u16,
}

impl ForwardRule {
    /// Parses a forwarding rule from SSH format.
    ///
    /// Supported formats:
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

impl fmt::Display for ForwardRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.bind_address.as_deref() == Some("localhost") {
            write!(f, "{} {}:{}", self.port, self.host, self.port)
        } else {
            write!(
                f,
                "{}:{} {}:{}",
                self.bind_address.as_deref().unwrap_or("*"),
                self.port,
                self.host,
                self.port
            )
        }
    }
}

/// Represents a Host block from the ssh_config file.
///
/// Known limitations:
/// - Only one `IdentityFile` per host (OpenSSH allows multiple; eventually will be `Vec<PathBuf>`)
/// - No support for `Match` blocks (silently ignored)
/// - No expansion of `~user/` (only `~/`)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Host {
    /// Primary host name (first pattern)
    pub name: String,
    /// Host block patterns (may include wildcards)
    pub patterns: Vec<String>,
    /// Real hostname or IP
    pub hostname: Option<String>,
    /// SSH user
    pub user: Option<String>,
    /// SSH port
    pub port: Option<u16>,
    /// Private key file
    pub identity_file: Option<PathBuf>,
    /// ProxyJump host
    pub proxy_jump: Option<String>,
    /// ProxyCommand
    pub proxy_command: Option<String>,
    /// ForwardAgent (yes/no)
    pub forward_agent: Option<bool>,
    /// Local forwards (LocalForward)
    pub local_forwards: Vec<ForwardRule>,
    /// Remote forwards (RemoteForward)
    pub remote_forwards: Vec<ForwardRule>,
    /// ServerAliveInterval in seconds
    pub server_alive_interval: Option<u64>,
    /// ServerAliveCountMax
    pub server_alive_count_max: Option<u64>,
    /// Additional options not explicitly mapped
    pub extra_options: HashMap<String, String>,

    // Additional metadata
    #[serde(default)]
    pub tags: Vec<String>,
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
            tags: Vec::new(),
        }
    }

    /// Returns the effective hostname (configured hostname or the host name).
    pub fn effective_hostname(&self) -> &str {
        self.hostname.as_deref().unwrap_or(&self.name)
    }

    /// Returns the effective port (configured or 22 by default).
    pub fn effective_port(&self) -> u16 {
        self.port.unwrap_or(22)
    }

    /// Returns the effective user (configured or the current system user).
    pub fn effective_user(&self) -> String {
        self.user.clone().unwrap_or_else(|| {
            std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .unwrap_or_else(|_| String::from("root"))
        })
    }

    /// Returns true if any of the patterns contains wildcards (`*` or `?`).
    pub fn is_pattern(&self) -> bool {
        self.patterns
            .iter()
            .any(|p| p.contains('*') || p.contains('?'))
    }

    /// Returns the connection string in `user@hostname:port` format.
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

/// Full SSH configuration parsed from a file.
#[derive(Debug)]
pub struct SshConfig {
    /// Global options (before the first Host block)
    pub global_options: HashMap<String, String>,
    /// Parsed Host blocks
    pub hosts: Vec<Host>,
    /// Source file path (if parsed from a file)
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

    /// Finds a host by exact name (case-insensitive).
    pub fn find_host(&self, name: &str) -> Option<&Host> {
        let name_lower = name.to_lowercase();
        self.hosts
            .iter()
            .find(|h| h.name.to_lowercase() == name_lower)
    }

    /// Returns only concrete hosts (without wildcards).
    pub fn concrete_hosts(&self) -> Vec<&Host> {
        self.hosts.iter().filter(|h| !h.is_pattern()).collect()
    }
}
