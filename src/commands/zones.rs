// izone/src/commands/zones.rs

use reqwest::blocking::Client;
use serde_json::{json, Value};
use colored::Colorize;
use std::process::exit;

// Corrected imports for API functions and added get_visible_length from helpers
use crate::api::{make_query_request, make_command_request};
use crate::constants::{self, ZONES}; // Import the constants module itself, and ZONES
use crate::helpers::{format_temp, get_battery_level_text, get_zone_type_text, get_visible_length, get_sensor_fault_text, get_colored_system_mode}; // Added get_colored_system_mode
use crate::models::ZonesV2Response; // Removed ZoneListV2Response from here, will use full path where needed

pub fn control_zone(client: &Client, zone_name: &str, action: &str, value: Option<&str>) {
    let zone_index = match ZONES.get(zone_name) {
        Some(&index) => index,
        None => {
            eprintln!(
                "{}{}{}{}",
                "Error: Unknown zone '".red(),
                zone_name.red(),
                "'.\nAvailable zones: ".red(),
                ZONES
                    .keys()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
                    .red()
            );
            exit(1);
        }
    };

    let capitalized_zone_name = {
        let mut chars = zone_name.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        }
    };

    const BOX_WIDTH: usize = 70; // Increased box width for zone control messages
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    // Initialize command_data directly from the match expression
    let command_data: Option<Value> = match action {
        "status" | "stat" => {
            get_zone_status(client, zone_name);
            return; // Exit function after displaying status, no command to send
        }
        "temp" => {
            get_zone_temperature(client, zone_name);
            return; // Exit function after displaying temperature, no command to send
        }
        "on" => Some(json!({"ZoneStatus":{"Index":zone_index,"Mode":3}})), // Auto mode (ON)
        "off" => Some(json!({"ZoneStatus":{"Index":zone_index,"Mode":0}})), // Off mode
        "open" => Some(json!({"ZoneStatus":{"Index":zone_index,"Mode":1}})), // Open mode
        "auto" => Some(json!({"ZoneStatus":{"Index":zone_index,"Mode":3}})), // Auto mode
        "override" => Some(json!({"ZoneStatus":{"Index":zone_index,"Mode":4}})), // Override mode
        "constant" => Some(json!({"ZoneStatus":{"Index":zone_index,"Mode":2}})), // Constant mode
        "set_setpoint" => {
            let setpoint_raw = value.expect("Setpoint temperature is required for set_setpoint action.");
            let setpoint_float: f32 = setpoint_raw
                .parse()
                .expect("Invalid setpoint temperature. Must be a number.");
            let setpoint_int = (setpoint_float * 100.0).round() as u32;

            if setpoint_int < 1500 || setpoint_int > 3000 {
                eprintln!(
                    "{}Setpoint temperature '{}' out of valid range (15.0-30.0°C).",
                    "Error: ".red(),
                    setpoint_raw
                );
                exit(1);
            }
            Some(json!({"ZoneSetpoint":{"Index":zone_index,"Setpoint":setpoint_int}}))
        }
        "set_max_air" => {
            let percentage_raw = value.expect("Max air percentage is required.");
            let percentage: u8 = percentage_raw
                .parse()
                .expect("Invalid percentage. Must be a number between 0 and 100.");
            if percentage > 100 {
                eprintln!("{}", "Error: Max air percentage cannot exceed 100.".red());
                exit(1);
            }
            Some(json!({"ZoneAirflow":{"Index":zone_index,"MaxAir":percentage}}))
        }
        "set_min_air" => {
            let percentage_raw = value.expect("Min air percentage is required.");
            let percentage: u8 = percentage_raw
                .parse()
                .expect("Invalid percentage. Must be a number between 0 and 100.");
            if percentage > 100 { // Although min, still sensible to cap at 100
                eprintln!("{}", "Error: Min air percentage cannot exceed 100.".red());
                exit(1);
            }
            Some(json!({"ZoneAirflow":{"Index":zone_index,"MinAir":percentage}}))
        }
        "set_name" => {
            let new_name = value.expect("New zone name is required.");
            if new_name.len() > 15 {
                eprintln!("{}", "Error: Zone name cannot exceed 15 characters.".red());
                exit(1);
            }
            Some(json!({"ZoneName":{"Index":zone_index,"Name":new_name}}))
        }
        "summary" => { // Handle summary action
            // This action doesn't send a command to a specific zone,
            // but rather calls the summary function directly.
            // No command_data is generated here.
            None
        }
        _ => {
            eprintln!(
                "{}{}{}",
                "Error: Unknown zone action '".red(),
                action.red(),
                "'.\nAvailable actions for zones: status, temp, on, off, open, auto, override, constant, set-setpoint, set-max-air, set-min-air, set-name, summary.".red()
            );
            exit(1); // Exit if unknown action, no command_data to return
        }
    };

    // Only make a command request if command_data is Some
    if let Some(cmd_data) = command_data {
        make_command_request(client, cmd_data)
            .expect(&format!("Failed to execute '{}' for zone '{}'", action, zone_name));

        println!("╔{}╗", "═".repeat(BOX_WIDTH));
        println!("║ {:^padding_width$} ║", format!("Zone Control: {}", capitalized_zone_name), padding_width = PADDING_WIDTH);
        println!("╠{}╣", "═".repeat(BOX_WIDTH));
        let message = format!("Action '{}' for zone '{}' successful.", action.green(), capitalized_zone_name.green());
        println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
        println!("╚{}╝", "═".repeat(BOX_WIDTH));
    }
}


