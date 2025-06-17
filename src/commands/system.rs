// izone/src/commands/system.rs

use reqwest::blocking::Client;
use serde_json::json;
use colored::Colorize;
use std::process::exit;
use stringcase::Caser;
// Corrected imports for API functions and get_visible_length from helpers
use crate::api::{make_query_request, make_command_request};
use crate::constants;
use crate::helpers::{format_temp, get_colored_system_mode, get_fan_speed_text, get_visible_length, get_system_mode_value, get_fan_speed_value};
use crate::models::SystemV2Response;

// Removed: The `print_status_line` helper function has been removed as requested.

pub fn get_system_status(client: &Client) {
    const BOX_WIDTH: usize = 45; // Total width of the box's horizontal lines
    const PADDING_WIDTH: usize = BOX_WIDTH - 2; // Subtract 2 for the '║ ' and ' ║'
    const LABEL_WIDTH: usize = 24; // Width for the labels like "Aircon Power:", "Mode:", etc.

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "AIRCON STATUS", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let query_data = json!({ "iZoneV2Request": { "Type": 1, "No": 0, "No1": 0 } });

    let response_value = match make_query_request(client, query_data) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}", format!("Error querying system status: {}", e).red());
            exit(1);
        }
    };

    let system_response: SystemV2Response = serde_json::from_value(response_value.clone())
        .expect("Failed to parse system status response");

    let sys_v2 = system_response.system_v2;

    let status_text = if sys_v2.sys_on {
        "ON".green().to_string()
    } else {
        "OFF".red().to_string()
    };

    let ac_error_text = if sys_v2.ac_error.trim() == "OK" {
        sys_v2.ac_error.green().to_string()
    } else {
        sys_v2.ac_error.red().to_string()
    };

    // Construct each line with internal padding for label and value
    let sys_on_line = format!("{:width$} {}", "Aircon Power:", status_text, width = LABEL_WIDTH);
    let sys_mode_line = format!("{:width$} {}", "Mode:", get_colored_system_mode(sys_v2.sys_mode), width = LABEL_WIDTH);
    let sys_fan_line = format!("{:width$} {}", "Fan Speed:", get_fan_speed_text(sys_v2.sys_fan).cyan(), width = LABEL_WIDTH);
    let sys_setpoint_line = format!("{:width$} {}°C", "Target Setpoint:", format_temp(sys_v2.setpoint), width = LABEL_WIDTH);
    let sys_temp_line = format!("{:width$} {}°C", "Controller Temperature:", format_temp(sys_v2.temp).cyan(), width = LABEL_WIDTH);
    let ac_error_line = format!("{:width$} {}", "AC Error:", ac_error_text, width = LABEL_WIDTH - 1); // Adjusted width for AC Error

    // Print each line, adjusting the external padding based on the visible length of the formatted line
    println!("║ {:<pw$} ║", sys_on_line, pw = PADDING_WIDTH - get_visible_length(&sys_on_line) + sys_on_line.len());
    println!("║ {:<pw$} ║", sys_mode_line, pw = PADDING_WIDTH - get_visible_length(&sys_mode_line) + sys_mode_line.len());
    println!("║ {:<pw$} ║", sys_fan_line, pw = PADDING_WIDTH - get_visible_length(&sys_fan_line) + sys_fan_line.len());
    // Adjust pw for these two lines to remove the extra space
    println!("║ {:<pw$} ║", sys_setpoint_line, pw = PADDING_WIDTH - get_visible_length(&sys_setpoint_line) + sys_setpoint_line.len() - 1);
    println!("║ {:<pw$} ║", sys_temp_line, pw = PADDING_WIDTH - get_visible_length(&sys_temp_line) + sys_temp_line.len() - 1);
    println!("║ {:<pw$} ║", ac_error_line, pw = PADDING_WIDTH - get_visible_length(&ac_error_line) + ac_error_line.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));

    unsafe {
        if constants::VERBOSE {
            // Use the original response_value for verbose output
            println!("Full SystemV2Response: {:#?}", response_value);
        }
    }
}

pub fn get_system_temperature(client: &Client) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Controller Temperature", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let query_data = json!({ "iZoneV2Request": { "Type": 1, "No": 0, "No1": 0 } });

    let response_value = match make_query_request(client, query_data) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}", format!("Error querying controller temperature: {}", e).red());
            exit(1);
        }
    };

    let system_response: SystemV2Response = serde_json::from_value(response_value.clone()) // Cloned here for verbose output
        .expect("Failed to parse controller temperature response");

    let temp_text = format_temp(system_response.system_v2.temp);
    let temp_line = format!("Current Controller Temperature: {}°C", temp_text.cyan());
    println!("║ {:<padding_width$} ║", temp_line, padding_width = PADDING_WIDTH -1 - get_visible_length(&temp_line) + temp_line.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));

    unsafe {
        if constants::VERBOSE {
            println!("{}", serde_json::to_string_pretty(&response_value).unwrap());
        }
    }
}

pub fn turn_on_ac(client: &Client) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"SysOn":1});
    make_command_request(client, command_data)
        .expect("Failed to turn on AC system");
    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let message = format!("AC system turned {}.", "ON".green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn turn_off_ac(client: &Client) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"SysOn":0});
    make_command_request(client, command_data)
        .expect("Failed to turn off AC system");
    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let message = format!("AC system turned {}.", "OFF".red());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_system_mode(client: &Client, mode_name: &str) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let mode_value = match get_system_mode_value(mode_name) {
        Some(value) => value,
        None => {
            // Construct the error message clearly to avoid character literal issues
            let error_message = format!(
                "Error: Unknown system mode '{}'.\nAvailable modes: auto, cool, heat, vent, dry.", // Updated modes list
                mode_name
            ).red().to_string();
            eprintln!("{}", error_message);
            exit(1);
        }
    };

    let command_data = json!({"SysMode": mode_value}); // Use SysMode command
    make_command_request(client, command_data)
        .expect(&format!("Failed to set system mode to '{}'", mode_name));

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let message = format!("System mode set to {}.", mode_name.to_pascal_case().cyan());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_system_fan(client: &Client, fan_speed_name: &str) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let fan_speed_value = match get_fan_speed_value(fan_speed_name) {
        Some(value) => value,
        None => {
            let error_message = format!(
                "Error: Unknown fan speed '{}'.\nAvailable fan speeds: auto, low, medium, high.",
                fan_speed_name
            ).red().to_string();
            eprintln!("{}", error_message);
            exit(1);
        }
    };

    let command_data = json!({"SysFan": fan_speed_value}); // Use SysFan command
    make_command_request(client, command_data)
        .expect(&format!("Failed to set system fan to '{}'", fan_speed_name));

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let message = format!("Aircon Fan Speed set to {}.", fan_speed_name.to_pascal_case().cyan());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}