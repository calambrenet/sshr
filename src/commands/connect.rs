// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::cli::{Cli, ConnectArgs};
use crate::config::models::Host;
use crate::config::parser::parse_ssh_config;
use crate::utils::expand_tilde;

/// Builds the arguments for the ssh command.
///
/// Takes the config path, subcommand arguments, and the list
/// of parsed hosts. Returns the argument vector and, optionally,
/// the reference to the Host found in config.
pub fn build_ssh_args<'a>(
    config_path: &Path,
    args: &ConnectArgs,
    hosts: &'a [Host],
) -> (Vec<String>, Option<&'a Host>) {
    let mut ssh_args: Vec<String> = Vec::new();

    // Config file override: if the user specified a path other than the default
    let config_str = config_path.to_string_lossy().to_string();
    let default_config = expand_tilde("~/.ssh/config");
    if config_str != default_config {
        ssh_args.push("-F".to_string());
        ssh_args.push(config_str);
    }

    // Look up the host in the configuration
    let host_entry = hosts
        .iter()
        .find(|h| h.name.to_lowercase() == args.host.to_lowercase());

    // Port override
    if let Some(port) = args.port {
        ssh_args.push("-p".to_string());
        ssh_args.push(port.to_string());
    }

    // Verbosity
    match args.verbose {
        1 => ssh_args.push("-v".to_string()),
        2 => ssh_args.push("-vv".to_string()),
        3.. => ssh_args.push("-vvv".to_string()),
        _ => {}
    }

    // Host target (with optional user override)
    let target = if let Some(ref user) = args.user {
        format!("{user}@{}", args.host)
    } else {
        args.host.clone()
    };
    ssh_args.push(target);

    // Extra arguments passed after --
    ssh_args.extend(args.ssh_args.iter().cloned());

    (ssh_args, host_entry)
}

/// Executes ssh replacing the current process (Unix).
///
/// On Unix, `exec()` replaces the sshr process with ssh — if successful, it never returns.
#[cfg(unix)]
fn exec_ssh(ssh_args: &[String]) -> Result<()> {
    use std::os::unix::process::CommandExt;

    let mut cmd = Command::new("ssh");
    cmd.args(ssh_args);

    // exec() replaces the process — if successful, it never returns
    let err = cmd.exec();
    Err(err).context("Failed to execute ssh")
}

/// Executes ssh as a child process (non-Unix fallback).
#[cfg(not(unix))]
fn exec_ssh(ssh_args: &[String]) -> Result<()> {
    let status = Command::new("ssh")
        .args(ssh_args)
        .status()
        .context("Failed to execute ssh")?;

    std::process::exit(status.code().unwrap_or(1));
}

