use clap::{command, Command};

fn main() {
    let mut cmd = command!()
        .subcommand(Command::new("get-apikey").about("Get test server APIKEY."))
        .subcommand(Command::new("start-server").about("Start test server"))
        .subcommand(Command::new("shutdown-server").about("Shutdown test server"));

    match cmd.get_matches_mut().subcommand() {
        Some(("get-apikey", _)) => {
            println!("get-apikey");
        }
        Some(("start-server", _)) => {
            println!("start-server");
        }
        Some(("shutdown-server", _)) => {
            println!("shutdown!");
        }
        _ => {
            cmd.print_help().unwrap();
        }
    }
}
