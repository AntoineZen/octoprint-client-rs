use serde_derive::{Deserialize, Serialize};
use std::io::{Read, Write};

use hyper;
use hyper::body::Buf;
use hyper::{Body, Client, Method, Request, Response, StatusCode};

// type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

use thiserror::Error;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub server_url: String,
    pub api_key: String,
}

#[derive(Debug)]
pub struct OctoPrintClient {
    config: Configuration,
}

#[derive(Deserialize, Debug)]
pub struct File {
    pub date: Option<u64>,
    pub display: Option<String>,
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

#[derive(Deserialize, Debug)]
pub struct TemperatureState {
    pub tool0: Option<TemperatureData>,
    pub bed: Option<TemperatureData>,
}

#[derive(Deserialize, Debug)]
pub struct TemperatureData {
    pub actual: f32,
    pub target: f32,
    pub offset: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct SDState {
    pub ready: bool,
}

#[derive(Deserialize, Debug)]
pub struct PrinterState {
    text: String,
    error: Option<String>,
    flags: PrinterFlags,
}

#[derive(Deserialize, Debug)]
pub struct PrinterFlags {
    pub operational: bool,
    pub paused: bool,
    pub printing: bool,
    pub pausing: bool,
    pub cancelling: bool,
    #[serde(rename = "sdReady")]
    pub sd_ready: bool,
    pub error: bool,
    pub ready: bool,
    #[serde(rename = "closedOrError")]
    pub closed_on_error: bool,
}
#[derive(Deserialize, Debug)]
pub struct PrinterInfo {
    pub temperature: Option<TemperatureState>,
    pub sd: Option<SDState>,
    pub state: Option<PrinterState>,
}

// TODO: Use something geretated randomly for each request.
const BONDARY: &'static str = "----WebKitFormBoundaryNhILabgMzjj9z3Io";

#[derive(Error, Debug)]
pub enum OctoPrintClientError {
    #[error("Server Error")]
    ServerError(String),
    #[error("Client Error")]
    ClientError(#[from] hyper::Error),
    #[error("HTTP Error")]
    HttpError(#[from] hyper::http::Error),
    #[error("JSON decode Error")]
    JSONDecodeError(#[from] serde_json::Error),
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
}

impl OctoPrintClient {
    pub fn from_config(config: Configuration) -> Self {
        OctoPrintClient { config }
    }

    async fn fetch_url(&self, endpoint: &str) -> Result<Response<Body>, OctoPrintClientError> {
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
            return Err(OctoPrintClientError::ServerError(error_msg.error));
        }

        Ok(resp)
    }

    pub async fn get_current_job(&self) -> Result<JobInformation, OctoPrintClientError> {
        let mut resp = self.fetch_url("job").await?;
        let json_doc = hyper::body::aggregate(resp.body_mut()).await?;

        Ok(serde_json::from_reader(json_doc.reader())?)
    }

    pub async fn get_server_info(&self) -> Result<ServerInfo, OctoPrintClientError> {
        let mut resp = self.fetch_url("server").await?;
        let json_doc = hyper::body::aggregate(resp.body_mut()).await?;

        Ok(serde_json::from_reader(json_doc.reader())?)
    }

    pub async fn get_printer_state(&self) -> Result<PrinterInfo, OctoPrintClientError> {
        let mut resp = self.fetch_url("printer").await?;
        let json_doc = hyper::body::aggregate(resp.body_mut()).await?;

        Ok(serde_json::from_reader(json_doc.reader())?)
    }

    pub async fn upload(
        &self,
        mut file: std::fs::File,
        file_name: &str,
    ) -> Result<(), OctoPrintClientError> {
        let mut payload = Vec::new();

        write!(payload, "--{}\r\n", BONDARY)?;
        write!(
            payload,
            "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n",
            file_name
        )?;
        write!(payload, "Content-Type: text/x.gcode\r\n")?;
        write!(payload, "\r\n")?;
        file.read_to_end(&mut payload)?;
        write!(payload, "\r\n")?;
        write!(payload, "--{}\r\n", BONDARY)?;
        write!(
            payload,
            "Content-Disposition: form-data; name=\"select\"\r\n"
        )?;
        write!(payload, "\r\n")?;
        write!(payload, "true\r\n")?;
        write!(payload, "--{}\r\n", BONDARY)?;
        write!(
            payload,
            "Content-Disposition: form-data; name=\"print\"\r\n"
        )?;
        write!(payload, "\r\n")?;
        write!(payload, "false\r\n")?;
        write!(payload, "--{}--\r\n", BONDARY)?;

        let length = payload.len();

        let req = Request::builder()
            .method(Method::POST)
            .uri(self.config.server_url.clone() + "/api/files/local")
            .header("X-Api-Key", &self.config.api_key)
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={}", BONDARY),
            )
            .header("Content-Length", length)
            .body(Body::from(payload))?;

        let client = Client::new();
        let mut resp = client.request(req).await?;
        if resp.status() != StatusCode::CREATED {
            eprintln!(
                "{}",
                hyper::body::aggregate(resp.body_mut()).await?.remaining()
            );
            return Err(OctoPrintClientError::ServerError("Server error".into()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn get_client() -> OctoPrintClient {
        /*let c = Configuration {
            api_key: "".to_string(),
            server_url: "".to_string()
        };*/

        OctoPrintClient::from_config(confy::load("octoprint-client").unwrap())
    }

    fn get_client_with_wrong_url() -> OctoPrintClient {
        let c = Configuration {
            api_key: "38863B6406FC4C1299E1974FAC6842B4".to_string(),
            server_url: "http://idontexist.org".to_string(),
        };

        OctoPrintClient::from_config(c)
    }

    fn get_client_with_wrong_api_key() -> OctoPrintClient {
        let c = Configuration {
            api_key: "abdcasdfasfdasf".to_string(),
            server_url: "http://localhost".to_string(),
        };

        OctoPrintClient::from_config(c)
    }

    #[tokio::test]
    pub async fn test_get_server_info() {
        let c = get_client();

        //println!("{:?}", c);

        let info = c.get_server_info().await.unwrap();

        assert_eq!(info.version, "1.7.3");
        assert_eq!(info.safemode, None);
    }

    #[tokio::test]
    pub async fn test_false_url() {
        let c = get_client_with_wrong_url();
        let info_result = c.get_server_info().await;
        println!("{:?}", info_result);

        assert!(info_result.is_err());
    }

    #[tokio::test]
    pub async fn test_false_apikey() {
        let c = get_client_with_wrong_api_key();
        let info_result = c.get_server_info().await;
        println!("{:?}", info_result);

        assert!(info_result.is_err());
    }

    #[tokio::test]
    pub async fn test_get_printer_state() {
        let c = get_client();

        c.get_printer_state().await.unwrap();
    }

    #[tokio::test]
    pub async fn test_get_current_job() {
        let c = get_client();

        c.get_current_job().await.unwrap();
    }
}
