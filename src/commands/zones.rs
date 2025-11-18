// izone/src/commands/zones.rs

use reqwest::blocking::Client;
use serde_json::{json, Value};
use colored::Colorize;
use std::process::exit;
use std::thread; // Added for sleep functionality
use std::time::Duration; // Added for duration specification

// Corrected imports for API functions and added get_visible_length from helpers
use crate::api::{make_query_request, make_command_request};
use crate::constants::{self, ZONES}; // Import the constants module itself, and ZONES
use crate::helpers::{format_temp, get_battery_level_text, get_zone_type_text, get_visible_length, get_sensor_fault_text}; // Added get_colored_system_mode
use crate::models::ZonesV2Response; // Removed ZoneListV2Response from here, will use full path where needed

// New helper function to get colored zone mode text
fn get_colored_zone_mode(mode: u8) -> String {
    match mode {
        1 => "OPEN".yellow().to_string(),
        2 => "OFF".red().to_string(),
        3 => "CLIMATE".green().to_string(),
        4 => "OVERRIDE".yellow().to_string(),
        5 => "CONSTANT".yellow().to_string(),
        _ => format!("UNKNOWN ({})", mode).normal().to_string(),
    }
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
        "open" => Some(json!({"ZoneMode":{"Index":zone_index,"Mode":1}})), // Open mode
        "off" => Some(json!({"ZoneMode":{"Index":zone_index,"Mode":2}})), // Off mode
        "on" | "auto" => Some(json!({"ZoneMode":{"Index":zone_index,"Mode":3}})), // Auto mode (ON)
        "override" => Some(json!({"ZoneMode":{"Index":zone_index,"Mode":4}})), // Override mode
        "constant" => Some(json!({"ZoneMode":{"Index":zone_index,"Mode":5}})), // Constant mode
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
        make_command_request(client, cmd_data.clone()) // Use .clone() to allow sending the same data twice
            .expect(&format!("Failed to execute '{}' for zone '{}'", action, zone_name));

        // If the action is "set_setpoint", call the API a second time after a delay
        if action == "set_setpoint" {
            thread::sleep(Duration::from_millis(100)); // 0.1 second delay
            make_command_request(client, cmd_data)
                .expect(&format!("Failed to re-execute '{}' for zone '{}' after delay", action, zone_name));
        }

        println!("╔{}╗", "═".repeat(BOX_WIDTH));
        println!("║ {:^padding_width$} ║", format!("Zone Control: {}", capitalized_zone_name), padding_width = PADDING_WIDTH);
        println!("╠{}╣", "═".repeat(BOX_WIDTH));
        let message = match action {
            "on" | "auto" => format!("Zone '{}' successfully set to {} mode.", capitalized_zone_name.green(), "Auto".green()),
            "off" => format!("Zone '{}' successfully turned {}.", capitalized_zone_name.green(), "OFF".red()),
            "open" => format!("Zone '{}' successfully set to {} mode.", capitalized_zone_name.green(), "OPEN".yellow()),
            "override" => format!("Zone '{}' successfully set to {} mode.", capitalized_zone_name.green(), "Override".yellow()),
            "constant" => format!("Zone '{}' successfully set to {} mode.", capitalized_zone_name.green(), "Constant".yellow()),
            "set_setpoint" => {
                let setpoint_raw = value.expect("Setpoint temperature is required for set_setpoint action.");
                format!("Setpoint for zone '{}' successfully set to {}°C", capitalized_zone_name.green(), setpoint_raw.green())
            },
            "set_max_air" => {
                let percentage_raw = value.expect("Max air percentage is required.");
                format!("Max air for zone '{}' successfully set to {}%.", capitalized_zone_name.green(), percentage_raw.green())
            },
            "set_min_air" => {
                let percentage_raw = value.expect("Min air percentage is required.");
                format!("Min air for zone '{}' successfully set to {}%.", capitalized_zone_name.green(), percentage_raw.green())
            },
            "set_name" => {
                let new_name = value.expect("New zone name is required.");
                format!("Zone '{}' successfully renamed to '{}'.", capitalized_zone_name.green(), new_name.green())
            },
            _ => format!("Action '{}' for zone '{}' successful.", action.green(), capitalized_zone_name.green()),
        };
        // Adjusted padding for messages in control_zone to fix off-by-one alignment
        let visible_message_length = get_visible_length(&message);
        let spaces_needed = PADDING_WIDTH - visible_message_length;
        println!("║ {}{}{} ║", message, " ".repeat(spaces_needed), "");
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

    // Changed Type from 3 to 2 for zone status queries
    let query_data = json!({ "iZoneV2Request": { "Type": 2, "No": zone_index, "No1": 0 } });

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

    // Changed to use the new get_colored_zone_mode for zone status display
    let mode_text = get_colored_zone_mode(zone.mode);
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
        let space_between_label_and_value = PADDING_WIDTH - LABEL_WIDTH - visible_value_len;

        println!(
            "║ {:<LABEL_WIDTH$}{}{} ║",
            label,
            " ".repeat(space_between_label_and_value),
            value
        );
    };


    print_line("Name:", zone.name.normal().to_string());
    print_line("Mode:", mode_text);
    print_line("Current Temperature:", format!("{}°C", temp_text).cyan().to_string());
    print_line("Setpoint:", format!("{}°C", setpoint_text).normal().to_string());
    print_line("Damper Position:", damper_pos_text.normal().to_string());
    print_line("Zone Type:", zone_type_text.normal().to_string());
    print_line("Sensor Type:", zone.sens_type.to_string().normal().to_string());
    print_line("Max Air:", format!("{}%", zone.max_air).normal().to_string());
    print_line("Min Air:", format!("{}%", zone.min_air).normal().to_string());
    print_line("Constant:", zone.constant.to_string().normal().to_string());
    print_line("Constant Air:", zone.constant_a.to_string().normal().to_string());
    print_line("Master:", zone.master.to_string().normal().to_string());
    print_line("Damper Fault:", zone.damper_fault.to_string().normal().to_string());
    print_line("Sensor Fault:", sensor_fault_text);
    print_line("Damper Skip:", zone.damper_skip.to_string().normal().to_string());
    print_line("Isense:", zone.isense.to_string().normal().to_string());
    print_line("Calibration:", zone.calibration.to_string().normal().to_string());
    print_line("RF Signal:", rf_signal_text.normal().to_string());
    print_line("Battery Voltage:", format!("{}V", batt_volt_text).normal().to_string());
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

    let query_data = json!({ "iZoneV2Request": { "Type": 2, "No": zone_index, "No1": 0 } });

    const BOX_WIDTH_TEMP: usize = 56; // Adjust based on example
    const PADDING_WIDTH_TEMP: usize = BOX_WIDTH_TEMP - 2;

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

    // Capitalize the first letter of zone_name for the header, and keep the rest as is
    let capitalized_zone_name_for_header = {
        let mut chars = zone_name.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        }
    };

    println!("╔{}╗", "═".repeat(BOX_WIDTH_TEMP));
    // Corrected the named argument for padding_width
    println!("║ {:^width$} ║", format!("Zone Temperature: {}", capitalized_zone_name_for_header), width = PADDING_WIDTH_TEMP);
    println!("╠{}╣", "═".repeat(BOX_WIDTH_TEMP));

    let message = format!("{} Room Temperature: {}°C", capitalized_zone_name_for_header, temp_text.cyan());
    // Corrected the named argument for padding_width
    let visible_message_length = get_visible_length(&message);
    let spaces_needed = PADDING_WIDTH_TEMP - visible_message_length;
    println!("║ {}{}{} ║", message, " ".repeat(spaces_needed), "");
    println!("╚{}╝", "═".repeat(BOX_WIDTH_TEMP));
}


