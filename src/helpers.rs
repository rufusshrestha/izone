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
        6 => "Exhaust".normal().to_string(),
        7 => "Pump Only".normal().to_string(),
        _ => format!("Mode({})", sys_mode).normal().to_string(),
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
        _ => format!("Fan({})", sys_fan).to_string(),
    }
}

/// Converts ZoneType_e to human-readable text.
pub fn get_zone_type_text(type_code: u8) -> String {
    match type_code {
        1 => "Open/Close".to_string(),
        2 => "Constant".to_string(),
        3 => "Auto".to_string(),
        _ => format!("Type({})", type_code).to_string(),
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
            '\x1B' => in_escape = true, // Start of an ANSI escape sequence
            'm' if in_escape => in_escape = false, // End of a common ANSI color sequence
            _ if in_escape => { /* Do nothing, part of escape sequence */ },
            _ => len += c.len_utf8(), // Count visible character length
        }
    }
    len
}