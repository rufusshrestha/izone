use reqwest::blocking::Client;
use serde_json::json;
use colored::Colorize;
use std::process::exit;

// Corrected imports for API functions and get_visible_length from helpers
use crate::api::{make_query_request, make_command_request}; // Updated API function names
use crate::constants;
use crate::helpers::{format_temp, get_colored_system_mode, get_fan_speed_text, get_visible_length}; // Imported get_visible_length
use crate::models::SystemV2Response;

// Removed: The local get_visible_length function is removed from here.
// It is now imported from `crate::helpers`.

pub fn get_system_status(client: &Client) {
    const BOX_WIDTH: usize = 45; // Total width of the box's horizontal lines
    const PADDING_WIDTH: usize = BOX_WIDTH - 2; // Subtract 2 for the '║ ' and ' ║'

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "AIRCON STATUS", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let query_data = json!({ "iZoneV2Request": { "Type": 1, "No": 0, "No1": 0 } });

    // Using make_query_request and handling its Result
    let response_value = match make_query_request(client, query_data) { // Updated function call
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}", format!("Failed to retrieve Aircon status: {}", e).red());
            exit(1);
        }
    };

    let system_v2_response: SystemV2Response =
        match serde_json::from_value(response_value.clone()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}Failed to parse Aircon status: {}", "Error: ".red(), e);
                exit(1);
            }
        };

    let system = system_v2_response.system_v2;

    // Section 1: General Status
    const LABEL_WIDTH: usize = 15; // "System On:" is 10 chars, "Set Temp:" is 9. "Temperature:" is 12. Max needed is "System On:".

    let status_text = if system.sys_on {
        "ON".green()
    } else {
        "OFF".red()
    };

    let sys_on_line = format!("{:width$} {}", "System On:", status_text, width = LABEL_WIDTH);
    let sys_mode_line = format!("{:width$} {}", "Mode:", get_colored_system_mode(system.sys_mode), width = LABEL_WIDTH);
    let sys_temp_line = format!("{:width$} {}°C", "Temperature:", format_temp(system.temp).cyan(), width = LABEL_WIDTH);
    let sys_setpoint_line = format!("{:width$} {}°C", "Set Temp:", format_temp(system.setpoint), width = LABEL_WIDTH);
    let sys_fan_line = format!("{:width$} {}", "Fan Speed:", get_fan_speed_text(system.sys_fan), width = LABEL_WIDTH);
    let ac_error_line = format!("{:width$} {}", "AC Error:", system.ac_error.normal(), width = LABEL_WIDTH-1);


    // Adjust padding based on visible length for colored strings
    println!("║ {:<pw$} ║", sys_on_line, pw = PADDING_WIDTH - get_visible_length(&sys_on_line) + sys_on_line.len());
    println!("║ {:<pw$} ║", sys_mode_line, pw = PADDING_WIDTH - get_visible_length(&sys_mode_line) + sys_mode_line.len());
    println!("║ {:<pw$} ║", sys_temp_line, pw = PADDING_WIDTH - get_visible_length(&sys_temp_line) + sys_temp_line.len());
    println!("║ {:<pw$} ║", sys_setpoint_line, pw = PADDING_WIDTH - get_visible_length(&sys_setpoint_line) + sys_setpoint_line.len());
    println!("║ {:<pw$} ║", sys_fan_line, pw = PADDING_WIDTH - get_visible_length(&sys_fan_line) + sys_fan_line.len());
    println!("║ {:<pw$} ║", ac_error_line, pw = PADDING_WIDTH - get_visible_length(&ac_error_line) + ac_error_line.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));

    unsafe {
        if constants::VERBOSE {
            println!("{}", serde_json::to_string_pretty(&response_value).unwrap());
        }
    }
}

pub fn get_system_temp(client: &Client) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Temperature", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let query_data = json!({ "iZoneV2Request": { "Type": 1, "No": 0, "No1": 0 } });

    // Using make_query_request and handling its Result
    let response_value = match make_query_request(client, query_data) { // Updated function call
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}", format!("Failed to retrieve system temperature: {}", e).red());
            exit(1);
        }
    };

    let system_v2_response: SystemV2Response =
        match serde_json::from_value(response_value.clone()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}Failed to parse system temperature: {}", "Error: ".red(), e);
                exit(1);
            }
        };

    let temp_line = format!("Current Temperature: {}°C", format_temp(system_v2_response.system_v2.temp).cyan());
    println!("║ {:<padding_width$} ║", temp_line, padding_width = PADDING_WIDTH - get_visible_length(&temp_line) + temp_line.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));

    unsafe {
        if constants::VERBOSE {
            println!("{}", serde_json::to_string_pretty(&response_value).unwrap());
        }
    }
}

pub fn turn_on_ac(client: &Client) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2; // Corrected typo here

    let command_data = json!({"SysOn":1});
    // Using make_command_request and handling its Result
    make_command_request(client, command_data) // Updated function call
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
    const PADDING_WIDTH: usize = BOX_WIDTH - 2; // Corrected typo here

    let command_data = json!({"SysOn":0});
    // Using make_command_request and handling its Result
    make_command_request(client, command_data) // Updated function call
        .expect("Failed to turn off AC system");
    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let message = format!("AC system turned {}.", "OFF".red());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}