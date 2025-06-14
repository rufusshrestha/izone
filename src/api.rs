// izone/src/api.rs

use reqwest::blocking::Client;
use serde_json::Value;
use colored::Colorize;
use std::process::exit;
use crate::constants::{COMMAND_URL, QUERY_URL, VERBOSE};

/// Sends a command to the iZone system.
pub fn send_command(client: &Client, data: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let res_text = client
        .post(COMMAND_URL)
        .header("Content-Type", "application/json")
        .json(data)
        .send()?
        .text()?;

    if res_text.contains("error") {
        eprintln!("{}", format!("{}{}", "API Error: ".red(), res_text.normal()));
        exit(1); // Exit directly on API error, mimicking bash script
    }

    unsafe {
        if VERBOSE {
            println!("API Response: {}", res_text);
        }
    }
    Ok(())
}

/// Queries the iZone system and returns raw JSON as a `Value`.
pub fn query_izone_raw(client: &Client, data: &Value) -> Result<Value, Box<dyn std::error::Error>> {
    let res_value = client
        .post(QUERY_URL)
        .header("Content-Type", "application/json")
        .json(data)
        .send()?
        .json::<Value>()?;

    if res_value.to_string().contains("error") {
        eprintln!("{}", format!("{}{}", "API Error: ".red(), res_value.to_string().normal()));
        exit(1); // Exit directly on API error, mimicking bash script
    }
    Ok(res_value)
}