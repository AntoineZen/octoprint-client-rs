use confy;
use dialoguer::Input;
use serde_derive::{Deserialize, Serialize};

use hyper;
use hyper::body::{Buf, Bytes};
use hyper::{Body, Client, Method, Request};

#[derive(Default, Debug, Serialize, Deserialize)]
struct Configuration {
    server_url: String,
    api_key: String,
}

struct OctoPrintClient {
    config: Configuration,
}

impl OctoPrintClient {
    fn from_config(config: Configuration) -> Self {
        OctoPrintClient { config }
    }

    async fn fetch_url(&self, endpoint: &str) -> Bytes {
        let full_uri = self.config.server_url.clone() + "/api/" + endpoint;
        let req = Request::builder()
            .method(Method::GET)
            .uri(&full_uri)
            .header("X-Api-Key", &self.config.api_key)
            .body(Body::empty())
            .expect("Failed to create request");

        let client = Client::new();
        let mut resp = client.request(req).await.expect("Request failed");

        let mut json_doc = hyper::body::aggregate(resp.body_mut())
            .await
            .expect("Failed to read answer");

        json_doc.copy_to_bytes(json_doc.remaining())
    }
}

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

    dbg!(opc.fetch_url("job").await);
    dbg!(opc.fetch_url("server").await);

    Ok(())
}
