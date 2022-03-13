use confy;
use dialoguer::Input;

mod octoprintclient;
use octoprintclient::{Configuration, OctoPrintClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    dbg!(opc.get_server_info().await);

    dbg!(opc.get_current_job().await);

    Ok(())
}
