use assert_cmd::cargo;
use predicates::prelude::*;
use std::process::Command;

fn sshr() -> Command {
    Command::new(cargo::cargo_bin!("sshr"))
}

#[test]
fn test_no_args_shows_help() {
    assert_cmd::Command::from_std(sshr())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}
#[test]
fn test_help_flag() {
    assert_cmd::Command::from_std(sshr())
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("SSH connection manager"));
}
#[test]
fn test_version_flag() {
    assert_cmd::Command::from_std(sshr())
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("sshr"));
}
#[test]
fn test_list_alias() {
    // "ls" debería funcionar igual que "list"
    assert_cmd::Command::from_std(sshr())
        .arg("ls")
        .assert()
        .success();
}
#[test]
fn test_implicit_connect() {
    // "sshr nonexistent_host" se interpreta como "sshr connect nonexistent_host"
    assert_cmd::Command::from_std(sshr())
        .arg("nonexistent_host")
        .assert()
        .failure()
        .stderr(predicate::str::contains("conectando"));
}
#[test]
fn test_implicit_connect_user_at_host() {
    // "sshr root@host" se interpreta como "sshr connect root@host"
    assert_cmd::Command::from_std(sshr())
        .arg("root@somehost")
        .assert()
        .failure()
        .stderr(predicate::str::contains("conectando"));
}
#[test]
fn test_help_shows_main_help_not_connect() {
    // --help sin subcomando debe mostrar la ayuda principal
    assert_cmd::Command::from_std(sshr())
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("SSH connection manager"));
}
#[test]
fn test_invalid_format_value() {
    // Un valor inválido para --format sigue siendo error de clap
    assert_cmd::Command::from_std(sshr())
        .args(["--format=invalid", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
#[test]
fn test_custom_config_path() {
    assert_cmd::Command::from_std(sshr())
        .args(["-F", "/tmp/test_ssh_config", "list"])
        .assert()
        .success();
}
#[test]
fn test_connect_help() {
    assert_cmd::Command::from_std(sshr())
        .args(["connect", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Connect to an SSH host"));
}
