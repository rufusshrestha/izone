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
            let text = res.text().map_err(|e| format!("Failed to read response from iZone controller at {}: {}", &**IZONE_IP, e))?;
            // The controller replies with a bare "{ERROR}" (not JSON) when it rejects a request,
            // e.g. an index that doesn't exist on this system.
            if text.trim().eq_ignore_ascii_case("{error}") {
                if unsafe { VERBOSE } {
                    println!("Request URL: {}\nRequest Payload: {}\nResponse Body (Text): {}", query_url, payload, text);
                }
                return Err(format!("iZone controller rejected the request {} (does that zone/schedule exist on this system?)", payload));
            }
            let json_body: Value = serde_json::from_str(&text).map_err(|e| {
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
        Err(e) => Err({ crate::helpers::mark_network_error(); format!(
            "Failed to connect to iZone controller at {}.\n\
            Ensure the IP address is correct and the controller is reachable in your network.\n\
            Error details: {}",
            &**IZONE_IP, e
        ) }),
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
            if res_text.to_lowercase().contains("error") { // device replies "{ERROR}" on rejected commands
                eprintln!("{}", format!("Error: the iZone controller rejected the command {} (reply: {})", payload, res_text.trim()).red());
                exit(1); // Exit directly on API error for commands
            }

            if status.is_success() {
                Ok(()) // Return Ok(()) on success
            } else {
                // Exit cleanly rather than letting callers' `.expect()` panic.
                eprintln!("{}", format!("API command failed with status: {} and body: {}", status, res_text).red());
                exit(1);
            }
        }
        Err(e) => {
            // Exit cleanly rather than letting callers' `.expect()` panic.
            eprintln!("{}", format!(
                "Failed to connect to iZone controller at {}.\n\
                Ensure the IP address is correct and the controller is reachable in your network.\n\
                Error details: {}",
                &**IZONE_IP, e
            ).red());
            exit(1);
        }
    }
}

/// HTTP client with timeouts so an unreachable controller fails fast instead of hanging.
pub fn build_client() -> Client {
    Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_else(|_| Client::new())
}