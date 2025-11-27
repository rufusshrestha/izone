// izone/src/commands/system.rs

use reqwest::blocking::Client;
use serde_json::json;
use colored::Colorize;
use std::process::exit;
use stringcase::Caser;
// Corrected imports for API functions and get_visible_length from helpers
use crate::api::{make_query_request, make_command_request};
use crate::constants;
use crate::helpers::{format_temp, get_colored_system_mode, get_fan_speed_text, get_visible_length, get_system_mode_value, get_fan_speed_value};
use crate::models::SystemV2Response;

// Removed: The `print_status_line` helper function has been removed as requested.

pub fn get_system_status(client: &Client) {
    const BOX_WIDTH: usize = 45; // Total width of the box's horizontal lines
    const PADDING_WIDTH: usize = BOX_WIDTH - 2; // Subtract 2 for the '║ ' and ' ║'
    const LABEL_WIDTH: usize = 24; // Width for the labels like "Aircon Power:", "Mode:", etc.

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "AIRCON STATUS", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let query_data = json!({ "iZoneV2Request": { "Type": 1, "No": 0, "No1": 0 } });

    let response_value = match make_query_request(client, query_data) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}", e.red());
            exit(1);
        }
    };

    let system_response: SystemV2Response = match serde_json::from_value(response_value.clone()) {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!(
                "{}",
                format!(
                    "Unexpected response from iZone controller at {}.\n\
                    Ensure your configuration has the correct iZone IP and the controller is reachable in your network.\n\
                    Error details: Failed to parse system status response - {}",
                    &**constants::IZONE_IP, e
                ).red()
            );
            exit(1);
        }
    };

    let sys_v2 = system_response.system_v2;

    let status_text = if sys_v2.sys_on {
        "ON".green().to_string()
    } else {
        "OFF".red().to_string()
    };

    let ac_error_text = if sys_v2.ac_error.trim() == "OK" {
        sys_v2.ac_error.green().to_string()
    } else {
        sys_v2.ac_error.red().to_string()
    };

    // Construct each line with internal padding for label and value
    let sys_on_line = format!("{:width$} {}", "Aircon Power:", status_text, width = LABEL_WIDTH);
    let sys_mode_line = format!("{:width$} {}", "Mode:", get_colored_system_mode(sys_v2.sys_mode), width = LABEL_WIDTH);
    let sys_fan_line = format!("{:width$} {}", "Fan Speed:", get_fan_speed_text(sys_v2.sys_fan).cyan(), width = LABEL_WIDTH);
    let sys_setpoint_line = format!("{:width$} {}°C", "Target Setpoint:", format_temp(sys_v2.setpoint), width = LABEL_WIDTH);
    let sys_temp_line = format!("{:width$} {}°C", "Controller Temperature:", format_temp(sys_v2.temp).cyan(), width = LABEL_WIDTH);
    let ac_error_line = format!("{:width$} {}", "System Check Status:", ac_error_text, width = LABEL_WIDTH - 1); // Adjusted width for System Check Status

    // Print each line, adjusting the external padding based on the visible length of the formatted line
    println!("║ {:<pw$} ║", sys_on_line, pw = PADDING_WIDTH - get_visible_length(&sys_on_line) + sys_on_line.len());
    println!("║ {:<pw$} ║", sys_mode_line, pw = PADDING_WIDTH - get_visible_length(&sys_mode_line) + sys_mode_line.len());
    println!("║ {:<pw$} ║", sys_fan_line, pw = PADDING_WIDTH - get_visible_length(&sys_fan_line) + sys_fan_line.len());
    // Adjust pw for these two lines to remove the extra space
    println!("║ {:<pw$} ║", sys_setpoint_line, pw = PADDING_WIDTH - get_visible_length(&sys_setpoint_line) + sys_setpoint_line.len() - 1);
    println!("║ {:<pw$} ║", sys_temp_line, pw = PADDING_WIDTH - get_visible_length(&sys_temp_line) + sys_temp_line.len() - 1);
    println!("║ {:<pw$} ║", ac_error_line, pw = PADDING_WIDTH - get_visible_length(&ac_error_line) + ac_error_line.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));

    unsafe {
        if constants::VERBOSE {
            // Use the original response_value for verbose output
            println!("Full SystemV2Response: {:#?}", response_value);
        }
    }
}

