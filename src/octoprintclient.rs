use serde_derive::{Deserialize, Serialize};

use hyper;
use hyper::body::Buf;
use hyper::client::ResponseFuture;
use hyper::{Body, Client, Method, Request, Response, StatusCode};

// type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
use anyhow::{anyhow, Result};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub server_url: String,
    pub api_key: String,
}

pub struct OctoPrintClient {
    config: Configuration,
}

#[derive(Deserialize, Debug)]
pub struct File {
    pub date: Option<u64>,
    pub disply: Option<String>,
    pub name: Option<String>,
    pub origin: Option<String>,
    pub path: Option<String>,
    pub size: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct Tool {
    pub length: f32,
    pub volume: f32,
}

#[derive(Deserialize, Debug)]
pub struct Filament {
    pub tool0: Option<Tool>,
}

#[derive(Deserialize, Debug)]
pub struct Job {
    pub file: File,
    #[serde(rename = "estimatedPrintTime")]
    pub estimated_print_time: Option<f32>,
    #[serde(rename = "lastPrintTime")]
    pub last_print_time: Option<f32>,
    pub filament: Option<Filament>,
}

#[derive(Deserialize, Debug)]
pub struct Progress {
    pub completion: Option<f32>,
    pub filepos: Option<u32>,
    #[serde(rename = "printTime")]
    pub print_time: Option<u32>,
    #[serde(rename = "printTimeLeft")]
    pub print_time_left: Option<i64>,
    #[serde(rename = "printTimeLeftOrigin")]
    pub print_time_left_origin: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct JobInformation {
    pub job: Job,
    pub progress: Progress,
    pub state: String,
    pub error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ServerInfo {
    pub version: String,
    pub safemode: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ErrorMsg {
    pub error: String,
}

impl OctoPrintClient {
    pub fn from_config(config: Configuration) -> Self {
        OctoPrintClient { config }
    }

    async fn fetch_url(&self, endpoint: &str) -> Result<Response<Body>> {
        let full_uri = self.config.server_url.clone() + "/api/" + endpoint;
        let req = Request::builder()
            .method(Method::GET)
            .uri(&full_uri)
            .header("X-Api-Key", &self.config.api_key)
            .body(Body::empty())?;

        let client = Client::new();
        let mut resp = client.request(req).await?;
        if resp.status() != StatusCode::OK {
            let error_msg: ErrorMsg =
                serde_json::from_reader(hyper::body::aggregate(resp.body_mut()).await?.reader())?;
            return Err(anyhow!("{}", error_msg.error));
        }

        Ok(resp)
    }

    pub async fn get_current_job(&self) -> Result<JobInformation> {
        let mut resp = self.fetch_url("job").await?;
        let json_doc = hyper::body::aggregate(resp.body_mut()).await?;

        Ok(serde_json::from_reader(json_doc.reader())?)
    }

    pub async fn get_server_info(&self) -> Result<ServerInfo> {
        let mut resp = self.fetch_url("server").await?;

        let json_doc = hyper::body::aggregate(resp.body_mut()).await?;

        Ok(serde_json::from_reader(json_doc.reader())?)
    }
}
