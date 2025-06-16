// izone/src/models.rs

use serde::{Deserialize, Serialize};
use crate::helpers::deserialize_int_as_bool; // <--- This import is correct

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
#[serde(rename_all = "PascalCase")]
pub struct SystemV2 {
    #[serde(deserialize_with = "deserialize_int_as_bool")] // <--- This line is important
    pub sys_on: bool,
    pub sys_mode: u8,
    pub temp: u32,
    pub setpoint: u32,
    pub sys_fan: u8,
    #[serde(rename = "ACError")]
    pub ac_error: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
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
    #[serde(default)]
    #[serde(rename = "Const")]
    pub constant: u8,
    #[serde(rename = "ConstA")]
    pub constant_a: u8,
    pub master: u8,
    #[serde(rename = "DmpFlt")]
    pub damper_fault: u8,
    #[serde(rename = "SnsFlt", default)] // Made optional by adding default
    pub sensor_fault: u8, // Will default to 0 if missing
    #[serde(rename = "DmpSkip", default)] // Made optional by adding default
    pub damper_skip: u8, // Will default to 0 if missing
    #[serde(default)] // Made optional by adding default
    pub isense: u8, // Will default to 0 if missing
    pub calibration: u8,
    #[serde(rename = "RFSig", default)] // Made optional by adding default
    pub rf_signal: u8, // Will default to 0 if missing
    #[serde(rename = "BattVolt")]
    pub batt_volt: u8,
    pub area: u32,
    pub bypass: u8,
    #[serde(rename = "BalanceMax")]
    pub balance_max: u8,
    #[serde(rename = "BalanceMin")]
    pub balance_min: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ZonesV2Response {
    #[serde(rename = "ZonesV2")]
    pub zones_v2: ZonesV2,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ZoneListV2Response {
    #[serde(rename = "ZoneListV2")]
    pub zone_list_v2: Vec<ZonesV2>,
}