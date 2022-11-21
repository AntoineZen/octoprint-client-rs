use anyhow::{Context, Result};
use clap::{command, Command};
use duct::cmd;

fn main() -> Result<()> {
    let mut cmd = command!()
        .subcommand(Command::new("get-apikey").about("Get test server APIKEY."))
        .subcommand(Command::new("start-server").about("Start test server"))
        .subcommand(Command::new("shutdown-server").about("Shutdown test server"));

    match cmd.get_matches_mut().subcommand() {
        Some(("get-apikey", _)) => {
            let api_key = cmd!("unzip", "-p", "tests/data.zip", "basedir/users.yaml")
                .pipe(cmd!("yq", ".rust.apikey"))
                .read()?;

            println!("{}", api_key);
            Ok(())
        }
        Some(("start-server", _)) => {
            println!("start-server");
            Ok(())
        }
        Some(("shutdown-server", _)) => {
            println!("shutdown!");
            Ok(())
        }
        _ => cmd.print_help().context("Printing help"),
    }
}
