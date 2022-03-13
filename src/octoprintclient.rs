use serde_derive::{Deserialize, Serialize};

use hyper;
use hyper::body::{Buf, Bytes};
use hyper::{Body, Client, Method, Request};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub server_url: String,
    pub api_key: String,
}

pub struct OctoPrintClient {
    config: Configuration,
}

impl OctoPrintClient {
    pub fn from_config(config: Configuration) -> Self {
        OctoPrintClient { config }
    }

    pub async fn fetch_url(&self, endpoint: &str) -> Bytes {
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
