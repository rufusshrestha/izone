// izone/src/api.rs

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::Value;

// Import the necessary constants and the global VERBOSE flag
// Note: VERBOSE is a static mut, so we access it via constants::VERBOSE
use crate::constants::{IZONE_IP, QUERY_URL_SUFFIX, COMMAND_URL_SUFFIX, VERBOSE};
use colored::Colorize; // Re-added Colorize for error messages in this module
use std::process::exit; // Re-added exit for immediate termination on API errors

/// Makes a POST request to the iZone API query endpoint.
///
/// Constructs the full URL using IZONE_IP and QUERY_URL_SUFFIX.
/// Includes JSON payload and sets Content-Type header.
/// Prints verbose output if the global VERBOSE flag is true.
/// Returns a `Result` to allow caller to handle errors gracefully.
pub fn make_query_request(client: &Client, payload: Value) -> Result<Value, String> {
    // Construct the full QUERY_URL using IZONE_IP and QUERY_URL_SUFFIX
    let query_url = format!("{}{}", &**IZONE_IP, QUERY_URL_SUFFIX);

    // Create headers
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Make the request
    let response = client
        .post(&query_url)
        .headers(headers)
        .json(&payload)
        .send();

    match response {
        Ok(res) => {
            let status = res.status();
            let json_body: Value = res.json().map_err(|e| {
                format!(
                    "Unexpected response from iZone controller at {}.\n\
                    Ensure your configuration has the correct iZone IP and the controller is reachable in your network.\n\
                    Error details: {}",
                    &**IZONE_IP, e
                )
            })?;

            // Access the global VERBOSE flag safely within the unsafe block
            if unsafe { VERBOSE } {
                println!("Request URL: {}", query_url);
                println!("Request Payload: {}", payload);
                println!("Response Status: {}", status);
                println!("Response Body: {}", serde_json::to_string_pretty(&json_body).unwrap_or_default());
            }

            if status.is_success() {
                Ok(json_body)
            } else {
                Err(format!("API query failed with status: {} and body: {}", status, json_body))
            }
        }
        Err(e) => Err(format!(
            "Failed to connect to iZone controller at {}.\n\
            Ensure the IP address is correct and the controller is reachable in your network.\n\
            Error details: {}",
            &**IZONE_IP, e
        )),
    }
}

/// Makes a POST request to the iZone API command endpoint.
///
/// Constructs the full URL using IZONE_IP and COMMAND_URL_SUFFIX.
/// Includes JSON payload and sets Content-Type header.
/// Prints verbose output if the global VERBOSE flag is true.
/// Returns `Result<(), String>` as command responses are often non-JSON or empty.
pub fn make_command_request(client: &Client, payload: Value) -> Result<(), String> { // Changed return type
    // Construct the full COMMAND_URL using IZONE_IP and COMMAND_URL_SUFFIX
    let command_url = format!("{}{}", &**IZONE_IP, COMMAND_URL_SUFFIX);

    // Create headers
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Make the request
    let response = client
        .post(&command_url)
        .headers(headers)
        .json(&payload)
        .send();

    match response {
        Ok(res) => {
            let status = res.status();
            // Changed to read response as text, as commands may not return JSON
            let res_text = res.text().map_err(|e| {
                format!(
                    "Unexpected response from iZone controller at {}.\n\
                    Ensure your configuration has the correct iZone IP and the controller is reachable in your network.\n\
                    Error details: {}",
                    &**IZONE_IP, e
                )
            })?;

            // Access the global VERBOSE flag safely within the unsafe block
            if unsafe { VERBOSE } {
                println!("Request URL: {}", command_url);
                println!("Request Payload: {}", payload);
                println!("Response Status: {}", status);
                println!("Response Body (Text): {}", res_text); // Print as text
            }

            // Check for common error indicators in the text response
            if res_text.contains("error") || res_text.contains("Error") {
                eprintln!("{}", format!("API Error: {}", res_text).red());
                exit(1); // Exit directly on API error for commands
            }

            if status.is_success() {
                Ok(()) // Return Ok(()) on success
            } else {
                Err(format!("API command failed with status: {} and body: {}", status, res_text))
            }
        }
        Err(e) => Err(format!(
            "Failed to connect to iZone controller at {}.\n\
            Ensure the IP address is correct and the controller is reachable in your network.\n\
            Error details: {}",
            &**IZONE_IP, e
        )),
    }
}