pub fn get_zone_status(client: &Client, zone_name: &str) {
    let zone_index = match constants::ZONES.get(zone_name) {
        Some(&index) => index,
        None => {
            eprintln!(
                "{}{}{}{}",
                "Error: Unknown zone '".red(),
                zone_name.red(),
                "'.\nAvailable zones: ".red(),
                constants::ZONES
                    .keys()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
                    .red()
            );
            exit(1);
        }
    };

    let query_data = json!({ "iZoneV2Request": { "Type": 3, "No": zone_index, "No1": 0 } });

    const BOX_WIDTH: usize = 60; // Slightly wider box for detailed zone status
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;
    const LABEL_WIDTH: usize = 20;

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", format!("ZONE STATUS: {}", zone_name.to_uppercase()), padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let response_value = match make_query_request(client, query_data) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}", format!("Error querying zone status: {}", e).red());
            exit(1);
        }
    };

    let zone_response: ZonesV2Response = serde_json::from_value(response_value.clone())
        .expect("Failed to parse zone status response");

    let zone = zone_response.zones_v2;

    let mode_text = get_colored_system_mode(zone.mode); // Re-using system mode for zones
    let temp_text = format_temp(zone.temp);
    let setpoint_text = format_temp(zone.setpoint);
    let damper_pos_text = format!("{}%", zone.damper_pos);
    let zone_type_text = get_zone_type_text(zone.zone_type);
    let batt_volt_text = get_battery_level_text(zone.batt_volt);
    let rf_signal_text = format!("{}%", zone.rf_signal);
    let sensor_fault_text = get_sensor_fault_text(zone.sensor_fault); // Using new helper


    // Function to print a line with left-aligned label and right-aligned value,
    // accounting for invisible ANSI escape codes.
    let print_line = |label: &str, value: String| {
        let visible_value_len = get_visible_length(&value);
        let ansi_offset = value.len() - visible_value_len;
        let padding = PADDING_WIDTH - LABEL_WIDTH - visible_value_len; // Calculate spaces between label and value
        println!("║ {:<LABEL_WIDTH$}{:padding$}{}{} ║", label, "", value, " ".repeat(ansi_offset), padding=padding);
    };


    print_line("Name:", zone.name.normal().to_string());
    print_line("Mode:", mode_text);
    print_line("Current Temp:", format!("{}°C", temp_text).cyan().to_string());
    print_line("Setpoint:", format!("{}°C", setpoint_text).normal().to_string());
    print_line("Damper Position:", damper_pos_text.normal().to_string());
    print_line("Zone Type:", zone_type_text.normal().to_string());
    print_line("Sensor Type:", zone.sens_type.to_string().normal().to_string());
    print_line("Max Air:", format!("{}%", zone.max_air).normal().to_string());
    print_line("Min Air:", format!("{}%", zone.min_air).normal().to_string());
    print_line("Constant:", zone.constant.to_string().normal().to_string());
    print_line("Constant A:", zone.constant_a.to_string().normal().to_string());
    print_line("Master:", zone.master.to_string().normal().to_string());
    print_line("Damper Fault:", zone.damper_fault.to_string().normal().to_string());
    print_line("Sensor Fault:", sensor_fault_text); // Using the new helper
    print_line("Damper Skip:", zone.damper_skip.to_string().normal().to_string());
    print_line("Isense:", zone.isense.to_string().normal().to_string());
    print_line("Calibration:", zone.calibration.to_string().normal().to_string());
    print_line("RF Signal:", rf_signal_text.normal().to_string());
    print_line("Battery Voltage:", batt_volt_text);
    print_line("Area:", zone.area.to_string().normal().to_string());
    print_line("Bypass:", zone.bypass.to_string().normal().to_string());
    print_line("Balance Max:", format!("{}%", zone.balance_max).normal().to_string());
    print_line("Balance Min:", format!("{}%", zone.balance_min).normal().to_string());


    println!("╚{}╝", "═".repeat(BOX_WIDTH));

    unsafe {
        if constants::VERBOSE {
            println!("Full ZonesV2Response: {:#?}", response_value);
        }
    }
}