pub fn execute(cli: &Cli, args: &ConnectArgs) -> Result<()> {
    // Parse SSH configuration
    let config = parse_ssh_config(&cli.config_file)?;

    // Build SSH arguments
    let (ssh_args, host_entry) = build_ssh_args(&cli.config_file, args, &config.hosts);

    // Print connection info to stderr
    if let Some(host) = host_entry {
        eprintln!(
            "sshr: connecting to {} ({})",
            host.name,
            host.connection_string()
        );
    } else {
        eprintln!(
            "sshr: connecting to {} (host not found in config, using ssh directly)",
            args.host
        );
    }

    // Execute ssh
    exec_ssh(&ssh_args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn default_config_path() -> PathBuf {
        PathBuf::from(expand_tilde("~/.ssh/config"))
    }

    fn make_args(host: &str) -> ConnectArgs {
        ConnectArgs {
            host: host.to_string(),
            port: None,
            user: None,
            verbose: 0,
            persistent: false,
            trust: false,
            ssh_args: vec![],
        }
    }

    fn make_test_hosts() -> Vec<Host> {
        let mut host = Host::new("myserver", vec!["myserver".to_string()]);
        host.hostname = Some("192.168.1.100".to_string());
        host.user = Some("admin".to_string());
        host.port = Some(2222);
        vec![host]
    }

    #[test]
    fn test_default_config_no_flag() {
        let args = make_args("myhost");
        let (ssh_args, _) = build_ssh_args(&default_config_path(), &args, &[]);
        assert!(!ssh_args.contains(&"-F".to_string()));
    }

    #[test]
    fn test_custom_config_adds_flag() {
        let custom_path = PathBuf::from("/tmp/custom_config");
        let args = make_args("myhost");
        let (ssh_args, _) = build_ssh_args(&custom_path, &args, &[]);
        assert!(ssh_args.contains(&"-F".to_string()));
        assert!(ssh_args.contains(&"/tmp/custom_config".to_string()));
    }

    #[test]
    fn test_port_override() {
        let mut args = make_args("myhost");
        args.port = Some(2222);
        let (ssh_args, _) = build_ssh_args(&default_config_path(), &args, &[]);
        let pos = ssh_args.iter().position(|a| a == "-p").unwrap();
        assert_eq!(ssh_args[pos + 1], "2222");
    }

    #[test]
    fn test_user_override() {
        let mut args = make_args("myhost");
        args.user = Some("admin".to_string());
        let (ssh_args, _) = build_ssh_args(&default_config_path(), &args, &[]);
        assert!(ssh_args.contains(&"admin@myhost".to_string()));
    }

    #[test]
    fn test_verbose_level_0() {
        let args = make_args("myhost");
        let (ssh_args, _) = build_ssh_args(&default_config_path(), &args, &[]);
        assert!(!ssh_args.iter().any(|a| a.starts_with("-v")));
    }

    #[test]
    fn test_verbose_level_1() {
        let mut args = make_args("myhost");
        args.verbose = 1;
        let (ssh_args, _) = build_ssh_args(&default_config_path(), &args, &[]);
        assert!(ssh_args.contains(&"-v".to_string()));
    }

    #[test]
    fn test_verbose_level_2() {
        let mut args = make_args("myhost");
        args.verbose = 2;
        let (ssh_args, _) = build_ssh_args(&default_config_path(), &args, &[]);
        assert!(ssh_args.contains(&"-vv".to_string()));
    }

    #[test]
    fn test_verbose_level_3() {
        let mut args = make_args("myhost");
        args.verbose = 3;
        let (ssh_args, _) = build_ssh_args(&default_config_path(), &args, &[]);
        assert!(ssh_args.contains(&"-vvv".to_string()));
    }

    #[test]
    fn test_extra_args_passed() {
        let mut args = make_args("myhost");
        args.ssh_args = vec!["-L".to_string(), "8080:localhost:80".to_string()];
        let (ssh_args, _) = build_ssh_args(&default_config_path(), &args, &[]);
        assert!(ssh_args.contains(&"-L".to_string()));
        assert!(ssh_args.contains(&"8080:localhost:80".to_string()));
    }

    #[test]
    fn test_host_found_in_config() {
        let hosts = make_test_hosts();
        let args = make_args("myserver");
        let (_, host_entry) = build_ssh_args(&default_config_path(), &args, &hosts);
        assert!(host_entry.is_some());
        assert_eq!(host_entry.unwrap().name, "myserver");
    }

    #[test]
    fn test_host_not_found_in_config() {
        let hosts = make_test_hosts();
        let args = make_args("unknown");
        let (_, host_entry) = build_ssh_args(&default_config_path(), &args, &hosts);
        assert!(host_entry.is_none());
    }

    #[test]
    fn test_combined_overrides() {
        let custom_path = PathBuf::from("/tmp/my_config");
        let mut args = make_args("myhost");
        args.port = Some(3333);
        args.user = Some("root".to_string());
        args.verbose = 2;
        args.ssh_args = vec!["-N".to_string()];

        let (ssh_args, _) = build_ssh_args(&custom_path, &args, &[]);

        assert!(ssh_args.contains(&"-F".to_string()));
        assert!(ssh_args.contains(&"/tmp/my_config".to_string()));
        assert!(ssh_args.contains(&"-p".to_string()));
        assert!(ssh_args.contains(&"3333".to_string()));
        assert!(ssh_args.contains(&"-vv".to_string()));
        assert!(ssh_args.contains(&"root@myhost".to_string()));
        assert!(ssh_args.contains(&"-N".to_string()));
    }

    #[test]
    fn test_host_target_is_last_before_extra_args() {
        let mut args = make_args("myhost");
        args.ssh_args = vec!["-N".to_string()];
        let (ssh_args, _) = build_ssh_args(&default_config_path(), &args, &[]);

        // The host must come before the extra args
        let host_pos = ssh_args.iter().position(|a| a == "myhost").unwrap();
        let extra_pos = ssh_args.iter().position(|a| a == "-N").unwrap();
        assert!(host_pos < extra_pos);
    }
}
