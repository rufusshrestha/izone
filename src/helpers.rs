// izone/src/helpers.rs

use colored::Colorize;
use serde::{Deserializer, Deserialize};
use serde::de::Error; // Import Error trait from serde::de

/// Formats temperature from iZone raw (e.g., 2100 -> 21.0)
pub fn format_temp(temp_raw: u32) -> String {
    format!("{:.1}", temp_raw as f32 / 100.0)
}

/// Converts raw system mode integer to human-readable colored text.
pub fn get_colored_system_mode(sys_mode: u8) -> String {
    match sys_mode {
        1 => "Cool".blue().to_string(),
        2 => "Heat".red().to_string(),
        3 => "Vent".white().to_string(),
        4 => "Dry".yellow().to_string(),
        5 => "Auto".cyan().to_string(),
        _ => format!("Mode({})", sys_mode).normal().to_string(),
    }
}

/// Converts human-readable system mode string to its corresponding u8 value.
/// Limited to Auto, Cool, Heat, Vent, Dry.
pub fn get_system_mode_value(mode_name: &str) -> Option<u8> {
    match mode_name.to_lowercase().as_str() {
        "auto" => Some(5), // Auto is 5
        "cool" => Some(1), // Cool is 1
        "heat" => Some(2), // Heat is 2
        "vent" => Some(3), // Vent is 3
        "dry" => Some(4),  // Dry is 4
        _ => None,
    }
}

/// Converts raw fan integer to human-readable text.
pub fn get_fan_speed_text(sys_fan: u8) -> String {
    match sys_fan {
        1 => "Low".to_string(),
        2 => "Medium".to_string(),
        3 => "High".to_string(),
        4 => "Auto".to_string(),
        5 => "Top".to_string(),
        99 => "NonGasHeat".to_string(),
        _ => format!("Fan({})", sys_fan),
    }
}

/// Converts human-readable fan speed string to its corresponding u8 value.
/// Limited to Low, Medium, High, Auto.
pub fn get_fan_speed_value(fan_speed_name: &str) -> Option<u8> {
    match fan_speed_name.to_lowercase().as_str() {
        "low" => Some(1),
        "medium" => Some(2),
        "high" => Some(3),
        "auto" => Some(4),
        _ => None,
    }
}

/// Converts ZoneType_e to human-readable text.
pub fn get_zone_type_text(type_code: u8) -> String {
    match type_code {
        1 => "Open/Close".to_string(),
        2 => "Constant".to_string(),
        3 => "Auto".to_string(),
        _ => format!("Type({})", type_code),
    }
}

/// Returns the raw BatteryLevel_e value, optionally colored red if 0.
pub fn get_battery_level_text(batt_code: u8) -> String {
    if batt_code == 0 {
        batt_code.to_string().red().to_string()
    } else {
        batt_code.to_string().normal().to_string()
    }
}

/// Converts sensor fault code to human-readable colored text.
pub fn get_sensor_fault_text(fault_code: u8) -> String {
    if fault_code == 0 {
        "OK".green().to_string()
    } else {
        format!("FLT:{}", fault_code).red().to_string()
    }
}

/// Custom deserializer for booleans that are represented as 0 or 1 integers.
pub fn deserialize_int_as_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let i = i8::deserialize(deserializer)?;
    match i {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(D::Error::custom(format!("invalid boolean integer: {}", i))),
    }
}

#[allow(dead_code)] // Added this attribute to suppress the warning
pub fn get_visible_length(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        match c {
            '\x1b' => in_escape = true, // Start of an ANSI escape sequence
            'm' if in_escape => in_escape = false, // End of an ANSI escape sequence
            _ if !in_escape => len += 1, // Count only non-escape characters
            _ => {},
        }
    }
    len
}