// izone/src/commands/schedules.rs

use reqwest::blocking::Client;
use serde_json::json;
use colored::Colorize;
use std::process::exit;

use crate::api::{make_query_request, make_command_request};
use crate::constants::{self};
use crate::helpers::{
    format_temp, get_colored_system_mode, get_fan_speed_text, get_visible_length,
};
use crate::models::{SchedulesV2, SchedulesResponseWrapper}; // Import DaysEnabled

// Define the maximum number of schedules (favourtites) as per iZone API documentation
const MAX_SCHEDULES: u8 = 8; // Typically 0-7 or 1-8

pub fn get_schedule_status(client: &Client, schedule_index: u8) {
    if schedule_index >= MAX_SCHEDULES {
        eprintln!(
            "{}Error: Schedule index {} is out of valid range (0-{}).",
            "Error: ".red(),
            schedule_index,
            MAX_SCHEDULES - 1
        );
        exit(1);
    }

    const BOX_WIDTH: usize = 75;
    const PADDING_WIDTH: usize = BOX_WIDTH - 2;
    const LABEL_WIDTH: usize = 25;

    let query_data = json!({ "iZoneV2Request": { "Type": 3, "No": schedule_index, "No1": 0 } });

    println!("╔{}╗", "═".repeat(BOX_WIDTH));
    println!("║ {:^width$} ║", format!("SCHEDULE STATUS: {}", schedule_index), width = PADDING_WIDTH);
    println!("╠{}╣", "═".repeat(BOX_WIDTH));

    let response_value = match make_query_request(client, query_data) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}", format!("Error querying schedule status: {}", e).red());
            exit(1);
        }
    };

    let schedules_wrapper: SchedulesResponseWrapper = serde_json::from_value(response_value.clone())
        .expect("Failed to parse schedule status response");

    let schedule = schedules_wrapper.schedules_v2;

    // Helper to print formatted lines - CORRECTED
    let print_line = |label: &str, value: String| {
        let visible_value_len = get_visible_length(&value);
        let actual_padding_needed = PADDING_WIDTH - LABEL_WIDTH - visible_value_len;
        // Ensure actual_padding_needed is not negative
        let final_padding = actual_padding_needed.max(0);
        println!("║ {:<LABEL_WIDTH$}{:final_padding$}{} ║", label, "", value);
    };

    print_line("Name:", schedule.name.normal().to_string());
    print_line("Index:", schedule.index.to_string().normal().to_string());
    print_line("Active:", if schedule.active { "Yes".green() } else { "No".red() }.to_string());
    print_line("System Mode:", get_colored_system_mode(schedule.mode.unwrap_or(0)));
    print_line("System Fan:", get_fan_speed_text(schedule.fan.unwrap_or(0)));

    let start_h = schedule.start_h.unwrap_or(0);
    let start_m = schedule.start_m.unwrap_or(0);
    let stop_h = schedule.stop_h.unwrap_or(0);
    let stop_m = schedule.stop_m.unwrap_or(0);

    let start_time_str = if (start_h == 255 && start_m == 255) || (start_h == 31 && start_m == 63) {
        "N/A".to_string()
    } else {
        format!("{:02}:{:02}", start_h, start_m)
    };
    let stop_time_str = if (stop_h == 255 && stop_m == 255) || (stop_h == 31 && stop_m == 63) {
        "N/A".to_string()
    } else {
        format!("{:02}:{:02}", stop_h, stop_m)
    };

    print_line("Start Time:", start_time_str.normal().to_string());
    print_line("Stop Time:", stop_time_str.normal().to_string());

    // Access days from the nested struct
    let days_status = format!(
        "M:{} Tu:{} W:{} Th:{} F:{} Sa:{} Su:{}",
        if schedule.days_enabled.monday { "✓".green() } else { "x".red() },
        if schedule.days_enabled.tuesday { "✓".green() } else { "x".red() },
        if schedule.days_enabled.wednesday { "✓".green() } else { "x".red() },
        if schedule.days_enabled.thursday { "✓".green() } else { "x".red() },
        if schedule.days_enabled.friday { "✓".green() } else { "x".red() },
        if schedule.days_enabled.saturday { "✓".green() } else { "x".red() },
        if schedule.days_enabled.sunday { "✓".green() } else { "x".red() },
    ).to_string();
    print_line("Days Enabled:", days_status);

    if let Some(coolbreeze) = schedule.coolbreeze {
        println!("╠{}╣", "═".repeat(BOX_WIDTH));
        println!("║ {:^padding_width$} ║", "COOLBREEZE SETTINGS", padding_width = PADDING_WIDTH);
        println!("╠{}╣", "═".repeat(BOX_WIDTH));
        print_line("Unit Setpoint:", format_temp(coolbreeze.unit_setpoint).normal().to_string());
        print_line("Fan Speed:", format!("{}%", coolbreeze.fan_speed).normal().to_string());
        print_line("RH Setpoint:", format!("{}%", coolbreeze.rh_setpoint).normal().to_string());
    }

    if let Some(zones_settings) = schedule.zones {
        println!("╠{}╣", "═".repeat(BOX_WIDTH));
        println!("║ {:^padding_width$} ║", "ZONE SETTINGS", padding_width = PADDING_WIDTH);
        println!("╠{}╣", "═".repeat(BOX_WIDTH));
        for (i, zone_set) in zones_settings.iter().enumerate() {
            print_line(
                &format!("Zone {}:", i),
                format!(
                    "Mode:{} Setpoint:{}°C",
                    get_colored_system_mode(zone_set.mode),
                    format_temp(zone_set.setpoint)
                ).to_string(),
            );
        }
    }

    println!("╚{}╝", "═".repeat(BOX_WIDTH));

    unsafe {
        if constants::VERBOSE {
            println!("Full SchedulesV2Response: {:#?}", response_value);
        }
    }
}

