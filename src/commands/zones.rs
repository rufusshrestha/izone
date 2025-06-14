// izone/src/commands/zones.rs

use reqwest::blocking::Client;
use serde_json::{json, Value};
use colored::Colorize;
use std::process::exit;

use crate::api::{query_izone_raw, send_command};
use crate::constants::{self, ZONES}; // Import the constants module itself, and ZONES
use crate::helpers::{format_temp, get_battery_level_text, get_zone_type_text};
use crate::models::ZonesV2Response;

// Helper function to calculate visible length, ignoring ANSI escape codes
fn get_visible_length(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        match c {
            '\x1B' => in_escape = true, // Start of an ANSI escape sequence
            'm' if in_escape => in_escape = false, // End of a common ANSI color sequence
            _ if in_escape => { /* Do nothing, part of escape sequence */ },
            _ => len += c.len_utf8(), // Count visible character length (works for UTF-8 chars)
        }
    }
    len
}

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

    const ZONE_INNER_WIDTH: usize = 50; // Inner content width, excluding '║ ' and ' ║'
    const BOX_WIDTH: usize = ZONE_INNER_WIDTH + 2; // Total width for the box

    match action {
        "status" | "stat" => {
            println!("╔{}╗", "═".repeat(BOX_WIDTH));
            println!("║ {:^ZONE_INNER_WIDTH$} ║", format!("Zone Status: {} Room", capitalized_zone_name));
            println!("╠{}╣", "═".repeat(BOX_WIDTH));
            let query_data = json!({ "iZoneV2Request": { "Type": 2, "No": zone_index, "No1": 0 } });

            let response_value = match query_izone_raw(client, &query_data) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("{}", format!("Failed to retrieve zone status: {} Room", e).red());
                    exit(1);
                }
            };

            let zones_v2_response: ZonesV2Response =
                match serde_json::from_value(response_value.clone()) {
                    Ok(z) => z,
                    Err(e) => {
                        eprintln!("{}Failed to parse zone status: {} Room", "Error: ".red(), e);
                        exit(1);
                    }
                };

            let zone = zones_v2_response.zones_v2;

            let zone_mode_text = match zone.mode {
                1 => "OPEN (Manual)".yellow().to_string(),
                2 => "OFF".red().to_string(),
                3 => "ON".green().to_string(),
                4 => "OVERRIDE".yellow().to_string(),
                5 => "CONSTANT".yellow().to_string(),
                _ => format!("Unknown ({})", zone.mode).normal().to_string(),
            };

            let damper_fault_text = if zone.damper_fault == 1 {
                "FAULT".red().to_string()
            } else {
                zone.damper_fault.to_string().normal().to_string()
            };
            let sensor_fault_text = if zone.sensor_fault == 1 {
                "FAULT".red().to_string()
            } else {
                zone.sensor_fault.to_string().normal().to_string()
            };

            // Section 1: General Status
            const GENERAL_LABEL_WIDTH: usize = 17; // "Current Temp:" is 13 chars

            let line_name = format!("{:width$} {}", "Zone Name:", zone.name, width = GENERAL_LABEL_WIDTH);
            let line_status = format!("{:width$} {}", "Status:", zone_mode_text, width = GENERAL_LABEL_WIDTH);
            let line_zone_type = format!("{:width$} {}", "Zone Type:", get_zone_type_text(zone.zone_type), width = GENERAL_LABEL_WIDTH);
            let line_current_temp = format!("{:width$} {}°C", "Current Temp:", format_temp(zone.temp).cyan(), width = GENERAL_LABEL_WIDTH);
            let line_setpoint = format!("{:width$} {}°C", "Setpoint:", format_temp(zone.setpoint), width = GENERAL_LABEL_WIDTH);
            let line_damper_pos = format!("{:width$} {}%", "Damper Pos:", zone.damper_pos, width = GENERAL_LABEL_WIDTH);
            let line_max_air = format!("{:width$} {}%", "Max Airflow:", zone.max_air, width = GENERAL_LABEL_WIDTH);
            let line_min_air = format!("{:width$} {}%", "Min Airflow:", zone.min_air, width = GENERAL_LABEL_WIDTH);

            println!("║ {:<pw$} ║", line_name, pw = ZONE_INNER_WIDTH - get_visible_length(&line_name) + line_name.len());
            println!("║ {:<pw$} ║", line_status, pw = ZONE_INNER_WIDTH - get_visible_length(&line_status) + line_status.len());
            println!("║ {:<pw$} ║", line_zone_type, pw = ZONE_INNER_WIDTH - get_visible_length(&line_zone_type) + line_zone_type.len());
            println!("║ {:<pw$} ║", line_current_temp, pw = ZONE_INNER_WIDTH - get_visible_length(&line_current_temp) + line_current_temp.len());
            println!("║ {:<pw$} ║", line_setpoint, pw = ZONE_INNER_WIDTH - get_visible_length(&line_setpoint) + line_setpoint.len());
            println!("║ {:<pw$} ║", line_damper_pos, pw = ZONE_INNER_WIDTH - get_visible_length(&line_damper_pos) + line_damper_pos.len());
            println!("║ {:<pw$} ║", line_max_air, pw = ZONE_INNER_WIDTH - get_visible_length(&line_max_air) + line_max_air.len());
            println!("║ {:<pw$} ║", line_min_air, pw = ZONE_INNER_WIDTH - get_visible_length(&line_min_air) + line_min_air.len());

            println!("╠{}╣", "═".repeat(BOX_WIDTH));
            println!("║ {:^ZONE_INNER_WIDTH$} ║", "Sensor & Faults");
            println!("╠{}╣", "═".repeat(BOX_WIDTH));

            // Section 2: Sensor & Faults
            const SENSOR_FAULT_LABEL_WIDTH: usize = 17; // "Sensor Fault:" is 13 chars, "RF Signal:" is 10. Max needed is "Sensor Type:". Using 14 for good measure.

            let line_sens_type = format!("{:width$} {}", "Sensor Type:", zone.sens_type, width = SENSOR_FAULT_LABEL_WIDTH);
            let line_sensor_fault = format!("{:width$} {}", "Sensor Fault:", sensor_fault_text, width = SENSOR_FAULT_LABEL_WIDTH);
            let line_damper_fault = format!("{:width$} {}", "Damper Fault:", damper_fault_text, width = SENSOR_FAULT_LABEL_WIDTH);
            let line_isense = format!("{:width$} {}", "iSense Active:", zone.isense, width = SENSOR_FAULT_LABEL_WIDTH);
            let line_calibration = format!("{:width$} {}", "Calibration:", zone.calibration, width = SENSOR_FAULT_LABEL_WIDTH);
            let line_rf_signal = format!("{:width$} {}", "RF Signal:", zone.rf_signal, width = SENSOR_FAULT_LABEL_WIDTH);
            let line_battery = format!("{:width$} {}", "Battery:", get_battery_level_text(zone.batt_volt), width = SENSOR_FAULT_LABEL_WIDTH);

            println!("║ {:<pw$} ║", line_sens_type, pw = ZONE_INNER_WIDTH - get_visible_length(&line_sens_type) + line_sens_type.len());
            println!("║ {:<pw$} ║", line_sensor_fault, pw = ZONE_INNER_WIDTH - get_visible_length(&line_sensor_fault) + line_sensor_fault.len());
            println!("║ {:<pw$} ║", line_damper_fault, pw = ZONE_INNER_WIDTH - get_visible_length(&line_damper_fault) + line_damper_fault.len());
            println!("║ {:<pw$} ║", line_isense, pw = ZONE_INNER_WIDTH - get_visible_length(&line_isense) + line_isense.len());
            println!("║ {:<pw$} ║", line_calibration, pw = ZONE_INNER_WIDTH - get_visible_length(&line_calibration) + line_calibration.len());
            println!("║ {:<pw$} ║", line_rf_signal, pw = ZONE_INNER_WIDTH - get_visible_length(&line_rf_signal) + line_rf_signal.len());
            println!("║ {:<pw$} ║", line_battery, pw = ZONE_INNER_WIDTH - get_visible_length(&line_battery) + line_battery.len());

            println!("╠{}╣", "═".repeat(BOX_WIDTH));
            println!("║ {:^ZONE_INNER_WIDTH$} ║", "Advanced");
            println!("╠{}╣", "═".repeat(BOX_WIDTH));

            // Section 3: Advanced
            const ADVANCED_LABEL_WIDTH: usize = 17; // "Constant Active:" is 16 chars. Using 17 for padding.

            let line_constant = format!("{:width$} {}", "Constant:", zone.constant, width = ADVANCED_LABEL_WIDTH);
            let line_constant_a = format!("{:width$} {}", "Constant Active:", zone.constant_a, width = ADVANCED_LABEL_WIDTH);
            let line_master = format!("{:width$} {}", "Master Zone:", zone.master, width = ADVANCED_LABEL_WIDTH);
            let line_area = format!("{:width$} {}m²", "Area:", zone.area, width = ADVANCED_LABEL_WIDTH);
            let line_bypass = format!("{:width$} {}", "Bypass:", zone.bypass, width = ADVANCED_LABEL_WIDTH);
            let line_balance_max = format!("{:width$} {}", "Balance Max:", zone.balance_max, width = ADVANCED_LABEL_WIDTH);
            let line_balance_min = format!("{:width$} {}", "Balance Min:", zone.balance_min, width = ADVANCED_LABEL_WIDTH);
            let line_damper_skip = format!("{:width$} {}", "Damper Skip:", zone.damper_skip, width = ADVANCED_LABEL_WIDTH);

            println!("║ {:<pw$} ║", line_constant, pw = ZONE_INNER_WIDTH - get_visible_length(&line_constant) + line_constant.len());
            println!("║ {:<pw$} ║", line_constant_a, pw = ZONE_INNER_WIDTH - get_visible_length(&line_constant_a) + line_constant_a.len());
            println!("║ {:<pw$} ║", line_master, pw = ZONE_INNER_WIDTH - get_visible_length(&line_master) + line_master.len());
            println!("║ {:<pw$} ║", line_area, pw = ZONE_INNER_WIDTH - get_visible_length(&line_area) + line_area.len());
            println!("║ {:<pw$} ║", line_bypass, pw = ZONE_INNER_WIDTH - get_visible_length(&line_bypass) + line_bypass.len());
            println!("║ {:<pw$} ║", line_balance_max, pw = ZONE_INNER_WIDTH - get_visible_length(&line_balance_max) + line_balance_max.len());
            println!("║ {:<pw$} ║", line_balance_min, pw = ZONE_INNER_WIDTH - get_visible_length(&line_balance_min) + line_balance_min.len());
            println!("║ {:<pw$} ║", line_damper_skip, pw = ZONE_INNER_WIDTH - get_visible_length(&line_damper_skip) + line_damper_skip.len());

            println!("╚{}╝", "═".repeat(BOX_WIDTH));

            unsafe {
                if constants::VERBOSE {
                    println!("{}", serde_json::to_string_pretty(&response_value).unwrap());
                }
            }
        }
        "on" | "auto" => {
            let command_data = json!({"ZoneMode":{"Index":zone_index,"Mode":3}});
            send_command(client, &command_data)
                .expect("Failed to send ON/AUTO command");
            println!("╔{}╗", "═".repeat(BOX_WIDTH));
            println!("║ {:^ZONE_INNER_WIDTH$} ║", format!("Zone Control: {} Room", capitalized_zone_name));
            println!("╠{}╣", "═".repeat(BOX_WIDTH));
            let message = format!(
                "Set zone {} to {}.", capitalized_zone_name, "ON (Auto Mode)".green()
            );
            println!("║ {:<pw$} ║", message, pw = ZONE_INNER_WIDTH - get_visible_length(&message) + message.len());
            println!("╚{}╝", "═".repeat(BOX_WIDTH));
        }
        "off" => {
            let command_data = json!({"ZoneMode":{"Index":zone_index,"Mode":2}});
            send_command(client, &command_data)
                .expect("Failed to send OFF command");
            println!("╔{}╗", "═".repeat(BOX_WIDTH));
            println!("║ {:^ZONE_INNER_WIDTH$} ║", format!("Zone Control: {} Room", capitalized_zone_name));
            println!("╠{}╣", "═".repeat(BOX_WIDTH));
            let message = format!(
                "Set zone {} to {}.", capitalized_zone_name, "OFF (Close Mode)".red()
            );
            println!("║ {:<pw$} ║", message, pw = ZONE_INNER_WIDTH - get_visible_length(&message) + message.len());
            println!("╚{}╝", "═".repeat(BOX_WIDTH));
        }
        "open" => {
            let command_data = json!({"ZoneMode":{"Index":zone_index,"Mode":1}});
            send_command(client, &command_data)
                .expect("Failed to send OPEN command");
            println!("╔{}╗", "═".repeat(BOX_WIDTH));
            println!("║ {:^ZONE_INNER_WIDTH$} ║", format!("Zone Control: {} Room", capitalized_zone_name));
            println!("╠{}╣", "═".repeat(BOX_WIDTH));
            let message = format!(
                "Set zone {} to {}.", capitalized_zone_name, "OPEN (Manual)".yellow()
            );
            println!("║ {:<pw$} ║", message, pw = ZONE_INNER_WIDTH - get_visible_length(&message) + message.len());
            println!("╚{}╝", "═".repeat(BOX_WIDTH));
        }
        "override" => {
            let command_data = json!({"ZoneMode":{"Index":zone_index,"Mode":4}});
            send_command(client, &command_data)
                .expect("Failed to send OVERRIDE command");
            println!("╔{}╗", "═".repeat(BOX_WIDTH));
            println!("║ {:^ZONE_INNER_WIDTH$} ║", format!("Zone Control: {} Room", capitalized_zone_name));
            println!("╠{}╣", "═".repeat(BOX_WIDTH));
            let message = format!(
                "Set zone {} to {}.", capitalized_zone_name, "OVERRIDE".yellow()
            );
            println!("║ {:<pw$} ║", message, pw = ZONE_INNER_WIDTH - get_visible_length(&message) + message.len());
            println!("╚{}╝", "═".repeat(BOX_WIDTH));
        }
        "constant" => {
            let command_data = json!({"ZoneMode":{"Index":zone_index,"Mode":5}});
            send_command(client, &command_data)
                .expect("Failed to send CONSTANT command");
            println!("╔{}╗", "═".repeat(BOX_WIDTH));
            println!("║ {:^ZONE_INNER_WIDTH$} ║", format!("Zone Control: {} Room", capitalized_zone_name));
            println!("╠{}╣", "═".repeat(BOX_WIDTH));
            let message = format!(
                "Set zone {} to {}.", capitalized_zone_name, "CONSTANT".yellow()
            );
            println!("║ {:<pw$} ║", message, pw = ZONE_INNER_WIDTH - get_visible_length(&message) + message.len());
            println!("╚{}╝", "═".repeat(BOX_WIDTH));
        }
        "temp" | "temperature" => {
            println!("╔{}╗", "═".repeat(BOX_WIDTH));
            println!("║ {:^ZONE_INNER_WIDTH$} ║", format!("Zone Temperature: {}", capitalized_zone_name));
            println!("╠{}╣", "═".repeat(BOX_WIDTH));

            let query_data = json!({ "iZoneV2Request": { "Type": 2, "No": zone_index, "No1": 0 } });

            let response_value = match query_izone_raw(client, &query_data) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("{}", format!("Failed to retrieve zone temperature: {}", e).red());
                    exit(1);
                }
            };

            let zones_v2_response: ZonesV2Response =
                match serde_json::from_value(response_value.clone()) {
                    Ok(z) => z,
                    Err(e) => {
                        eprintln!(
                            "{}Failed to parse zone temperature: {}",
                            "Error: ".red(),
                            e
                        );
                        exit(1);
                    }
                };

            let temp_line = format!(
                "{} Temperature: {}°C", zones_v2_response.zones_v2.name, format_temp(zones_v2_response.zones_v2.temp).cyan()
            );
            println!("║ {:<pw$} ║", temp_line, pw = ZONE_INNER_WIDTH - get_visible_length(&temp_line) + temp_line.len());
            println!("╚{}╝", "═".repeat(BOX_WIDTH));
            unsafe {
                if constants::VERBOSE {
                    println!("{}", serde_json::to_string_pretty(&response_value).unwrap());
                }
            }
        }
        "set_setpoint" => {
            let setpoint_raw = value.expect("Missing setpoint temperature.");
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

            let command_data =
                json!({"ZoneSetpoint":{"Index":zone_index,"Setpoint":setpoint_int}});
            send_command(client, &command_data)
                .expect("Failed to set setpoint");
            println!("╔{}╗", "═".repeat(BOX_WIDTH));
            println!("║ {:^ZONE_INNER_WIDTH$} ║", format!("Zone Control: {} Room", capitalized_zone_name));
            println!("╠{}╣", "═".repeat(BOX_WIDTH));
            let message = format!(
                "Set {} setpoint to {}°C.", capitalized_zone_name, format_temp(setpoint_int).cyan()
            );
            println!("║ {:<pw$} ║", message, pw = ZONE_INNER_WIDTH - get_visible_length(&message) + message.len());
            println!("╚{}╝", "═".repeat(BOX_WIDTH));
        }
        "set_max_air" => {
            let max_air_str = value.expect("Missing maximum airflow percentage.");
            let max_air: u8 = max_air_str
                .parse()
                .expect("Invalid max airflow percentage. Must be 0-100.");

            if max_air > 100 {
                eprintln!(
                    "{}Max airflow percentage '{}' invalid. Must be 0-100.",
                    "Error: ".red(),
                    max_air_str
                );
                exit(1);
            }

            let command_data = json!({"ZoneMaxAir":{"Index":zone_index,"MaxAir":max_air}});
            send_command(client, &command_data)
                .expect("Failed to set max airflow");
            println!("╔{}╗", "═".repeat(BOX_WIDTH));
            println!("║ {:^ZONE_INNER_WIDTH$} ║", format!("Zone Control: {} Room", capitalized_zone_name));
            println!("╠{}╣", "═".repeat(BOX_WIDTH));
            let message = format!(
                "Set {} max airflow to {}%.", capitalized_zone_name, max_air
            );
            println!("║ {:<pw$} ║", message, pw = ZONE_INNER_WIDTH - get_visible_length(&message) + message.len());
            println!("╚{}╝", "═".repeat(BOX_WIDTH));
        }
        "set_min_air" => {
            let min_air_str = value.expect("Missing minimum airflow percentage.");
            let min_air: u8 = min_air_str
                .parse()
                .expect("Invalid min airflow percentage. Must be 0-100.");

            if min_air > 100 {
                eprintln!(
                    "{}Min airflow percentage '{}' invalid. Must be 0-100.",
                    "Error: ".red(),
                    min_air_str
                );
                exit(1);
            }

            let command_data = json!({"ZoneMinAir":{"Index":zone_index,"MinAir":min_air}});
            send_command(client, &command_data)
                .expect("Failed to set min airflow");
            println!("╔{}╗", "═".repeat(BOX_WIDTH));
            println!("║ {:^ZONE_INNER_WIDTH$} ║", format!("Zone Control: {} Room", capitalized_zone_name));
            println!("╠{}╣", "═".repeat(BOX_WIDTH));
            let message = format!(
                "Set {} min airflow to {}%.", capitalized_zone_name, min_air
            );
            println!("║ {:<pw$} ║", message, pw = ZONE_INNER_WIDTH - get_visible_length(&message) + message.len());
            println!("╚{}╝", "═".repeat(BOX_WIDTH));
        }
        "set_name" => {
            let new_name = value.expect("Missing new zone name.");
            if new_name.len() > 15 {
                eprintln!(
                    "{}New zone name '{}' too long. Max 15 characters.",
                    "Error: ".red(),
                    new_name
                );
                exit(1);
            }
            let command_data = json!({"ZoneName":{"Index":zone_index,"Name":new_name}});
            send_command(client, &command_data)
                .expect("Failed to set zone name");
            println!("╔{}╗", "═".repeat(BOX_WIDTH));
            println!("║ {:^ZONE_INNER_WIDTH$} ║", format!("Zone Control: {} Room", capitalized_zone_name));
            println!("╠{}╣", "═".repeat(BOX_WIDTH));
            let message = format!(
                "Set {} name to '{}'.", capitalized_zone_name, new_name.green()
            );
            println!("║ {:<pw$} ║", message, pw = ZONE_INNER_WIDTH - get_visible_length(&message) + message.len());
            println!("╚{}╝", "═".repeat(BOX_WIDTH));
        }
        _ => {
            eprintln!(
                "{}{}{}",
                "Invalid action for zone '".red(),
                zone_name.red(),
                "'.\nZone Actions: on, off, open, auto, override, constant, status, temp, set_setpoint, set_max_air, set_min_air, set_name.".red()
            );
            exit(1);
        }
    }
}

