use assert_cmd::Command;
use predicates::prelude::*;
#[test]
fn test_no_args_shows_help() {
    Command::cargo_bin("sshr")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}
#[test]
fn test_help_flag() {
    Command::cargo_bin("sshr")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("SSH connection manager"));
}
#[test]
fn test_version_flag() {
    Command::cargo_bin("sshr")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("sshr"));
}
#[test]
fn test_list_alias() {
    // "ls" debería funcionar igual que "list"
    Command::cargo_bin("sshr")
        .unwrap()
        .arg("ls")
        .assert()
        .success();
}
#[test]
fn test_unknown_subcommand() {
    Command::cargo_bin("sshr")
        .unwrap()
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
#[test]
fn test_custom_config_path() {
    Command::cargo_bin("sshr")
        .unwrap()
        .args(["-F", "/tmp/test_ssh_config", "list"])
        .assert();
    // Verificar que usa el fichero especificado
}
