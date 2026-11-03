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
pub fn make_command_request(client: &Client, payload: Value) -> Result<(), String> {
    // CLI behaviour: report the error and exit (callers rely on this).
    match try_command_request(client, payload) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("{}", e.red());
            exit(1);
        }
    }
}

/// Send a command to the controller and return an error instead of exiting.
/// Used by the web server, which must keep running when a command is rejected.
pub fn try_command_request(client: &Client, payload: Value) -> Result<(), String> {
    let command_url = format!("{}{}", &**IZONE_IP, COMMAND_URL_SUFFIX);

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let res = client
        .post(&command_url)
        .headers(headers)
        .json(&payload)
        .send()
        .map_err(|e| {
            format!(
                "Failed to connect to iZone controller at {}.\n\
                Ensure the IP address is correct and the controller is reachable in your network.\n\
                Error details: {}",
                &**IZONE_IP, e
            )
        })?;

    let status = res.status();
    // Commands reply with text such as "{OK}" or "{ERROR}", not JSON.
    let res_text = res.text().map_err(|e| {
        format!(
            "Unexpected response from iZone controller at {}.\n\
            Ensure your configuration has the correct iZone IP and the controller is reachable in your network.\n\
            Error details: {}",
            &**IZONE_IP, e
        )
    })?;

    if unsafe { VERBOSE } {
        println!("Request URL: {}", command_url);
        println!("Request Payload: {}", payload);
        println!("Response Status: {}", status);
        println!("Response Body (Text): {}", res_text);
    }

    if res_text.to_lowercase().contains("error") { // device replies "{ERROR}" on rejected commands
        return Err(format!("Error: the iZone controller rejected the command {} (reply: {})", payload, res_text.trim()));
    }
    if !status.is_success() {
        return Err(format!("API command failed with status: {} and body: {}", status, res_text));
    }
    Ok(())
}

/// HTTP client with timeouts so an unreachable controller fails fast instead of hanging.
pub fn build_client() -> Client {
    Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_else(|_| Client::new())
}