pub fn get_all_schedules_summary(client: &Client) {
    // Current `SUMMARY_BOX_WIDTH` (96) determines the number of '═' characters.
    // This results in a total line length of 98 characters (96 '═' + 2 corners '╔'/'╗').
    // The inner content for the title needs 96 - 2 = 94 chars.
    // The inner content for column headers/data rows needs to be 94 chars as well.
    // CORRECTED: Adjusted DAYS_COL_WIDTH from 40 to 41 to ensure all lines are 98 chars total
    const SUMMARY_BOX_WIDTH: usize = 96;
    const SUMMARY_INNER_CONTENT_WIDTH: usize = SUMMARY_BOX_WIDTH - 2;

    const IDX_COL_WIDTH: usize = 5;
    const NAME_COL_WIDTH: usize = 15;
    const ACTIVE_COL_WIDTH: usize = 8;
    const TIME_COL_WIDTH: usize = 10;
    const DAYS_COL_WIDTH: usize = 41; // Adjusted from 40 to 41

    println!("╔{}╗", "═".repeat(SUMMARY_BOX_WIDTH));
    println!("║ {:^width$} ║", "SCHEDULE / FAVOURITES SUMMARY", width = SUMMARY_INNER_CONTENT_WIDTH);
    println!("╠{}╣", "═".repeat(SUMMARY_BOX_WIDTH));

    let mut schedules_data: Vec<SchedulesV2> = Vec::new();

    for i in 0..MAX_SCHEDULES {
        let query_data = json!({ "iZoneV2Request": { "Type": 3, "No": i, "No1": 0 } });

        let response_value = match make_query_request(client, query_data) {
            Ok(val) => val,
            Err(e) => {
                eprintln!(
                    "{}Error retrieving schedule {} status: {}",
                    "Error: ".red(),
                    i,
                    e
                );
                continue;
            }
        };

        match serde_json::from_value::<SchedulesResponseWrapper>(response_value.clone()) {
            Ok(schedules_wrapper) => {
                let schedule = schedules_wrapper.schedules_v2;
                schedules_data.push(schedule);
            }
            Err(e) => {
                eprintln!(
                    "{}Error parsing schedule {} data: {}",
                    "Error: ".red(),
                    i,
                    e
                );
                unsafe {
                    if constants::VERBOSE {
                        println!("Raw response for schedule {}: {:#?}", i, response_value);
                    }
                }
                continue;
            }
        };
    }

    if schedules_data.is_empty() {
        println!("║ {:^width$} ║", "No schedules configured.", width = SUMMARY_INNER_CONTENT_WIDTH);
    } else {
        schedules_data.sort_by_key(|s| s.index);

        println!(
            "║ {:<idx_w$} {:<name_w$} {:<active_w$} {:<time_w$} {:<time_w$} {:<days_w$} ║",
            "Idx", "Name", "Active", "Start", "Stop", "Days",
            idx_w = IDX_COL_WIDTH,
            name_w = NAME_COL_WIDTH,
            active_w = ACTIVE_COL_WIDTH,
            time_w = TIME_COL_WIDTH,
            days_w = DAYS_COL_WIDTH,
        );
        println!("╠{}╣", "═".repeat(SUMMARY_BOX_WIDTH));

        for schedule in schedules_data {
            let active_status_colored = if schedule.active { "ON".green() } else { "OFF".red() }.to_string();
            let start_h = schedule.start_h.unwrap_or(0);
            let start_m = schedule.start_m.unwrap_or(0);
            let stop_h = schedule.stop_h.unwrap_or(0);
            let stop_m = schedule.stop_m.unwrap_or(0);

            let start_time_str = if (start_h == 255 && start_m == 255) || (start_h == 31 && start_m == 63) {
                "N/A".to_string()
            } else {
                format!("{:02}:{:02}", start_h, start_m)
            };
            let stop_time_str = if (stop_h == 255 && stop_m == 255) || (stop_h == 31 && stop_m == 63) {
                "N/A".to_string()
            } else {
                format!("{:02}:{:02}", stop_h, stop_m)
            };

            // Access days from the nested struct
            let mut enabled_days_vec = Vec::new();
            if schedule.days_enabled.monday { enabled_days_vec.push("Mon".to_string()); }
            if schedule.days_enabled.tuesday { enabled_days_vec.push("Tue".to_string()); }
            if schedule.days_enabled.wednesday { enabled_days_vec.push("Wed".to_string()); }
            if schedule.days_enabled.thursday { enabled_days_vec.push("Thu".to_string()); }
            if schedule.days_enabled.friday { enabled_days_vec.push("Fri".to_string()); }
            if schedule.days_enabled.saturday { enabled_days_vec.push("Sat".to_string()); }
            if schedule.days_enabled.sunday { enabled_days_vec.push("Sun".to_string()); }

            let days_display = if enabled_days_vec.is_empty() {
                "None".to_string()
            } else {
                enabled_days_vec.join(", ")
            };
            let days_colored = days_display.cyan().to_string();

            // Calculate actual padding for colored strings based on visible length
            let active_visible_len = get_visible_length(&active_status_colored);
            let active_padding_needed = ACTIVE_COL_WIDTH.saturating_sub(active_visible_len);
            let active_display_str = format!("{}{}", active_status_colored, " ".repeat(active_padding_needed));

            let days_visible_len = get_visible_length(&days_colored);
            let days_padding_needed = DAYS_COL_WIDTH.saturating_sub(days_visible_len);
            let days_display_str = format!("{}{}", days_colored, " ".repeat(days_padding_needed));


            println!(
                "║ {:<idx_w$} {:<name_w$} {} {:<time_w$} {:<time_w$} {} ║",
                schedule.index.to_string(),
                schedule.name,
                active_display_str, // Use the manually padded string
                start_time_str,
                stop_time_str,
                days_display_str,   // Use the manually padded string
                idx_w = IDX_COL_WIDTH,
                name_w = NAME_COL_WIDTH,
                time_w = TIME_COL_WIDTH,
            );
        }
    }
    println!("╚{}╝", "═".repeat(SUMMARY_BOX_WIDTH));
}