pub fn get_all_zones_summary(client: &Client) {


    // Column widths for the summary table (visible characters)


    const NAME_COL_WIDTH: usize = 15;


    const MODE_COL_WIDTH: usize = 10;


    const TEMP_COL_WIDTH: usize = 10;


    const SETPOINT_COL_WIDTH: usize = 10;


    const DAMPER_COL_WIDTH: usize = 10;


    const MAX_AIR_COL_WIDTH: usize = 10;


    const STATUS_COL_WIDTH: usize = 25;





    // Header definition


    let header = format!(


        "{:<NAME_COL_WIDTH$} {:<MODE_COL_WIDTH$} {:<TEMP_COL_WIDTH$} {:<SETPOINT_COL_WIDTH$} {:<DAMPER_COL_WIDTH$} {:<MAX_AIR_COL_WIDTH$} {:<STATUS_COL_WIDTH$}",


        "ZONE", "MODE", "TEMP", "SETPOINT", "DAMPER", "MAX AIR", "STATUS"


    );





    let total_width = NAME_COL_WIDTH + 1 + MODE_COL_WIDTH + 1 + TEMP_COL_WIDTH + 1 + SETPOINT_COL_WIDTH + 1 + DAMPER_COL_WIDTH + 1 + MAX_AIR_COL_WIDTH + 1 + STATUS_COL_WIDTH;





    println!("╔{}╗", "═".repeat(total_width + 2));


    println!("║ {:^width$} ║", "ZONE SUMMARY", width = total_width);


    println!("╠{}╣", "═".repeat(total_width + 2));


    println!("║ {} ║", header);


    println!("╠{}╣", "═".repeat(total_width + 2));





    // Fetch all zone data first


    let mut zones_data: Vec<crate::models::ZonesV2> = Vec::new(); // Store ZonesV2 structs





    for (&zone_index, zone_name) in ZONES.values().zip(ZONES.keys()) {


        let query_data = json!({ "iZoneV2Request": { "Type": 2, "No": zone_index, "No1": 0 } });





        // Using make_query_request and handling its Result


        let response_value = match make_query_request(client, query_data) {


            Ok(val) => val,


            Err(e) => {


                let error_message = format!("{:<NAME_COL_WIDTH$} ERROR: {}", zone_name, e);


                println!("║ {:<width$} ║", error_message.red(), width = total_width + get_visible_length(&error_message.red().to_string()) - error_message.len());


                continue;


            }


        };





        let zones_v2_response: ZonesV2Response =


            match serde_json::from_value(response_value.clone()) {


                Ok(z) => z,


                Err(e) => {


                    let error_message = format!("{:<NAME_COL_WIDTH$} ERROR: {}", zone_name, e);


                    println!("║ {:<width$} ║", error_message.red(), width = total_width + get_visible_length(&error_message.red().to_string()) - error_message.len());


                    continue;


                }


            };


        zones_data.push(zones_v2_response.zones_v2);


    }





    // Sort zones alphabetically by name


    zones_data.sort_by(|a, b| a.name.cmp(&b.name));





    // Now iterate through the sorted zones to print the summary


    for zone in &zones_data {


        let zone_name_display = zone.name.replace(' ', "_");


        let zone_mode_text = get_colored_zone_mode(zone.mode);


        let temp_text = format!("{}°C", format_temp(zone.temp));


        let setpoint_text = format!("{}°C", format_temp(zone.setpoint));


        let damper_text = format!("{}%", zone.damper_pos);


        let max_air_text = format!("{}%", zone.max_air);





        let mut status_parts = Vec::new();


        if zone.damper_fault == 1 {


            status_parts.push("DmpFlt".red().to_string());


        }


        if zone.sensor_fault == 1 {


            status_parts.push("Sensor Fault".red().to_string());


        }


        if zone.batt_volt == 0 && zone.sens_type == 9 {


            status_parts.push("LowBatt".red().to_string());


        }





        let status_text = if status_parts.is_empty() {


            "Ok".green().to_string()


        } else {


            status_parts.join(" ")


        };





        let line = format!(


            "{:<name_col_width$} {:<mode_col_width$} {:<temp_col_width$} {:<setpoint_col_width$} {:<damper_col_width$} {:<max_air_col_width$} {:<status_col_width$}",


            zone_name_display,


            zone_mode_text,


            temp_text,


            setpoint_text,


            damper_text,


            max_air_text,


            status_text,


            name_col_width = NAME_COL_WIDTH,


            mode_col_width = MODE_COL_WIDTH + zone_mode_text.len() - get_visible_length(&zone_mode_text),


            temp_col_width = TEMP_COL_WIDTH + temp_text.len() - get_visible_length(&temp_text),


            setpoint_col_width = SETPOINT_COL_WIDTH + setpoint_text.len() - get_visible_length(&setpoint_text),


            damper_col_width = DAMPER_COL_WIDTH + damper_text.len() - get_visible_length(&damper_text),


            max_air_col_width = MAX_AIR_COL_WIDTH + max_air_text.len() - get_visible_length(&max_air_text),


            status_col_width = STATUS_COL_WIDTH + status_text.len() - get_visible_length(&status_text),


        );


        println!("║{}║", line);


    }


    println!("╚{}╝", "═".repeat(total_width + 2));


    unsafe {


        if constants::VERBOSE {


            // Reconstruct all_zones_responses to ensure sorted output if VERBOSE is true


            let sorted_all_zones_responses: Vec<Value> = zones_data.iter().map(|z| serde_json::to_value(&z).unwrap()).collect();


            println!("{}", serde_json::to_string_pretty(&sorted_all_zones_responses).unwrap());


        }


    }


}