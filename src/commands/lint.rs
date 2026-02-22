use anyhow::Result;
use crate::cli::{Cli, LintArgs};

pub fn execute(_cli: &Cli, _args: &LintArgs) -> Result<()> {
    println!("TODO: lint command");
    Ok(())
}