pub fn set_schedule_name(client: &Client, schedule_index: u8, new_name: &str) {
    if schedule_index >= MAX_SCHEDULES {
        eprintln!(
            "{}Error: Schedule index {} is out of valid range (0-{}).",
            "Error: ".red(),
            schedule_index,
            MAX_SCHEDULES - 1
        );
        exit(1);
    }
    if new_name.len() > 16 {
        eprintln!("{}", "Error: Schedule name cannot exceed 15 characters.".red());
        exit(1);
    }

    let command_data = json!({"SchedName": {"Index": schedule_index, "Name": new_name}});
    make_command_request(client, command_data)
        .expect(&format!("Failed to set name for schedule {}", schedule_index));

    println!("Schedule {} name set to '{}'.", schedule_index.to_string().green(), new_name.green());
}

pub fn set_schedule_time(client: &Client, schedule_index: u8, start_h: u8, start_m: u8, stop_h: u8, stop_m: u8) {
    if schedule_index >= MAX_SCHEDULES {
        eprintln!("{}Error: Schedule index {} is out of valid range (0-{}).", "Error: ".red(), schedule_index, MAX_SCHEDULES - 1);
        exit(1);
    }
    if start_h > 23 || start_m > 59 || stop_h > 23 || stop_m > 59 {
        eprintln!("{}", "Error: Invalid time format. Hours must be 0-23, minutes 0-59.".red());
        exit(1);
    }

    let command_data = json!({
        "SchedSettings": {
            "Index": schedule_index,
            "StartH": start_h,
            "StartM": start_m,
            "StopH": stop_h,
            "StopM": stop_m,
            "DaysEnabled": {
                "M": 0, "Tu": 0, "W": 0, "Th": 0, "F": 0, "Sa": 0, "Su": 0
            }
        }
    });
    make_command_request(client, command_data)
        .expect(&format!("Failed to set time for schedule {}", schedule_index));

    println!(
        "Schedule {} time set to Start: {:02}:{:02}, Stop: {:02}:{:02}.",
        schedule_index.to_string().green(),
        start_h, start_m, stop_h, stop_m
    );
}

