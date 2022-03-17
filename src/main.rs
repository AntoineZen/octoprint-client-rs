use anyhow::{anyhow, Context, Result};
use confy;
use console::Style;
use dialoguer::Input;
use time_humanize::HumanTime;
mod octoprintclient;
use octoprintclient::{Configuration, OctoPrintClient};

#[tokio::main]
async fn main() -> Result<()> {
    let cfg: Configuration =
        confy::load("octoprint-client").context("Configuration loading failed")?;
    //dbg!(&cfg);

    if cfg.server_url.is_empty() {
        println!("Configuration is empty, let's fix that...");

        let mut new_config = Configuration {
            server_url: "".to_string(),
            api_key: "".to_string(),
        };
        new_config.server_url = Input::new().with_prompt("Server URL").interact_text()?;

        new_config.api_key = Input::new().with_prompt("API Key").interact_text()?;

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

    let opc = OctoPrintClient::from_config(cfg);

    let server = opc
        .get_server_info()
        .await
        .with_context(|| "Get server info")?;
    println!("Connected to Octoprint version {}", server.version);

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

    // Printe error if reported
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

    Ok(())
}
