use anyhow::{anyhow, Context, Result};
use clap::{command, value_parser, Arg, ArgAction, Command};
use confy;
use console::Style;
use dialoguer::Input;
use time_humanize::HumanTime;

mod octoprintclient;
use octoprintclient::datamodel::ConnectionCommand;
use octoprintclient::{Configuration, OctoPrintClient};

async fn get_configuration() -> Result<Configuration> {
    // Try to get configuration using "confy"
    let cfg: Configuration =
        confy::load("octoprint-client").context("Configuration loading failed")?;
    //dbg!(&cfg);

    // if configuration seems emply (i.e. no server URL is configured
    if cfg.server_url.is_empty() {
        // Ask the user to create one
        println!("Configuration is empty, let's fix that...");

        let mut new_config = Configuration {
            server_url: "".to_string(),
            api_key: "".to_string(),
        };
        new_config.server_url = Input::new().with_prompt("Server URL").interact_text()?;

        new_config.api_key = Input::new().with_prompt("API Key").interact_text()?;

        // Test configuration by getting server info.
        match OctoPrintClient::from_config(new_config.clone())
            .get_server_info()
            .await
        {
            Ok(info) => {
                println!("Connected to Octoprint version {}.", info.version);
                confy::store("octoprint-client", new_config).expect("Failed to save configuration");
                Ok(cfg)
            }
            Err(e) => Err(anyhow!("Connection failed: {}", e)),
        }
    } else {
        Ok(cfg)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Try to get configuration using "confy"
    let cfg = get_configuration().await?;

    // Parse command line
    let matches = command!()
        .subcommand(
            Command::new("upload")
                .about("Upload a file to Octoprint instance")
                .arg(Arg::new("dir").short('d').help("Specify upload dir"))
                .arg(Arg::new("file").required(true).help("File to upload")),
        )
        .subcommand(Command::new("connection").about("Print printer connection state"))
        .subcommand(
            Command::new("connect")
                .about("Connect to printer (open serial connection)")
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .help("Specify serial port"),
                )
                .arg(
                    Arg::new("baudrate")
                        .short('b')
                        .long("baudrate")
                        .help("Set baudrate")
                        .value_parser(value_parser!(u32))
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("profile")
                        .short('t')
                        .long("profile")
                        .help("Select profile (must exists)"),
                ),
        )
        .subcommand(
            Command::new("disconnect").about("Disconnect from printer (close serial connection)"),
        )
        .get_matches();

    // Create the client object
    let opc = OctoPrintClient::from_config(cfg);

    let server = opc
        .get_server_info()
        .await
        .with_context(|| "Get server info")?;
    println!("Connected to Octoprint version {}", server.version);

    match matches.subcommand() {
        Some(("upload", sub_matches)) => {
            let file_name = sub_matches
                .get_one::<String>("file")
                .ok_or(anyhow!("Bad file name given"))?;
            println!("Uploading \"{}\"", file_name);
            let file = std::fs::File::open(file_name)?;
            opc.upload(file, file_name).await.with_context(|| "Upload")
        }
        Some(("connection", _)) => print_connection(opc).await,
        Some(("connect", sub_match)) => {
            let conn_state = opc.get_connection().await?;
            let br: u32 = if let Some(baudrate) = sub_match.get_one::<u32>("baudrate") {
                *baudrate
            } else {
                conn_state
                    .options
                    .baudrate_preference
                    .ok_or(anyhow!("No default baudrate stored in server"))?
            };
            let port = if let Some(port) = sub_match.get_one::<String>("port") {
                port.to_string()
            } else {
                conn_state
                    .options
                    .port_preference
                    .ok_or(anyhow!("No default port stored in server"))?
            };

            let profile = if let Some(profile) = sub_match.get_one::<String>("profile") {
                profile.to_string()
            } else {
                conn_state
                    .options
                    .printer_profile_preference
                    .ok_or(anyhow!("No default port stored in server"))?
            };

            let connect_cmd = ConnectionCommand {
                command: "connect".to_string(),
                port: Some(port),
                baudrate: Some(br),
                printer_profile: Some(profile), //Some("_default".to_string()),
                save: Some(true),
                autoconnect: Some(false),
            };
            opc.connect(&connect_cmd).await.with_context(|| "Connect")
        }
        Some(("disconnect", _)) => opc.disconnect().await.with_context(|| "Disconnect"),
        _ => print_state(opc).await,
    }
}

async fn print_state(opc: OctoPrintClient) -> Result<()> {
    // Get jom information from the server.
    let job = opc
        .get_current_job()
        .await
        .with_context(|| "Getting job state")?;

    //dbg!(&job);

    // Print state
    let style = if job.state.to_lowercase().contains("error") {
        Style::new().red().bold()
    } else if job.state.to_lowercase().contains("printing") {
        Style::new().green().bold()
    } else {
        Style::new().yellow()
    };
    println!("State    : {}", style.apply_to(job.state));

    // Print progress and estimate end time
    if let (Some(completion), Some(time_left)) =
        (job.progress.completion, job.progress.print_time_left)
    {
        println!(
            "Progress : {:2.1}% , ends {}",
            completion,
            HumanTime::from_seconds(time_left)
        );
    }

    // Print error if reported
    if let Some(err) = job.error {
        eprintln!(
            "{}",
            Style::new()
                .red()
                .bold()
                .apply_to(format!("ERROR: {}", err))
        );
    }

    // Print file name
    if let Some(path) = job.job.file.path {
        println!("File     : {}", path);
    }

    let printer = opc
        .get_printer_state()
        .await
        .with_context(|| "Getting printer state")?;
    if let Some(temperature_state) = printer.temperature {
        if let Some(temperature_data) = temperature_state.tool0 {
            println!(
                "Extruder : {}??C / {}??C",
                temperature_data.actual, temperature_data.target
            );
        }
        if let Some(temperature_data) = temperature_state.bed {
            println!(
                "Bed      : {}??C / {}??C",
                temperature_data.actual, temperature_data.target
            );
        }
    }

    Ok(())
}

async fn print_connection(opc: OctoPrintClient) -> Result<()> {
    let conn = opc.get_connection().await?;

    println!("Connection State : {}", conn.current.state);
    for profile in &conn.options.printer_profiles {
        if conn.current.printer_profile == profile.id {
            println!("Current Profile : {}", profile.name);
        }
    }

    if let Some(port) = conn.current.port {
        println!("Port : {}", port);
        if let Some(baudrate) = conn.current.baudrate {
            println!("Baudrate : {}", baudrate);
        }
    } else {
        println!("Available ports: ");
        for p in conn.options.ports {
            println!(" - {}", p);
        }
        println!("Available baudrates: ");
        for br in conn.options.baudrates {
            println!(" - {}", br);
        }

        println!("Available profiles: ");
        for profile in conn.options.printer_profiles {
            println!(" - {}", profile.name);
        }
    }
    Ok(())
}
