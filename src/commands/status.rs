use crate::cli::{Cli, StatusArgs};
use anyhow::Result;

pub fn execute(_cli: &Cli, _args: &StatusArgs) -> Result<()> {
    println!("TODO: status command");
    Ok(())
}