pub fn set_schedule_days(client: &Client, schedule_index: u8, days: Vec<String>) {
    if schedule_index >= MAX_SCHEDULES {
        eprintln!("{}Error: Schedule index {} is out of valid range (0-{}).", "Error: ".red(), schedule_index, MAX_SCHEDULES - 1);
        exit(1);
    }

    let mut m = 0;
    let mut tu = 0;
    let mut w = 0;
    let mut th = 0;
    let mut f = 0;
    let mut sa = 0;
    let mut su = 0;

    for day in days {
        match day.to_lowercase().as_str() {
            "m" | "mon" => m = 1,
            "tu" | "tue" => tu = 1,
            "w" | "wed" => w = 1,
            "th" | "thu" => th = 1,
            "f" | "fri" => f = 1,
            "sa" | "sat" => sa = 1,
            "su" | "sun" => su = 1,
            _ => {
                eprintln!("{}Warning: Unknown day '{}'. Skipping.", "Warning: ".yellow(), day);
            }
        }
    }

    let command_data = json!({
        "SchedSettings": {
            "Index": schedule_index,
            "StartH": 31,
            "StartM": 63,
            "StopH": 31,
            "StopM": 63,
            "DaysEnabled": {
                "M": m,
                "Tu": tu,
                "W": w,
                "Th": th,
                "F": f,
                "Sa": sa,
                "Su": su
            }
        }
    });

    make_command_request(client, command_data)
        .expect(&format!("Failed to set days for schedule {}", schedule_index));

    println!("Schedule {} days set.", schedule_index.to_string().green());
}

