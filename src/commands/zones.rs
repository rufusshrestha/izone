// izone/src/commands/zones.rs

use reqwest::blocking::Client;
use serde_json::{json, Value};
use colored::Colorize;
use std::process::exit;

use crate::api::{query_izone_raw, send_command};
use crate::constants::{VERBOSE, ZONES};
use crate::helpers::{format_temp, get_battery_level_text, get_zone_type_text};
use crate::models::ZonesV2Response;

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

    match action {
        "status" | "stat" => {
            println!("--- Zone Status: {} ---", capitalized_zone_name);
            let query_data = json!({ "iZoneV2Request": { "Type": 2, "No": zone_index, "No1": 0 } });

            let response_value = match query_izone_raw(client, &query_data) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("{}", format!("Failed to retrieve zone status: {}", e).red());
                    exit(1);
                }
            };

            let zones_v2_response: ZonesV2Response =
                match serde_json::from_value(response_value.clone()) {
                    Ok(z) => z,
                    Err(e) => {
                        eprintln!("{}Failed to parse zone status: {}", "Error: ".red(), e);
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

            println!("Zone Name:     {}", zone.name);
            println!("Status:        {}", zone_mode_text);
            println!("Zone Type:     {}", get_zone_type_text(zone.zone_type));
            println!(
                "Current Temp:  {}°C",
                format_temp(zone.temp).cyan()
            );
            println!("Setpoint:      {}°C", format_temp(zone.setpoint));
            println!("Damper Pos:    {}%", zone.damper_pos);
            println!("Max Airflow:   {}%", zone.max_air);
            println!("Min Airflow:   {}%", zone.min_air);
            println!("--- Sensor & Faults ---");
            println!("Sensor Type:   {}", zone.sens_type);
            println!("Sensor Fault:  {}", sensor_fault_text);
            println!("Damper Fault:  {}", damper_fault_text);
            println!("iSense Active: {}", zone.isense);
            println!("Calibration:   {}", zone.calibration);
            println!("RF Signal:     {}", zone.rf_signal);
            println!("Battery:       {}", get_battery_level_text(zone.batt_volt));
            println!("--- Advanced ---");
            println!("Constant:      {}", zone.constant);
            println!("Constant Active: {}", zone.constant_a);
            println!("Master Zone:   {}", zone.master);
            println!("Area:          {}m²", zone.area);
            println!("Bypass:        {}", zone.bypass);
            println!("Balance Max:   {}", zone.balance_max);
            println!("Balance Min:   {}", zone.balance_min);
            println!("Damper Skip:   {}", zone.damper_skip);
            println!("-------------------");

            unsafe {
                if VERBOSE {
                    println!("{}", serde_json::to_string_pretty(&response_value).unwrap());
                }
            }
        }
        "on" | "auto" => {
            let command_data = json!({"ZoneMode":{"Index":zone_index,"Mode":3}});
            send_command(client, &command_data)
                .expect("Failed to send ON/AUTO command");
            println!("--- Zone Control ---");
            println!(
                "Set zone {} to {}.",
                capitalized_zone_name,
                "ON (Auto Mode)".green()
            );
            println!("--------------------");
        }
        "off" => {
            let command_data = json!({"ZoneMode":{"Index":zone_index,"Mode":2}});
            send_command(client, &command_data)
                .expect("Failed to send OFF command");
            println!("--- Zone Control ---");
            println!(
                "Set zone {} to {}.",
                capitalized_zone_name,
                "OFF (Close Mode)".red()
            );
            println!("--------------------");
        }
        "open" => {
            let command_data = json!({"ZoneMode":{"Index":zone_index,"Mode":1}});
            send_command(client, &command_data)
                .expect("Failed to send OPEN command");
            println!("--- Zone Control ---");
            println!(
                "Set zone {} to {}.",
                capitalized_zone_name,
                "OPEN (Manual)".yellow()
            );
            println!("--------------------");
        }
        "override" => {
            let command_data = json!({"ZoneMode":{"Index":zone_index,"Mode":4}});
            send_command(client, &command_data)
                .expect("Failed to send OVERRIDE command");
            println!("--- Zone Control ---");
            println!(
                "Set zone {} to {}.",
                capitalized_zone_name,
                "OVERRIDE".yellow()
            );
            println!("--------------------");
        }
        "constant" => {
            let command_data = json!({"ZoneMode":{"Index":zone_index,"Mode":5}});
            send_command(client, &command_data)
                .expect("Failed to send CONSTANT command");
            println!("--- Zone Control ---");
            println!(
                "Set zone {} to {}.",
                capitalized_zone_name,
                "CONSTANT".yellow()
            );
            println!("--------------------");
        }
        "temp" | "temperature" => {
            println!("--- Zone Temperature: {} ---", capitalized_zone_name);
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

            println!(
                "{} Temperature: {}°C",
                zones_v2_response.zones_v2.name,
                format_temp(zones_v2_response.zones_v2.temp).cyan()
            );
            println!("-----------------------------------");
            unsafe {
                if VERBOSE {
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
            println!("--- Zone Control ---");
            println!(
                "Set {} setpoint to {}°C.",
                capitalized_zone_name,
                format_temp(setpoint_int).cyan()
            );
            println!("--------------------");
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
            println!("--- Zone Control ---");
            println!(
                "Set {} max airflow to {}%.",
                capitalized_zone_name, max_air
            );
            println!("--------------------");
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
            println!("--- Zone Control ---");
            println!(
                "Set {} min airflow to {}%.",
                capitalized_zone_name, min_air
            );
            println!("--------------------");
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
            println!("--- Zone Control ---");
            println!(
                "Set {} name to '{}'.",
                capitalized_zone_name,
                new_name.green()
            );
            println!("--------------------");
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
    println!("--- All Zones Summary ---");
    let mut all_zones_responses: Vec<Value> = Vec::new();

    for (zone_name, &zone_index) in ZONES.iter() {
        let query_data = json!({ "iZoneV2Request": { "Type": 2, "No": zone_index, "No1": 0 } });

        let response_value = match query_izone_raw(client, &query_data) {
            Ok(val) => val,
            Err(e) => {
                println!(
                    "{:<20}: {}{}",
                    zone_name.replace(' ', "_"),
                    format!("ERROR retrieving status: {}", e).red(),
                    ""
                );
                continue;
            }
        };

        all_zones_responses.push(response_value.clone());

        let zones_v2_response: ZonesV2Response =
            match serde_json::from_value(response_value.clone()) {
                Ok(z) => z,
                Err(e) => {
                    eprintln!("{}Failed to parse zone data: {}", "Error: ".red(), e);
                    println!(
                        "{:<20}: {}{}",
                        zone_name.replace(' ', "_"),
                        "ERROR parsing data".red(),
                        ""
                    );
                    continue;
                }
            };
        let zone = zones_v2_response.zones_v2;

        let zone_mode_colored_text = match zone.mode {
            1 => "OPEN".yellow().to_string(),
            2 => "OFF".red().to_string(),
            3 => "ON".green().to_string(),
            4 => "OVRIDE".yellow().to_string(),
            5 => "CONST".yellow().to_string(),
            _ => "UNKNOWN".normal().to_string(),
        };

        let mut additional_status = String::new();
        if zone.damper_fault == 1 {
            additional_status.push_str(&format!(" {}{}", "DmpFlt".red(), "".normal()));
        }
        if zone.sensor_fault == 1 {
            additional_status.push_str(&format!(" {}{}", "SnsFlt".red(), "".normal()));
        }
        // IMPORTANT: Replace '9' with the actual SensType code(s) for your wireless battery sensors.
        // If you have multiple wireless sensor types, use: (zone.sens_type == 9 || zone.sens_type == X)
        if zone.batt_volt == 0 && zone.sens_type == 9 {
            additional_status.push_str(&format!(" {}{}", "LowBatt".red(), "".normal()));
        }

        println!(
            "{:<20}: {:<17} Temp: {:<5}°C Setpoint: {:<5}°C Damper: {:<3}%% {}",
            zone.name.replace(' ', "_"),
            zone_mode_colored_text,
            format_temp(zone.temp),
            format_temp(zone.setpoint),
            zone.damper_pos,
            additional_status
        );
    }
    println!("-------------------------");

    unsafe {
        if VERBOSE {
            println!("{}", serde_json::to_string_pretty(&all_zones_responses).unwrap());
        }
    }
}