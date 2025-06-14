// izone/src/models.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SystemV2 {
    #[serde(default)] // Default to false if not present
    pub sys_on: bool,
    pub sys_mode: u8,
    pub temp: u32,
    pub setpoint: u32,
    pub sys_fan: u8,
    #[serde(rename = "ACError")]
    pub ac_error: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SystemV2Response {
    #[serde(rename = "SystemV2")]
    pub system_v2: SystemV2,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ZonesV2 {
    pub name: String,
    pub mode: u8,
    pub setpoint: u32,
    pub temp: u32,
    #[serde(rename = "DmpPos")]
    pub damper_pos: u8,
    pub zone_type: u8,
    pub sens_type: u8,
    pub max_air: u8,
    pub min_air: u8,
    #[serde(default)] // Default to 0 if not present
    #[serde(rename = "Const")]
    pub constant: u8,
    #[serde(rename = "ConstA")]
    pub constant_a: u8,
    pub master: u8,
    #[serde(rename = "DmpFlt")]
    pub damper_fault: u8,
    #[serde(rename = "iSense")]
    pub isense: u8,
    pub area: u32,
    pub calibration: u8,
    pub bypass: u8,
    pub rf_signal: u8,
    pub batt_volt: u8,
    pub sensor_fault: u8,
    pub balance_max: u8,
    pub balance_min: u8,
    pub damper_skip: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ZonesV2Response {
    #[serde(rename = "ZonesV2")]
    pub zones_v2: ZonesV2,
}