use reqwest::blocking::Client;
use serde_json::json;
use colored::Colorize;
use std::process::exit;

use crate::api::{query_izone_raw, send_command};
use crate::constants;
use crate::helpers::{format_temp, get_colored_system_mode, get_fan_speed_text};
use crate::models::SystemV2Response;

// Helper function to calculate visible length, ignoring ANSI escape codes
fn get_visible_length(s: &str) -> usize {
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

pub fn get_system_status(client: &Client) {
    const BOX_WIDTH: usize = 45; // Total width of the box's horizontal lines
    const PADDING_WIDTH: usize = BOX_WIDTH - 2; // Subtract 2 for the '║ ' and ' ║'

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "AIRCON STATUS", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let query_data = json!({ "iZoneV2Request": { "Type": 1, "No": 0, "No1": 0 } });

    let response_value = match query_izone_raw(client, &query_data) {
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

    let sys_on_text = if system_v2_response.system_v2.sys_on {
        "ON".green().to_string()
    } else {
        "OFF".red().to_string()
    };

    let ac_error_text = if system_v2_response.system_v2.ac_error == "OK" {
        "OK".green().to_string()
    } else {
        system_v2_response.system_v2.ac_error.green().to_string()
    };

    // Construct the line content, then calculate visible length for padding
    let line1 = format!("Aircon Status: {}", sys_on_text);
    let line2 = format!("Mode:          {}", get_colored_system_mode(system_v2_response.system_v2.sys_mode));
    let line3 = format!("Current Temp:  {}°C", format_temp(system_v2_response.system_v2.temp).cyan());
    let line4 = format!("Setpoint:      {}°C", format_temp(system_v2_response.system_v2.setpoint));
    let line5 = format!("Fan Speed:     {}", get_fan_speed_text(system_v2_response.system_v2.sys_fan));
    let line6 = format!("AC Error:     {}", ac_error_text);

    // Print lines using visible length for padding
    println!("║ {:<padding_width$} ║", line1, padding_width = PADDING_WIDTH - get_visible_length(&line1) + line1.len());
    println!("║ {:<padding_width$} ║", line2, padding_width = PADDING_WIDTH - get_visible_length(&line2) + line2.len());
    println!("║ {:<padding_width$} ║", line3, padding_width = PADDING_WIDTH - get_visible_length(&line3) + line3.len());
    println!("║ {:<padding_width$} ║", line4, padding_width = PADDING_WIDTH - get_visible_length(&line4) + line4.len());
    println!("║ {:<padding_width$} ║", line5, padding_width = PADDING_WIDTH - get_visible_length(&line5) + line5.len());
    println!("║ {:<padding_width$} ║", line6, padding_width = PADDING_WIDTH - get_visible_length(&line6) + line6.len());

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

    let response_value = match query_izone_raw(client, &query_data) {
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
                eprintln!(
                    "{}Failed to parse system temperature: {}",
                    "Error: ".red(),
                    e
                );
                exit(1);
            }
        };

    let temp_line = format!(
        "Current System Temperature: {}°C",
        format_temp(system_v2_response.system_v2.temp).cyan()
    );

    println!(
        "║ {:<padding_width$} ║",
        temp_line,
        padding_width = PADDING_WIDTH - get_visible_length(&temp_line) + temp_line.len()
    );
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
    send_command(client, &command_data)
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
    send_command(client, &command_data)
        .expect("Failed to turn off AC system");
    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let message = format!("AC system turned {}.", "OFF".red());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}