pub fn get_zone_temperature(client: &Client, zone_name: &str) {
    let zone_index = match constants::ZONES.get(zone_name) {
        Some(&index) => index,
        None => {
            eprintln!(
                "{}{}{}{}",
                "Error: Unknown zone '".red(),
                zone_name.red(),
                "'.\nAvailable zones: ".red(),
                constants::ZONES
                    .keys()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
                    .red()
            );
            exit(1);
        }
    };

    let query_data = json!({ "iZoneV2Request": { "Type": 3, "No": zone_index, "No1": 0 } });

    let response_value = match make_query_request(client, query_data) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}", format!("Error querying zone temperature: {}", e).red());
            exit(1);
        }
    };

    let zone_response: ZonesV2Response = serde_json::from_value(response_value)
        .expect("Failed to parse zone temperature response");

    let temp_text = format_temp(zone_response.zones_v2.temp);
    println!("Current Temperature for {}: {}°C", zone_name.to_uppercase(), temp_text.cyan());
}


pub fn get_all_zones_summary(client: &Client) {
    // Column widths for the summary table (visible characters)
    const NAME_COL_WIDTH: usize = 15; // e.g., "Dining_Room_Zone"
    const MODE_COL_WIDTH: usize = 8;  // e.g., "OVRIDE" or "UNKNOWN" - longest is 7, so 8 is good.
    const TEMP_COL_WIDTH: usize = 12; // e.g., "Temp: 20.0°C" (12 visible chars)
    const SETPOINT_COL_WIDTH: usize = 15; // e.g., "Setpoint: 20.0°C" (15 visible chars)
    const DAMPER_COL_WIDTH: usize = 13; // e.g., "Damper: 100%" (13 visible chars)
    const STATUS_COL_WIDTH: usize = 25; // Max space for additional status, e.g., "DmpFlt SnsFlt LowBatt"

    // Calculate the total inner content width
    // Name (15) + ": " (2) + Mode (8) + " " (1) + Temp (12) + " " (1) + Setpoint (15) + " " (1) + Damper (13) + " " (1) + Status (25)
    const SUMMARY_INNER_CONTENT_WIDTH: usize = NAME_COL_WIDTH + 2 + MODE_COL_WIDTH + 1 + TEMP_COL_WIDTH + 1 + SETPOINT_COL_WIDTH + 1 + DAMPER_COL_WIDTH + 1 + STATUS_COL_WIDTH;

    // Total box width (inner content + 2 for "║ " and " ║")
    const SUMMARY_BOX_WIDTH: usize = SUMMARY_INNER_CONTENT_WIDTH + 2;

    println!("╔{}╗", "═".repeat(SUMMARY_BOX_WIDTH));
    println!("║ {:^SUMMARY_INNER_CONTENT_WIDTH$} ║", "ZONE SUMMARY");
    println!("╠{}╣", "═".repeat(SUMMARY_BOX_WIDTH));

    // Fetch all zone data first
    let mut zones_data: Vec<crate::models::ZonesV2> = Vec::new(); // Store ZonesV2 structs

    for (&zone_index, zone_name) in ZONES.values().zip(ZONES.keys()) {
        let query_data = json!({ "iZoneV2Request": { "Type": 2, "No": zone_index, "No1": 0 } });

        // Using make_query_request and handling its Result
        let response_value = match make_query_request(client, query_data) {
            Ok(val) => val,
            Err(e) => {
                let error_line_prefix_raw = format!("{:<NAME_COL_WIDTH$}: {}", zone_name.replace(' ', "_"), "ERROR retrieving status: ");
                let error_message_part_raw = format!("{}", e);
                let full_error_line_raw = format!("{}{}", error_line_prefix_raw, error_message_part_raw);
                let full_error_line_colored = format!("{}{}", error_line_prefix_raw.red(), error_message_part_raw.red());

                println!(
                    "║ {:<pw$}║",
                    full_error_line_colored,
                    pw = SUMMARY_INNER_CONTENT_WIDTH - get_visible_length(&full_error_line_raw) + full_error_line_colored.len()
                );
                continue;
            }
        };

        let zones_v2_response: ZonesV2Response =
            match serde_json::from_value(response_value.clone()) {
                Ok(z) => z,
                Err(e) => {
                    eprintln!("{}Failed to parse zone data: {}", "Error: ".red(), e);
                    let error_line_prefix_raw = format!("{:<NAME_COL_WIDTH$}: {}", zone_name.replace(' ', "_"), "ERROR parsing data: ");
                    let error_message_part_raw = format!("{}", e);
                    let full_error_line_raw = format!("{}{}", error_line_prefix_raw, error_message_part_raw);
                    let full_error_line_colored = format!("{}{}", error_line_prefix_raw.red(), error_message_part_raw.red());

                    println!(
                        "║ {:<pw$}║",
                        full_error_line_colored,
                        pw = SUMMARY_INNER_CONTENT_WIDTH - get_visible_length(&full_error_line_raw) + full_error_line_colored.len()
                    );
                    continue;
                }
            };
        zones_data.push(zones_v2_response.zones_v2);
    }

    // Sort zones alphabetically by name
    zones_data.sort_by(|a, b| a.name.cmp(&b.name));

    // Now iterate through the sorted zones to print the summary
    for zone in &zones_data { // Changed to iterate over a reference
        let zone_name_display = zone.name.replace(' ', "_"); // Replace spaces with underscores for display

        let zone_mode_colored_text = match zone.mode {
            1 => "OPEN".yellow().to_string(),
            2 => "OFF".red().to_string(),
            3 => "ON".green().to_string(),
            4 => "OVRIDE".yellow().to_string(),
            5 => "CONST".yellow().to_string(),
            _ => "UNKNOWN".normal().to_string(),
        };

        let mut additional_status_colored = String::new();
        if zone.damper_fault == 1 {
            additional_status_colored.push_str(&format!(" {}{}", "DmpFlt".red(), "".normal()));
        }
        if zone.sensor_fault == 1 {
            additional_status_colored.push_str(&format!(" {}{}", "SnsFlt".red(), "".normal()));
        }
        // IMPORTANT: Replace '9' with the actual SensType code(s) for your wireless battery sensors.
        // If you have multiple wireless sensor types, use: (zone.sens_type == 9 || zone.sens_type == X)
        if zone.batt_volt == 0 && zone.sens_type == 9 {
            additional_status_colored.push_str(&format!(" {}{}", "LowBatt".red(), "".normal()));
        }


        // 1. Zone Name (left-aligned within its column)
        let name_part = format!("{:<NAME_COL_WIDTH$}", zone_name_display);

        // 2. Mode (colored, left-aligned within its column, but padding needs visible length)
        let mode_raw = zone_mode_colored_text; // Already colored
        let mode_padding = MODE_COL_WIDTH - get_visible_length(&mode_raw) + mode_raw.len();
        let mode_part = format!("{:>mode_padding$}", mode_raw); // Right-align within its calculated padding

        // 3. Current Temp (colored temp value, left-aligned within its column)
        let temp_value_colored = format_temp(zone.temp).cyan().to_string();
        let temp_part_raw = format!("Temp: {}°C", temp_value_colored);
        let temp_padding = TEMP_COL_WIDTH - get_visible_length(&temp_part_raw) + temp_part_raw.len();
        let temp_part = format!("{:<temp_padding$}", temp_part_raw);

        // 4. Setpoint (not colored, left-aligned within its column)
        let setpoint_value = format_temp(zone.setpoint).to_string();
        let setpoint_part_raw = format!("Setpoint: {}°C", setpoint_value);
        let setpoint_padding = SETPOINT_COL_WIDTH - get_visible_length(&setpoint_part_raw) + setpoint_part_raw.len();
        let setpoint_part = format!("{:<setpoint_padding$}", setpoint_part_raw);

        // 5. Damper (not colored, left-aligned within its column)
        let damper_part_raw = format!("Damper: {}%", zone.damper_pos);
        let damper_padding = DAMPER_COL_WIDTH - get_visible_length(&damper_part_raw) + damper_part_raw.len();
        let damper_part = format!("{:<damper_padding$}", damper_part_raw);

        // 6. Additional Status (colored, left-aligned within its column)
        let status_part_raw = additional_status_colored; // Already colored
        let status_padding = STATUS_COL_WIDTH - get_visible_length(&status_part_raw) + status_part_raw.len();
        let status_part = format!("{:<status_padding$}", status_part_raw);


        // Assemble the final line content with explicit separators
        let final_line_content = format!(
            "{}:{} {} {} {} {}",
            name_part,
            mode_part,
            temp_part,
            setpoint_part,
            damper_part,
            status_part
        );

        // Print the assembled line, using overall padding logic for the entire line
        println!("║ {:<pw$}║", final_line_content, pw = SUMMARY_INNER_CONTENT_WIDTH - get_visible_length(&final_line_content) + final_line_content.len());
    }
    println!("╚{}╝", "═".repeat(SUMMARY_BOX_WIDTH));
    unsafe {
        if constants::VERBOSE {
            // Reconstruct all_zones_responses to ensure sorted output if VERBOSE is true
            let sorted_all_zones_responses: Vec<Value> = zones_data.iter().map(|z| serde_json::to_value(&z).unwrap()).collect(); // Changed to .iter() and &z
            println!("{}", serde_json::to_string_pretty(&sorted_all_zones_responses).unwrap());
        }
    }
}