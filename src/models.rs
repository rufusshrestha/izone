// izone/src/models.rs

use serde::{Deserialize, Serialize};
use crate::helpers::deserialize_int_as_bool;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SystemV2 {
    #[serde(deserialize_with = "deserialize_int_as_bool")]
    pub sys_on: bool,
    pub sys_mode: u8,
    pub temp: u32,
    pub setpoint: u32,
    pub sys_fan: u8,
    #[serde(rename = "ACError")]
    pub ac_error: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    #[serde(rename = "SnsFlt", default)]
    pub sensor_fault: u8,
    #[serde(rename = "DmpSkip", default)]
    pub damper_skip: u8,
    #[serde(default)]
    pub isense: u8,
    pub calibration: u8,
    #[serde(rename = "RFSig", default)]
    pub rf_signal: u8,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ScheduleZoneSettings {
    pub mode: u8, // zone mode ZoneMode_e
    pub setpoint: u32, // zone setpoint (1500 - 3000)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ScheduleCoolbreezeSettings {
    #[serde(rename = "UnitSetpoint")]
    pub unit_setpoint: u32,
    #[serde(rename = "FanSpeed")]
    pub fan_speed: u8,
    #[serde(rename = "RhSetpoint")]
    pub rh_setpoint: u8,
}

// New struct to represent the nested "DaysEnabled" object
#[derive(Debug, Serialize, Deserialize, Clone, Default)] // Added Default trait for optional use
#[serde(rename_all = "PascalCase")]
pub struct DaysEnabled {
    #[serde(rename = "M", deserialize_with = "deserialize_int_as_bool", default)]
    pub monday: bool,
    #[serde(rename = "Tu", deserialize_with = "deserialize_int_as_bool", default)]
    pub tuesday: bool,
    #[serde(rename = "W", deserialize_with = "deserialize_int_as_bool", default)]
    pub wednesday: bool,
    #[serde(rename = "Th", deserialize_with = "deserialize_int_as_bool", default)]
    pub thursday: bool,
    #[serde(rename = "F", deserialize_with = "deserialize_int_as_bool", default)]
    pub friday: bool,
    #[serde(rename = "Sa", deserialize_with = "deserialize_int_as_bool", default)]
    pub saturday: bool,
    #[serde(rename = "Su", deserialize_with = "deserialize_int_as_bool", default)]
    pub sunday: bool,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SchedulesV2 {
    #[serde(rename = "AirStreamDeviceUId", default)]
    pub airstream_device_uid: Option<String>,
    pub index: u8,
    pub name: String,
    #[serde(rename = "Enabled", deserialize_with = "deserialize_int_as_bool", default)]
    pub active: bool,
    #[serde(default)]
    pub mode: Option<u8>,
    #[serde(default)]
    pub fan: Option<u8>,
    #[serde(rename = "StartH", default)]
    pub start_h: Option<u8>,
    #[serde(rename = "StartM", default)]
    pub start_m: Option<u8>,
    #[serde(rename = "StopH", default)]
    pub stop_h: Option<u8>,
    #[serde(rename = "StopM", default)]
    pub stop_m: Option<u8>,
    // Removed individual day fields as they are now nested under `days_enabled`
    #[serde(rename = "DaysEnabled", default)] // Map the nested "DaysEnabled" JSON object
    pub days_enabled: DaysEnabled,
    #[serde(rename = "Coolbreeze", default)]
    pub coolbreeze: Option<ScheduleCoolbreezeSettings>,
    #[serde(rename = "Zones", default)]
    pub zones: Option<Vec<ScheduleZoneSettings>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SchedulesResponseWrapper {
    #[serde(rename = "AirStreamDeviceUId", default)]
    pub airstream_device_uid: Option<String>,
    #[serde(rename = "DeviceType", default)]
    pub device_type: Option<String>,
    #[serde(rename = "SchedulesV2")]
    pub schedules_v2: SchedulesV2,
}