#![allow(dead_code)]

use serde_derive::{Deserialize, Serialize};

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

#[derive(Deserialize, Debug)]
pub struct CurrentConnection {
    pub baudrate: Option<u32>,
    pub port: Option<String>,
    #[serde(rename = "printerProfile")]
    pub printer_profile: String,
    pub state: String,
}

#[derive(Deserialize, Debug)]
pub struct PrinterProfile {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct ConnectionOptions {
    #[serde(rename = "baudratePreference")]
    pub baudrate_preference: Option<u32>,
    pub baudrates: Vec<u32>,
    #[serde(rename = "portPreference")]
    pub port_preference: Option<String>,
    pub ports: Vec<String>,
    #[serde(rename = "printerProfilePreference")]
    pub printer_profile_preference: Option<String>,
    #[serde(rename = "printerProfiles")]
    pub printer_profiles: Vec<PrinterProfile>,
}

#[derive(Deserialize, Debug)]
pub struct PrinterConnection {
    pub current: CurrentConnection,
    pub options: ConnectionOptions,
}

#[derive(Serialize, Debug)]

pub struct ConnectionCommand {
    pub command: String,
    pub port: Option<String>,
    pub baudrate: Option<u32>,
    #[serde(rename = "printerProfile")]
    pub printer_profile: Option<String>,
    pub save: Option<bool>,
    pub autoconnect: Option<bool>,
}

#[derive(Serialize, Debug)]
pub struct DisconnectCommand {
    pub command: String,
}

impl DisconnectCommand {
    pub fn default() -> Self {
        DisconnectCommand {
            command: "disconnect".to_string(),
        }
    }
}
