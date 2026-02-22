use anyhow::Result;
use crate::cli::{Cli, StatusArgs};

pub fn execute(_cli: &Cli, _args: &StatusArgs) -> Result<()> {
    println!("TODO: status command");
    Ok(())
}
