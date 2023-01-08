use anyhow::{Context, Result};
use clap::{command, Command};
use duct::cmd;

fn get_api_key() -> Result<()> {
    let api_key = cmd!("unzip", "-p", "tests/data.zip", "basedir/users.yaml")
        .pipe(cmd!("yq", ".rust.apikey"))
        .read()?;

    println!("{}", api_key);
    Ok(())
}

fn start_server() -> Result<()> {
    cmd!("unzip", "-n", "data.zip")
        .dir("tests")
        .run()
        .context("Fail to unzip data achive")?;
    cmd!("mkdir", "-p", "tests/data/octoprint")
        .run()
        .context("mkdir")?;
    cmd!("bash", "-c", "mv -n tests/basedir/* tests/data/octoprint/")
        .run()
        .context("Fail to move octoprint saved configuration")?;
    cmd!("bash", "-c", "rm -r tests/basedir")
        .run()
        .context("rm basedir")?;
    cmd!("bash", "-c", "rm tests/metadata.json")
        .run()
        .context("rm metadata.json")?;
    cmd!("docker-compose", "up", "-d")
        .dir("tests")
        .run()
        .context("Docker-compose up")?;
    Ok(())
}

fn shutdown_server() -> Result<()> {
    cmd!("docker-compose", "down")
        .dir("tests")
        .run()
        .context("Docker-compose down")?;
    Ok(())
}

fn main() -> Result<()> {
    let mut cmd = command!()
        .subcommand(Command::new("get-apikey").about("Get test server APIKEY."))
        .subcommand(Command::new("start-server").about("Start test server"))
        .subcommand(Command::new("shutdown-server").about("Shutdown test server"));

    match cmd.get_matches_mut().subcommand() {
        Some(("get-apikey", _)) => get_api_key(),
        Some(("start-server", _)) => start_server(),
        Some(("shutdown-server", _)) => shutdown_server(),
        _ => cmd.print_help().context("Printing help"),
    }
}