pub fn get_system_temperature(client: &Client) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Controller Temperature", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let query_data = json!({ "iZoneV2Request": { "Type": 1, "No": 0, "No1": 0 } });

    let response_value = match make_query_request(client, query_data) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}", e.red());
            exit(1);
        }
    };

    let system_response: SystemV2Response = match serde_json::from_value(response_value.clone()) {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!(
                "{}",
                format!(
                    "Unexpected response from iZone controller at {}.\n\
                    Ensure your configuration has the correct iZone IP and the controller is reachable in your network.\n\
                    Error details: Failed to parse controller temperature response - {}",
                    &**constants::IZONE_IP, e
                ).red()
            );
            exit(1);
        }
    };

    let temp_text = format_temp(system_response.system_v2.temp);
    let temp_line = format!("Current Controller Temperature: {}°C", temp_text.cyan());
    println!("║ {:<padding_width$} ║", temp_line, padding_width = PADDING_WIDTH -1 - get_visible_length(&temp_line) + temp_line.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));

    unsafe {
        if constants::VERBOSE {
            println!("{}", serde_json::to_string_pretty(&response_value).unwrap());
        }
    }
}

pub fn turn_on_ac(client: &Client) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"SysOn":1});
    make_command_request(client, command_data)
        .expect("Failed to turn on AC system");
    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let message = format!("AC system turned {}.", "ON".green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn turn_off_ac(client: &Client) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"SysOn":0});
    make_command_request(client, command_data)
        .expect("Failed to turn off AC system");
    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let message = format!("AC system turned {}.", "OFF".red());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_system_mode(client: &Client, mode_name: &str) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let mode_value = match get_system_mode_value(mode_name) {
        Some(value) => value,
        None => {
            // Construct the error message clearly to avoid character literal issues
            let error_message = format!(
                "Error: Unknown system mode '{}'.\nAvailable modes: auto, cool, heat, vent, dry.", // Updated modes list
                mode_name
            ).red().to_string();
            eprintln!("{}", error_message);
            exit(1);
        }
    };

    let command_data = json!({"SysMode": mode_value}); // Use SysMode command
    make_command_request(client, command_data)
        .expect(&format!("Failed to set system mode to '{}'", mode_name));

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let message = format!("System mode set to {}.", mode_name.to_pascal_case().cyan());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_system_fan(client: &Client, fan_speed_name: &str) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let fan_speed_value = match get_fan_speed_value(fan_speed_name) {
        Some(value) => value,
        None => {
            let error_message = format!(
                "Error: Unknown fan speed '{}'.\nAvailable fan speeds: auto, low, medium, high.",
                fan_speed_name
            ).red().to_string();
            eprintln!("{}", error_message);
            exit(1);
        }
    };

    let command_data = json!({"SysFan": fan_speed_value}); // Use SysFan command
    make_command_request(client, command_data)
        .expect(&format!("Failed to set system fan to '{}'", fan_speed_name));

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let message = format!("Aircon Fan Speed set to {}.", fan_speed_name.to_pascal_case().cyan());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

// ==================== COOLBREEZE COMMANDS ====================

