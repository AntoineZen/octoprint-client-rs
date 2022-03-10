use confy;
use dialoguer::Input;
use serde_derive::{Deserialize, Serialize};

use hyper::{Client, Request, Method, Body};
use hyper::body::HttpBody;

#[derive(Default, Debug, Serialize, Deserialize)]
struct Configuration {
    server_url: String,
    api_key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cfg: Configuration = confy::load("octoprint-client").expect("Configuration loading failed");
    dbg!(&cfg);

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


    let req = Request::builder()
        .method(Method::GET)
        .uri(cfg.server_url+"/api/server")
        .header("X-Api-Key", cfg.api_key)
        .body(Body::empty())
        .expect("Failed to create request");

    let client = Client::new();
    let mut resp = client.request(req).await.expect("Request failed");
    dbg!(resp.body_mut().data().await.unwrap());

    Ok(())
}
