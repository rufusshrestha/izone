// izone/src/models.rs

use serde::{Deserialize, Serialize};
use crate::helpers::deserialize_int_as_bool;

// Nested structs for SystemV2
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct UnitOpt {
    #[serde(rename = "RA", default)]
    pub ra: u8,
    #[serde(default)]
    pub master: u8,
    #[serde(default)]
    pub zones: u8,
    #[serde(default)]
    pub history: u8,
    #[serde(rename = "SlaveOpt", default)]
    pub slave_opt: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Temperzone {
    #[serde(default)]
    pub heat_setpoint: u32,
    #[serde(default)]
    pub cool_setpoint: u32,
    #[serde(default)]
    pub fan_type: u8,
    #[serde(default)]
    pub mode_type: u8,
    #[serde(default)]
    pub quiet: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct GasHeat {
    #[serde(rename = "Type", default)]
    pub gas_type: u8,
    #[serde(default)]
    pub min_run_time: u8,
    #[serde(default)]
    pub anticycle_time: u8,
    #[serde(default)]
    pub stage_offset: u8,
    #[serde(default)]
    pub stage_delay: u8,
    #[serde(default)]
    pub cycle_fan_cool: u8,
    #[serde(default)]
    pub cycle_fan_heat: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Ventilation {
    #[serde(default)]
    pub rh_setpoint: u8,
    #[serde(default)]
    pub vocs_setpoint: u16,
    #[serde(default)]
    pub eco2_setpoint: u16,
    #[serde(default)]
    pub fan_stage_delay: u8,
    #[serde(default)]
    pub cycle_fan_off: u8,
    #[serde(default)]
    pub use_rh_control: u8,
    #[serde(default)]
    pub use_vcos_control: u8,
    #[serde(default)]
    pub use_eco2_control: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Coolbreeze {
    #[serde(default)]
    pub fan_speed: u8,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub rh_set: u8,
    #[serde(default)]
    pub rh_read: u8,
    #[serde(default)]
    pub fan_run_h: u32,
    #[serde(default)]
    pub pump_run_h: u32,
    #[serde(default)]
    pub prew_en: u8,
    #[serde(default)]
    pub prew_time: u8,
    #[serde(default)]
    pub dr_af_prew_en: u8,
    #[serde(default)]
    pub dr_cyc_en: u8,
    #[serde(default)]
    pub dr_cyc_per: u16,
    #[serde(default)]
    pub postw_en: u8,
    #[serde(default)]
    pub dr_bf_postw_en: u8,
    #[serde(rename = "PostwT", default)]
    pub postw_t: u8,
    #[serde(default)]
    pub inverter: u8,
    #[serde(default)]
    pub resume_last: u8,
    #[serde(default)]
    pub fan_max_auto: u8,
    #[serde(default)]
    pub fan_max: u8,
    #[serde(default)]
    pub exh_max: u8,
    #[serde(default)]
    pub exh_en: u8,
    #[serde(default)]
    pub ctrl_sens: u8,
    #[serde(default)]
    pub calib_temp: i16,
    #[serde(default)]
    pub dead_temp: u16,
    #[serde(default)]
    pub auto_fan_max_time: u8,
}

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
    #[serde(default)]
    pub sleep_timer: u32,
    #[serde(default)]
    pub supply: u32,
    #[serde(rename = "RAS", default)]
    pub ras: u8,
    #[serde(default)]
    pub ctrl_zone: u8,
    #[serde(default)]
    pub tag1: String,
    #[serde(default)]
    pub tag2: String,
    #[serde(default)]
    pub warnings: String,
    #[serde(default)]
    pub eco_lock: u8,
    #[serde(default)]
    pub eco_max: u32,
    #[serde(default)]
    pub eco_min: u32,
    #[serde(default)]
    pub no_of_const: u8,
    #[serde(default)]
    pub no_of_zones: u8,
    #[serde(default)]
    pub sys_type: u16,
    #[serde(rename = "iSaveEnable", default)]
    pub isave_enable: u8,
    #[serde(rename = "iSaveOn", default)]
    pub isave_on: u8,
    #[serde(default)]
    pub lock_code: String,
    #[serde(default)]
    pub lock_status: u8,
    #[serde(default)]
    pub lock_on: u8,
    #[serde(default)]
    pub fan_auto_en: u8,
    #[serde(default)]
    pub fan_auto_type: u8,
    #[serde(default)]
    pub fan_capacity: u16,
    #[serde(default)]
    pub fan_unit_capacity: u16,
    #[serde(default)]
    pub filter_warn: u8,
    #[serde(rename = "iZoneOnOff", default)]
    pub izone_on_off: u8,
    #[serde(rename = "iZoneMode", default)]
    pub izone_mode: u8,
    #[serde(rename = "iZoneFan", default)]
    pub izone_fan: u8,
    #[serde(rename = "iZoneSetpoint", default)]
    pub izone_setpoint: u8,
    #[serde(default)]
    pub ext_on_off: u8,
    #[serde(default)]
    pub ext_mode: u8,
    #[serde(default)]
    pub ext_fan: u8,
    #[serde(default)]
    pub ext_setpoint: u8,
    #[serde(default)]
    pub damper_time: u8,
    #[serde(default)]
    pub auto_off: u8,
    #[serde(default)]
    pub room_temp_disp: u8,
    #[serde(default)]
    pub rf_ch: u8,
    #[serde(default)]
    pub auto_mode_dead_b: u16,
    #[serde(default)]
    pub wired_leds: u8,
    #[serde(default)]
    pub airflow_lock: u8,
    #[serde(default)]
    pub airflow_min_lock: u8,
    #[serde(default)]
    pub out_of_view_ras: u8,
    #[serde(default)]
    pub cpu_type: u8,
    #[serde(default)]
    pub sys_no: u8,
    #[serde(default)]
    pub ac_unit_brand: u8,
    #[serde(default)]
    pub ac_unit_brand_set: u8,
    #[serde(default)]
    pub oem_make: u8,
    #[serde(default)]
    pub hide_induct: u8,
    #[serde(default)]
    pub air_pure_on: u8,
    #[serde(default)]
    pub reverse_dampers: u8,
    #[serde(default)]
    pub scrooge: u8,
    #[serde(default)]
    pub pass: String,
    #[serde(default)]
    pub cnst_ctrl_area_en: u8,
    #[serde(default)]
    pub cnst_ctrl_area: u16,
    #[serde(default)]
    pub static_p: u8,
    #[serde(default)]
    pub open_dampers_when_off: u8,
    #[serde(default)]
    pub show_act_temps: u8,
    #[serde(default)]
    pub use_induct_energy: u8,
    #[serde(default)]
    pub unit_opt: UnitOpt,
    #[serde(default)]
    pub temperzone: Temperzone,
    #[serde(default)]
    pub gas_heat: GasHeat,
    #[serde(default)]
    pub ventilation: Ventilation,
    #[serde(default)]
    pub coolbreeze: Coolbreeze,
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