pub fn set_coolbreeze_fan_speed(client: &Client, speed: u8) {
    if speed < 1 || speed > 100 {
        eprintln!("{}", "Error: Fan speed must be 1-100%".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"CoolbreezeFanSpeed": speed});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze fan speed");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = format!("Coolbreeze fan speed set to {}%.", speed.to_string().green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_rh_setpoint(client: &Client, rh: u8) {
    if rh < 10 || rh > 90 {
        eprintln!("{}", "Error: RH setpoint must be 10-90%".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"CoolbreezeRhSetpoint": rh});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze RH setpoint");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = format!("Coolbreeze RH setpoint set to {}%.", rh.to_string().green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_prewash(client: &Client, enable: bool, time_minutes: Option<u8>) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"CoolbreezePrewEn": enable_val});
    make_command_request(client, command_data.clone())
        .expect("Failed to set Coolbreeze prewash enable");

    if let Some(time) = time_minutes {
        if time < 1 || time > 60 {
            eprintln!("{}", "Error: Prewash time must be 1-60 minutes".red());
            exit(1);
        }
        let time_cmd = json!({"CoolbreezePrewTime": time});
        make_command_request(client, time_cmd)
            .expect("Failed to set Coolbreeze prewash time");
    }

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = if let Some(time) = time_minutes {
        format!("Coolbreeze prewash {} ({} min).", status, time.to_string().cyan())
    } else {
        format!("Coolbreeze prewash {}.", status)
    };
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_drain_after_prewash(client: &Client, enable: bool) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"CoolbreezeDrAfPrewEn": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze drain after prewash");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = format!("Drain after prewash {}.", status);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_drain_cycle(client: &Client, enable: bool, period_hours: Option<u16>) {
    const BOX_WIDTH: usize = 55;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"CoolbreezeDrCycEn": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze drain cycle enable");

    if let Some(hours) = period_hours {
        if hours < 1 || hours > 50 {
            eprintln!("{}", "Error: Drain cycle period must be 1-50 hours".red());
            exit(1);
        }
        let minutes = hours * 60;
        let period_cmd = json!({"CoolbreezeDrCycPer": minutes});
        make_command_request(client, period_cmd)
            .expect("Failed to set Coolbreeze drain cycle period");
    }

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = if let Some(hours) = period_hours {
        format!("Drain cycle {} (every {} hrs).", status, hours.to_string().cyan())
    } else {
        format!("Drain cycle {}.", status)
    };
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_postwash(client: &Client, enable: bool, time_minutes: Option<u8>) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"CoolbreezePostwEn": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze postwash enable");

    if let Some(time) = time_minutes {
        if time < 5 || time > 30 {
            eprintln!("{}", "Error: Postwash time must be 5-30 minutes".red());
            exit(1);
        }
        let time_cmd = json!({"CoolbreezePostwT": time});
        make_command_request(client, time_cmd)
            .expect("Failed to set Coolbreeze postwash time");
    }

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = if let Some(time) = time_minutes {
        format!("Postwash {} ({} min).", status, time.to_string().cyan())
    } else {
        format!("Postwash {}.", status)
    };
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_drain_before_postwash(client: &Client, enable: bool) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"CoolbreezeDrBfPostwEn": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze drain before postwash");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = format!("Drain before postwash {}.", status);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_inverter(client: &Client, enable: bool) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"CoolbreezeInverter": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze inverter");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = format!("Coolbreeze inverter {}.", status);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_resume_last(client: &Client, enable: bool) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"CoolbreezeResumeLast": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze resume last state");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = format!("Resume last state {}.", status);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_fan_max_auto(client: &Client, speed: u8) {
    if speed < 1 || speed > 100 {
        eprintln!("{}", "Error: Fan max auto speed must be 1-100%".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"CoolbreezeFanMaxAuto": speed});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze fan max auto");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = format!("Fan max auto speed set to {}%.", speed.to_string().green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_fan_max(client: &Client, speed: u8) {
    if speed < 1 || speed > 100 {
        eprintln!("{}", "Error: Fan max speed must be 1-100%".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"CoolbreezeFanMax": speed});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze fan max");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = format!("Fan max speed set to {}%.", speed.to_string().green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_exhaust_max(client: &Client, speed: u8) {
    if speed < 1 || speed > 100 {
        eprintln!("{}", "Error: Exhaust max speed must be 1-100%".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"CoolbreezeExhMax": speed});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze exhaust max");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = format!("Exhaust max speed set to {}%.", speed.to_string().green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_exhaust_enable(client: &Client, enable: bool) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"CoolbreezeExhEn": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze exhaust enable");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = format!("Exhaust mode {}.", status);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_control_sensor(client: &Client, sensor_type: &str) {
    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let sensor_val = match sensor_type.to_lowercase().as_str() {
        "screen" => 0,
        "remote" => 1,
        _ => {
            eprintln!("{}", "Error: Sensor type must be 'screen' or 'remote'".red());
            exit(1);
        }
    };

    let command_data = json!({"CoolbreezeCtrlSens": sensor_val});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze control sensor");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = format!("Control sensor set to {}.", sensor_type.cyan());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_temp_calibration(client: &Client, calibration: i16) {
    if calibration < -50 || calibration > 50 {
        eprintln!("{}", "Error: Calibration must be -50 to 50 (-5.0°C to +5.0°C)".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 55;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"CoolbreezeCalibTemp": calibration});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze temperature calibration");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let calib_val = (calibration as f32) / 10.0;
    let message = format!("Temperature calibration set to {}°C.", format!("{:+.1}", calib_val).green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_temp_deadband(client: &Client, deadband: u16) {
    if deadband < 100 || deadband > 500 {
        eprintln!("{}", "Error: Deadband must be 100-500 (1.0-5.0°C)".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 55;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"CoolbreezeDeadTemp": deadband});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze temperature deadband");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let deadband_val = (deadband as f32) / 100.0;
    let message = format!("Temperature deadband set to {}°C.", format!("{:.1}", deadband_val).green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_coolbreeze_auto_fan_max_time(client: &Client, time_minutes: u8) {
    if time_minutes > 60 {
        eprintln!("{}", "Error: Auto fan max time must be 0-60 minutes".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 55;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"CoolbreezeAutoFanMaxTime": time_minutes});
    make_command_request(client, command_data)
        .expect("Failed to set Coolbreeze auto fan max time");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Coolbreeze Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = format!("Auto fan max time set to {} minutes.", time_minutes.to_string().green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

// ==================== VENTILATION COMMANDS ====================

pub fn set_ventilation_rh_setpoint(client: &Client, rh: u8) {
    if rh < 5 || rh > 95 {
        eprintln!("{}", "Error: RH setpoint must be 5-95%".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"VentilationRfSetpoint": rh});
    make_command_request(client, command_data)
        .expect("Failed to set ventilation RH setpoint");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Ventilation Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = format!("RH setpoint set to {}%.", rh.to_string().green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_ventilation_vocs_setpoint(client: &Client, vocs: u16) {
    if vocs < 50 || vocs > 2500 {
        eprintln!("{}", "Error: VOCs setpoint must be 50-2500 ppb".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"VentilationVocsSetpoint": vocs});
    make_command_request(client, command_data)
        .expect("Failed to set ventilation VOCs setpoint");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Ventilation Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = format!("VOCs setpoint set to {} ppb.", vocs.to_string().green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_ventilation_eco2_setpoint(client: &Client, eco2: u16) {
    if eco2 < 500 || eco2 > 1500 {
        eprintln!("{}", "Error: eCO2 setpoint must be 500-1500 ppm".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"VentilationEco2Setpoint": eco2});
    make_command_request(client, command_data)
        .expect("Failed to set ventilation eCO2 setpoint");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Ventilation Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = format!("eCO2 setpoint set to {} ppm.", eco2.to_string().green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_ventilation_fan_stage_delay(client: &Client, delay_minutes: u8) {
    if delay_minutes < 3 || delay_minutes > 240 {
        eprintln!("{}", "Error: Fan stage delay must be 3-240 minutes".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"VentilationFanStageDelay": delay_minutes});
    make_command_request(client, command_data)
        .expect("Failed to set ventilation fan stage delay");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Ventilation Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = format!("Fan stage delay set to {} min.", delay_minutes.to_string().green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_ventilation_cycle_fan_off(client: &Client, enable: bool) {
    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"VentilationCycleFanOff": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set ventilation cycle fan off");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Ventilation Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = format!("Cycle fan off {}.", status);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_ventilation_use_rh_control(client: &Client, enable: bool) {
    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"VentilationUseRhControl": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set ventilation use RH control");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Ventilation Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = format!("Use RH control {}.", status);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_ventilation_use_vocs_control(client: &Client, enable: bool) {
    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"VentilationUseVcosControl": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set ventilation use VOCs control");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Ventilation Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = format!("Use VOCs control {}.", status);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_ventilation_use_eco2_control(client: &Client, enable: bool) {
    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"VentilationUseEco2Control": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set ventilation use eCO2 control");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "Ventilation Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = format!("Use eCO2 control {}.", status);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

// ==================== SYSTEM CONFIGURATION COMMANDS ====================

pub fn set_system_setpoint(client: &Client, setpoint: f32) {
    if setpoint < 15.0 || setpoint > 30.0 {
        eprintln!("{}", "Error: Setpoint must be 15.0-30.0°C".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let setpoint_int = (setpoint * 100.0).round() as u32;
    let command_data = json!({"SysSetpoint": setpoint_int});
    make_command_request(client, command_data)
        .expect("Failed to set system setpoint");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = format!("System setpoint set to {:.1}°C.", setpoint);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_sleep_timer(client: &Client, minutes: u32) {
    const BOX_WIDTH: usize = 45;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"SysSleepTimer": minutes});
    make_command_request(client, command_data)
        .expect("Failed to set sleep timer");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Control", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = if minutes == 0 {
        "Sleep timer disabled.".to_string()
    } else {
        format!("Sleep timer set to {} minutes.", minutes.to_string().green())
    };
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_economy_lock(client: &Client, enable: bool, min: Option<f32>, max: Option<f32>) {
    const BOX_WIDTH: usize = 55;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"EconomyLock": enable_val});
    make_command_request(client, command_data.clone())
        .expect("Failed to set economy lock");

    if let Some(min_temp) = min {
        if min_temp < 15.0 || min_temp > 30.0 {
            eprintln!("{}", "Error: Min temperature must be 15.0-30.0°C".red());
            exit(1);
        }
        let min_int = (min_temp * 100.0).round() as u32;
        let min_cmd = json!({"EconomyMin": min_int});
        make_command_request(client, min_cmd)
            .expect("Failed to set economy min");
    }

    if let Some(max_temp) = max {
        if max_temp < 15.0 || max_temp > 30.0 {
            eprintln!("{}", "Error: Max temperature must be 15.0-30.0°C".red());
            exit(1);
        }
        let max_int = (max_temp * 100.0).round() as u32;
        let max_cmd = json!({"EconomyMax": max_int});
        make_command_request(client, max_cmd)
            .expect("Failed to set economy max");
    }

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Configuration", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = if min.is_some() || max.is_some() {
        format!(
            "Economy lock {} ({:.1}-{:.1}°C).",
            status,
            min.unwrap_or(15.0),
            max.unwrap_or(30.0)
        )
    } else {
        format!("Economy lock {}.", status)
    };
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_filter_warning(client: &Client, months: u8) {
    if months != 0 && months != 3 && months != 6 && months != 12 {
        eprintln!("{}", "Error: Filter warning must be 0 (disabled), 3, 6, or 12 months".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"FilterWarn": months});
    make_command_request(client, command_data)
        .expect("Failed to set filter warning");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Configuration", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = if months == 0 {
        "Filter warning disabled.".to_string()
    } else {
        format!("Filter warning set to {} months.", months.to_string().green())
    };
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn reset_warning(client: &Client, warning_type: &str) {
    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"ResetWarning": warning_type});
    make_command_request(client, command_data)
        .expect("Failed to reset warning");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Configuration", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = format!("{} warning reset.", warning_type.to_pascal_case().green());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_damper_time(client: &Client, seconds: u8) {
    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"DamperTime": seconds});
    make_command_request(client, command_data)
        .expect("Failed to set damper time");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Configuration", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = if seconds == 0 {
        "Damper time set to automatic.".to_string()
    } else {
        format!("Damper time set to {} seconds.", seconds.to_string().green())
    };
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_auto_mode_deadband(client: &Client, deadband: f32) {
    if deadband < 0.75 || deadband > 5.0 {
        eprintln!("{}", "Error: Auto mode deadband must be 0.75-5.0°C".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 55;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let deadband_int = (deadband * 100.0).round() as u16;
    let command_data = json!({"AutoModeDeadB": deadband_int});
    make_command_request(client, command_data)
        .expect("Failed to set auto mode deadband");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Configuration", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let message = format!("Auto mode deadband set to {:.2}°C.", deadband);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_airflow_lock(client: &Client, enable: bool) {
    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"AirflowLock": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set airflow lock");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Configuration", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "locked".red() } else { "unlocked".green() };
    let message = format!("Airflow adjustment {}.", status);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_airflow_min_lock(client: &Client, enable: bool) {
    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"AirflowMinLock": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set airflow min lock");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Configuration", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "locked".red() } else { "unlocked".green() };
    let message = format!("Airflow min adjustment {}.", status);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_static_pressure(client: &Client, level: u8) {
    if level > 4 {
        eprintln!("{}", "Error: Static pressure level must be 0-4".red());
        exit(1);
    }

    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let command_data = json!({"StaticP": level});
    make_command_request(client, command_data)
        .expect("Failed to set static pressure");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Configuration", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let level_text = match level {
        0 => "lowest",
        4 => "highest",
        _ => "medium",
    };
    let message = format!("Static pressure set to level {} ({}).", level.to_string().green(), level_text.cyan());
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_open_dampers_when_off(client: &Client, enable: bool) {
    const BOX_WIDTH: usize = 55;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"OpenDampersWhenOff": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set open dampers when off");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Configuration", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = format!("Open dampers when off {}.", status);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_scrooge_mode(client: &Client, enable: bool) {
    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"ScroogeMode": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set scrooge mode");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Configuration", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".yellow() } else { "disabled".green() };
    let message = format!("Scrooge mode {}.", status);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_reverse_dampers(client: &Client, enable: bool) {
    const BOX_WIDTH: usize = 50;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"ReverseDampers": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set reverse dampers");

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Configuration", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".yellow() } else { "disabled".green() };
    let message = format!("Reverse dampers {}.", status);
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}

pub fn set_constant_control_by_area(client: &Client, enable: bool, area: Option<u16>) {
    const BOX_WIDTH: usize = 60;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;

    let enable_val = if enable { 1 } else { 0 };
    let command_data = json!({"CnstCtrlAreaEn": enable_val});
    make_command_request(client, command_data)
        .expect("Failed to set constant control by area enable");

    if let Some(area_val) = area {
        let area_cmd = json!({"CnstCtrlArea": area_val});
        make_command_request(client, area_cmd)
            .expect("Failed to set constant control area");
    }

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^padding_width$} ║", "System Configuration", padding_width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));
    let status = if enable { "enabled".green() } else { "disabled".red() };
    let message = if let Some(area_val) = area {
        format!("Constant control by area {} ({} m²).", status, area_val.to_string().cyan())
    } else {
        format!("Constant control by area {}.", status)
    };
    println!("║ {:<padding_width$} ║", message, padding_width = PADDING_WIDTH - get_visible_length(&message) + message.len());
    println!("╚{}╝", "═".repeat(BOX_WIDTH));
}