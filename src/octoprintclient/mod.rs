use serde_derive::{Deserialize, Serialize};
use std::io::{Read, Write};

use hyper;
use hyper::body::Buf;
use hyper::{Body, Client, Method, Request, Response, StatusCode};

pub mod datamodel;

use self::datamodel::*;

// type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

use thiserror::Error;

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

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub server_url: String,
    pub api_key: String,
}

#[derive(Debug)]
pub struct OctoPrintClient {
    config: Configuration,
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

    pub async fn get_connection(&self) -> Result<PrinterConnection, OctoPrintClientError> {
        let mut resp = self.fetch_url("connection").await?;
        let json_doc = hyper::body::aggregate(resp.body_mut()).await?;

        Ok(serde_json::from_reader(json_doc.reader())?)
    }

    pub async fn connect(&self, cmd: &ConnectionCommand) -> Result<(), OctoPrintClientError> {
        let req = Request::builder()
            .method(Method::POST)
            .uri(self.config.server_url.clone() + "/api/connection")
            .header("X-Api-Key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(cmd)?))?;

        let client = Client::new();
        let mut resp = client.request(req).await?;
        if resp.status() != StatusCode::NO_CONTENT {
            eprintln!(
                "{}",
                hyper::body::aggregate(resp.body_mut()).await?.remaining()
            );
            return Err(OctoPrintClientError::ServerError("Server error".into()));
        }
        Ok(())
    }

    pub async fn connect_default(&self) -> Result<(), OctoPrintClientError> {
        let conn_state = self.get_connection().await?;
        let br =
            conn_state
                .options
                .baudrate_preference
                .ok_or(OctoPrintClientError::ServerError(
                    "No default baudrate stored in server".to_string(),
                ))?;
        let port = conn_state
            .options
            .port_preference
            .ok_or(OctoPrintClientError::ServerError(
                "No default port stored in server".to_string(),
            ))?;

        let profile = conn_state.options.printer_profile_preference.ok_or(
            OctoPrintClientError::ServerError("No default port stored in server".to_string()),
        )?;

        let connect_cmd = ConnectionCommand {
            command: "connect".to_string(),
            port: Some(port),
            baudrate: Some(br),
            printer_profile: Some(profile), //Some("_default".to_string()),
            save: Some(true),
            autoconnect: Some(false),
        };

        let req = Request::builder()
            .method(Method::POST)
            .uri(self.config.server_url.clone() + "/api/connection")
            .header("X-Api-Key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&connect_cmd)?))?;

        let client = Client::new();
        let mut resp = client.request(req).await?;
        if resp.status() != StatusCode::NO_CONTENT {
            eprintln!(
                "{}",
                hyper::body::aggregate(resp.body_mut()).await?.remaining()
            );
            return Err(OctoPrintClientError::ServerError("Server error".into()));
        }
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), OctoPrintClientError> {
        let req = Request::builder()
            .method(Method::POST)
            .uri(self.config.server_url.clone() + "/api/connection")
            .header("X-Api-Key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(
                &DisconnectCommand::default(),
            )?))?;

        let client = Client::new();
        let mut resp = client.request(req).await?;
        if resp.status() != StatusCode::NO_CONTENT {
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
    use std::process::Command;

    fn get_apikey() -> String {
        let buffer = Command::new("./tests/get-apikey.sh")
            .output()
            .unwrap()
            .stdout;

        let raw_api_key = String::from_utf8(buffer).unwrap();
        let clean_api_key = raw_api_key
            .strip_suffix("\n")
            .unwrap_or(raw_api_key.as_str());

        clean_api_key.into()
    }

    fn get_client() -> OctoPrintClient {
        let c = Configuration {
            api_key: get_apikey(),
            server_url: "http://localhost".to_string(),
        };
        println!("Config: {:?}", c);

        //OctoPrintClient::from_config(confy::load("octoprint-client").unwrap())
        OctoPrintClient::from_config(c)
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

        assert_eq!(info.version, "1.8.6");
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

        c.connect_default().await.unwrap();

        c.get_printer_state().await.unwrap();
    }

    #[tokio::test]
    pub async fn test_get_current_job() {
        let c = get_client();

        c.get_current_job().await.unwrap();
    }

    #[tokio::test]
    pub async fn test_connect_disconnect() {
        let c = get_client();

        let connection_info = c.get_connection().await.unwrap();

        let connect_cmd = ConnectionCommand {
            command: "connect".to_string(),
            port: Some("VIRTUAL".to_string()),
            baudrate: Some(115200),
            printer_profile: Some(connection_info.options.printer_profile_preference).unwrap(), //Some("_default".to_string()),
            save: Some(true),
            autoconnect: Some(false),
        };

        c.connect(&connect_cmd).await.unwrap();

        std::thread::sleep(std::time::Duration::from_secs(2));

        let connection_info = c.get_connection().await.unwrap();

        assert_eq!(connection_info.current.state, "Operational");

        c.disconnect().await.unwrap();

        std::thread::sleep(std::time::Duration::from_secs(2));

        let connection_info = c.get_connection().await.unwrap();

        println!("connection info : {:?}", connection_info);

        assert_eq!(connection_info.current.state, "Closed");
    }
}