pub fn set_schedule_mode_fan(client: &Client, schedule_index: u8, mode: Option<&str>, fan: Option<&str>) {
    if schedule_index >= MAX_SCHEDULES {
        eprintln!("{}Error: Schedule index {} is out of valid range (0-{}).", "Error: ".red(), schedule_index, MAX_SCHEDULES - 1);
        exit(1);
    }

    let mut commands = Vec::new();

    if let Some(m_str) = mode {
        let mode_val = match crate::helpers::get_system_mode_value(m_str) {
            Some(v) => v,
            None => {
                eprintln!("{}Error: Invalid mode '{}'. Available modes: auto, cool, heat, vent, dry.", "Error: ".red(), m_str);
                exit(1);
            }
        };
        commands.push(json!({"SchedAcMode": {"Index": schedule_index, "Mode": mode_val}}));
    }

    if let Some(f_str) = fan {
        let fan_val = match f_str.to_lowercase().as_str() {
            "low" => 1,
            "medium" => 2,
            "high" => 3,
            "auto" => 4,
            "top" => 5,
            "nongasheat" => 99,
            _ => {
                eprintln!("{}Error: Invalid fan speed '{}'. Available: low, medium, high, auto, top, nongasheat.", "Error: ".red(), f_str);
                exit(1);
            }
        };
        commands.push(json!({"SchedAcFan": {"Index": schedule_index, "Fan": fan_val}}));
    }

    if commands.is_empty() {
        eprintln!("{}", "Error: No mode or fan speed provided to set for schedule.".red());
        exit(1);
    }

    for cmd in commands {
        make_command_request(client, cmd)
            .expect(&format!("Failed to set mode/fan for schedule {}", schedule_index));
    }
    println!("Schedule {} mode/fan set.", schedule_index.to_string().green());
}

pub fn enable_schedule(client: &Client, schedule_index: u8) {
    if schedule_index >= MAX_SCHEDULES {
        eprintln!("{}Error: Schedule index {} is out of valid range (0-{}).", "Error: ".red(), schedule_index, MAX_SCHEDULES - 1);
        exit(1);
    }
    let command_data = json!({"SchedEnable": {"Index": schedule_index, "Enabled": 1}});
    make_command_request(client, command_data)
        .expect(&format!("Failed to enable schedule {}", schedule_index));
    println!("Schedule {} enabled.", schedule_index.to_string().green());
}

pub fn disable_schedule(client: &Client, schedule_index: u8) {
    if schedule_index >= MAX_SCHEDULES {
        eprintln!("{}Error: Schedule index {} is out of valid range (0-{}).", "Error: ".red(), schedule_index, MAX_SCHEDULES - 1);
        exit(1);
    }
    let command_data = json!({"SchedEnable": {"Index": schedule_index, "Enabled": 0}});
    make_command_request(client, command_data)
        .expect(&format!("Failed to disable schedule {}", schedule_index));
    println!("Schedule {} disabled.", schedule_index.to_string().red());
}

pub fn set_schedule_zones(client: &Client, schedule_index: u8, zone_settings: Vec<(String, u8, u32)>) {
    if schedule_index >= MAX_SCHEDULES {
        eprintln!("{}Error: Schedule index {} is out of valid range (0-{}).", "Error: ".red(), schedule_index, MAX_SCHEDULES - 1);
        exit(1);
    }

    let mut zones_array = Vec::new();
    for (zone_name, mode_val, setpoint_raw) in zone_settings {
        let zone_index_map = match crate::constants::ZONES.get(zone_name.as_str()) {
            Some(&index) => index,
            None => {
                eprintln!(
                    "{}Error: Unknown zone '{}'. Skipping this zone for schedule {}.",
                    "Error: ".red(), zone_name, schedule_index
                );
                continue;
            }
        };

        if setpoint_raw < 1500 || setpoint_raw > 3000 {
            eprintln!(
                "{}Error: Setpoint '{}' for zone '{}' is out of valid range (15.0-30.0°C). Skipping.",
                "Error: ".red(), format_temp(setpoint_raw), zone_name
            );
            continue;
        }

        zones_array.push(json!({
            "Index": zone_index_map,
            "Mode": mode_val,
            "Setpoint": setpoint_raw
        }));
    }

    if zones_array.is_empty() {
        eprintln!("{}", "No valid zone settings provided for schedule.".red());
        exit(1);
    }

    let command_data = json!({
        "SchedZones": {
            "Index": schedule_index,
            "Zones": zones_array
        }
    });

    make_command_request(client, command_data)
        .expect(&format!("Failed to set zone settings for schedule {}", schedule_index));

    println!("Schedule {} zone settings updated.", schedule_index.to_string().green());
}