pub fn get_all_zones_summary(client: &Client) {
    // Column widths for the summary table (visible characters)
    const NAME_COL_WIDTH: usize = 15; // e.g., "Dining_Room_Zone"
    const MODE_COL_WIDTH: usize = 8;  // e.g., "OVRIDE" or "UNKNOWN" - longest is 7, so 8 is good.
    const TEMP_COL_WIDTH: usize = 12; // e.g., "Temp: 20.0°C" (12 visible chars)
    const SETPOINT_COL_WIDTH: usize = 15; // e.g., "Setpoint: 20.0°C" (15 visible chars)
    const DAMPER_COL_WIDTH: usize = 13; // e.g., "Damper: 100%%" (13 visible chars)
    const STATUS_COL_WIDTH: usize = 25; // Max space for additional status, e.g., " DmpFlt SnsFlt LowBatt"

    // Calculate the total inner content width
    // Name (15) + ": " (2) + Mode (8) + " " (1) + Temp (12) + " " (1) + Setpoint (15) + " " (1) + Damper (13) + " " (1) + Status (25)
    const SUMMARY_INNER_CONTENT_WIDTH: usize = NAME_COL_WIDTH + 2 + MODE_COL_WIDTH + 1 + TEMP_COL_WIDTH + 1 + SETPOINT_COL_WIDTH + 1 + DAMPER_COL_WIDTH + 1 + STATUS_COL_WIDTH;

    // Total box width (inner content + 2 for "║ " and " ║")
    const SUMMARY_BOX_WIDTH: usize = SUMMARY_INNER_CONTENT_WIDTH + 1;

    println!("╔{}╗", "═".repeat(SUMMARY_BOX_WIDTH));
    println!("║{:^SUMMARY_INNER_CONTENT_WIDTH$} ║", "ZONE SUMMARY");
    println!("╠{}╣", "═".repeat(SUMMARY_BOX_WIDTH));

    // Fetch all zone data first
    let mut zones_data: Vec<crate::models::ZonesV2> = Vec::new(); // Store ZonesV2 structs

    for (&zone_index, zone_name) in ZONES.values().zip(ZONES.keys()) {
        let query_data = json!({ "iZoneV2Request": { "Type": 2, "No": zone_index, "No1": 0 } });

        let response_value = match query_izone_raw(client, &query_data) {
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
        let damper_part_raw = format!("Damper: {}%%", zone.damper_pos);
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