use anyhow::{Context, Result};
use confy;
use dialoguer::Input;

mod octoprintclient;
use octoprintclient::{Configuration, OctoPrintClient};

#[tokio::main]
async fn main() -> Result<()> {
    let cfg: Configuration = confy::load("octoprint-client").expect("Configuration loading failed");
    //dbg!(&cfg);

    if cfg.server_url.is_empty() {
        println!("Configuration is empty, let's fix that");

        let mut new_config = Configuration {
            server_url: "".to_string(),
            api_key: "".to_string(),
        };
        new_config.server_url = Input::new()
            .with_prompt("Server URL")
            .interact_text()
            .expect("Bad input!");

        new_config.api_key = Input::new()
            .with_prompt("API Key")
            .interact_text()
            .expect("Bad input!");

        confy::store("octoprint-client", new_config).expect("Failed to save configuration");
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

    if let Some(completion) = job.progress.completion {
        println!("Progress: {:2.1}% done", completion);
    }

    println!("State : \"{}\"", job.state);

    if let Some(err) = job.error {
        eprintln!("ERROR: {}", err);
    }

    if let Some(path) = job.job.file.path {
        println!("File : {}", path);
    }

    Ok(())
}
