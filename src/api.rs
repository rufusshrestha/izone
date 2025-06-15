// izone/src/api.rs

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::Value; // Removed `json` as it's not used directly here

// Import the necessary constants and the global VERBOSE flag
use crate::constants::{IZONE_IP, QUERY_URL_SUFFIX, COMMAND_URL_SUFFIX, VERBOSE};
// Removed: use colored::Colorize; (Error handling is now passed to caller)
// Removed: use std::process::exit; (Error handling is now passed to caller)
// Removed: use crate::helpers::{self, deserialize_int_as_bool}; (Not used in this module)

/// Makes a POST request to the iZone API query endpoint.
///
/// Constructs the full URL using IZONE_IP and QUERY_URL_SUFFIX.
/// Includes JSON payload and sets Content-Type header.
/// Prints verbose output if the global VERBOSE flag is true.
/// Returns a `Result` to allow caller to handle errors gracefully.
pub fn make_query_request(client: &Client, payload: Value) -> Result<Value, String> {
    // Construct the full QUERY_URL using IZONE_IP and QUERY_URL_SUFFIX
    let query_url = format!("{}{}", IZONE_IP, QUERY_URL_SUFFIX);

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
            let json_body: Value = res.json().map_err(|e| format!("Failed to parse JSON: {}", e))?;

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
                Err(format!("API request failed with status: {} and body: {}", status, json_body))
            }
        }
        Err(e) => Err(format!("Failed to send request: {}", e)),
    }
}

/// Makes a POST request to the iZone API command endpoint.
///
/// Constructs the full URL using IZONE_IP and COMMAND_URL_SUFFIX.
/// Includes JSON payload and sets Content-Type header.
/// Prints verbose output if the global VERBOSE flag is true.
/// Returns a `Result` to allow caller to handle errors gracefully.
pub fn make_command_request(client: &Client, payload: Value) -> Result<Value, String> {
    // Construct the full COMMAND_URL using IZONE_IP and COMMAND_URL_SUFFIX
    let command_url = format!("{}{}", IZONE_IP, COMMAND_URL_SUFFIX);

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
            let json_body: Value = res.json().map_err(|e| format!("Failed to parse JSON: {}", e))?;

            // Access the global VERBOSE flag safely within the unsafe block
            if unsafe { VERBOSE } {
                println!("Request URL: {}", command_url);
                println!("Request Payload: {}", payload);
                println!("Response Status: {}", status);
                println!("Response Body: {}", serde_json::to_string_pretty(&json_body).unwrap_or_default());
            }

            if status.is_success() {
                Ok(json_body)
            } else {
                Err(format!("API command failed with status: {} and body: {}", status, json_body))
            }
        }
        Err(e) => Err(format!("Failed to send request: {}", e)),
    }
}