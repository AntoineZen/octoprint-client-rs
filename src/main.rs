use anyhow::{anyhow, Context, Result};
use clap::{arg, command, Arg, Command};
use confy;
use console::Style;
use dialoguer::Input;
use time_humanize::HumanTime;

mod octoprintclient;
use octoprintclient::{Configuration, OctoPrintClient};

#[tokio::main]
async fn main() -> Result<()> {
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
        return match OctoPrintClient::from_config(new_config.clone())
            .get_server_info()
            .await
        {
            Ok(info) => {
                println!("Connected to Octoprint version {}.", info.version);
                confy::store("octoprint-client", new_config).expect("Failed to save configuration");
                Ok(())
            }
            Err(e) => Err(anyhow!("Connection failed: {}", e)),
        };
    }

    // Parse command line
    let matches = command!()
        .subcommand(
            Command::new("upload")
                .about("Upload a file to Octoprint instance")
                .arg(Arg::new("dir").short('d').help("Specify upload dir"))
                .arg(Arg::new("file").required(true).help("File to upload")),
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
            let file_name = sub_matches.value_of("file").unwrap();
            let file = std::fs::File::open(file_name)?;
            opc.upload(file).await
        }
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
                "Extruder : {}°C / {}°C",
                temperature_data.actual, temperature_data.target
            );
        }
        if let Some(temperature_data) = temperature_state.bed {
            println!(
                "Bed      : {}°C / {}°C",
                temperature_data.actual, temperature_data.target
            );
        }
    }

    Ok(())
}
