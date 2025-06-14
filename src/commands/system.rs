// izone/src/commands/system.rs

use reqwest::blocking::Client;
use serde_json::{json, Value};
use colored::Colorize;
use std::process::exit;

use crate::api::{query_izone_raw, send_command};
use crate::constants::VERBOSE;
use crate::helpers::{format_temp, get_colored_system_mode, get_fan_speed_text};
use crate::models::SystemV2Response;

pub fn get_system_status(client: &Client) {
    println!("--- System Status ---");
    let query_data = json!({ "iZoneV2Request": { "Type": 1, "No": 0, "No1": 0 } });

    let response_value = match query_izone_raw(client, &query_data) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}", format!("Failed to retrieve system status: {}", e).red());
            exit(1);
        }
    };

    let system_v2_response: SystemV2Response =
        match serde_json::from_value(response_value.clone()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}Failed to parse system status: {}", "Error: ".red(), e);
                exit(1);
            }
        };

    let sys_on_text = if system_v2_response.system_v2.sys_on {
        "ON".green()
    } else {
        "OFF".red()
    };

    let ac_error_text = if system_v2_response.system_v2.ac_error != "OK" {
        system_v2_response.system_v2.ac_error.red().to_string()
    } else {
        system_v2_response.system_v2.ac_error.normal().to_string()
    };

    println!("System Status: {}", sys_on_text);
    println!(
        "Mode:          {}",
        get_colored_system_mode(system_v2_response.system_v2.sys_mode)
    );
    println!(
        "Current Temp:  {}°C",
        format_temp(system_v2_response.system_v2.temp).cyan()
    );
    println!(
        "Setpoint:      {}°C",
        format_temp(system_v2_response.system_v2.setpoint)
    );
    println!(
        "Fan Speed:     {}",
        get_fan_speed_text(system_v2_response.system_v2.sys_fan)
    );
    println!("AC Error:      {}", ac_error_text);
    println!("-------------------");

    unsafe {
        if VERBOSE {
            println!("{}", serde_json::to_string_pretty(&response_value).unwrap());
        }
    }
}

pub fn get_system_temp(client: &Client) {
    println!("--- System Temperature ---");
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

    println!(
        "Current System Temperature: {}°C",
        format_temp(system_v2_response.system_v2.temp).cyan()
    );
    println!("--------------------------");

    unsafe {
        if VERBOSE {
            println!("{}", serde_json::to_string_pretty(&response_value).unwrap());
        }
    }
}

pub fn turn_on_ac(client: &Client) {
    let command_data = json!({"SysOn":1});
    send_command(client, &command_data)
        .expect("Failed to turn on AC system");
    println!("--- System Control ---");
    println!("AC system turned {}.", "ON".green());
    println!("--------------------");
}

pub fn turn_off_ac(client: &Client) {
    let command_data = json!({"SysOn":0});
    send_command(client, &command_data)
        .expect("Failed to turn off AC system");
    println!("--- System Control ---");
    println!("AC system turned {}.", "OFF".red());
    println!("--------------------");
}