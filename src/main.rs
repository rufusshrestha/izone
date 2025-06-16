// izone/src/main.rs

use clap::{Args, Parser};
use reqwest::blocking::Client;
use std::process::exit;
use colored::Colorize; // New: Import Colorize trait for coloring strings

extern crate lazy_static;

// Declare modules
mod api;
mod commands;
mod constants;
mod helpers;
mod models;

// Use specific functions from modules
use crate::commands::system;
use crate::commands::zones;
use crate::commands::schedules; // New: Import schedules module

/// Command-line arguments using Clap
#[derive(Parser, Debug)]
#[command(name = "izone", author = "Rufus P. Shrestha <rufus.shrestha@outlook.com.au>", version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Airstream iZone Controller", long_about = None)]
struct Cli {
    #[arg(short = 'v', long = "verbose", help = "Show full JSON output for status queries and API responses.")]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Turn on the entire AC system.
    On,
    /// Turn off the entire AC system.
    Off,
    /// Get the overall Aircon status (detailed).
    Status,
    /// Get only the current system temperature.
    SystemTemp,
    /// Control or query a specific zone.
    #[clap(name = "zone")]
    Zone(ZoneArgs),
    /// Control the main Aircon system mode (e.g., Cool, Heat, Vent).
    #[clap(name = "mode")]
    Mode(ModeActionWrapper),
    /// Manage schedules (favourites).
    #[clap(name = "schedule", alias = "fav")] // Modified: Add alias "fav"
    Schedule(ScheduleArgs),
}

#[derive(Args, Debug)]
struct ZoneArgs {
    /// The name of the zone (e.g., "kitchen", "work"). Leave empty for summary of all zones.
    #[arg(help = "The name of the zone (e.g., \"kitchen\", \"work\"). Leave empty for summary of all zones.")]
    name: Option<String>,

    #[command(subcommand)]
    action: ZoneAction,
}

#[derive(clap::Subcommand, Debug)]
enum ZoneAction {
    /// Get detailed status for the zone.
    Status,
    /// Get only the current system temperature for the zone.
    Temp,
    /// Set zone to Auto mode (typically 'ON').
    On,
    /// Set zone to Close mode (typically 'OFF').
    Off,
    /// Set zone to Open mode (manual open).
    Open,
    /// Set zone to Auto mode (same as 'on').
    Auto,
    /// Set zone to Override mode.
    Override,
    /// Set zone to Constant mode.
    Constant,
    /// Set zone setpoint (e.g., 22.5).
    #[clap(name = "set-setpoint")]
    SetSetpoint {
        #[arg(help = "Temperature in Celsius (e.g., 22.5)")]
        temperature: String,
    },
    /// Set zone max airflow (e.g., 90).
    #[clap(name = "set-max-air")]
    SetMaxAir {
        #[arg(help = "Percentage (0-100)")]
        percentage: String,
    },
    /// Set zone min airflow (e.g., 5).
    #[clap(name = "set-min-air")]
    SetMinAir {
        #[arg(help = "Percentage (0-100)")]
        percentage: String,
    },
    /// Set zone name (max 15 chars).
    #[clap(name = "set-name")]
    SetName {
        #[arg(help = "New name for the zone (max 15 characters)")]
        new_name: String,
    },
    /// Get a summary of all zones.
    Summary,
}

#[derive(Args, Debug)]
struct ModeActionWrapper {
    #[command(subcommand)]
    action: ModeArgs,
}

#[derive(clap::Subcommand, Debug)]
enum ModeArgs {
    /// Set the system mode to Auto.
    Auto,
    /// Set the system mode to Cool.
    Cool,
    /// Set the system mode to Heat.
    Heat,
    /// Set the system mode to Vent.
    Vent,
    /// Set the system mode to Dry.
    Dry,
}

// New: ScheduleArgs for the 'schedule' command
#[derive(Args, Debug)]
struct ScheduleArgs {
    /// The index of the schedule (0-7).
    #[arg(short = 'i', long = "index", help = "The index of the schedule (0-7).")]
    index: Option<u8>,

    #[command(subcommand)]
    action: ScheduleAction,
}

// New: ScheduleAction enum for schedule subcommands
#[derive(clap::Subcommand, Debug)]
enum ScheduleAction {
    /// Get status for a specific schedule, or a summary of all schedules / favourites if no index is provided. // Modified: Updated help message
    Status,
    /// Set the name of a schedule.
    #[clap(name = "set-name")]
    SetName {
        #[arg(help = "New name for the schedule (max 15 characters)")]
        new_name: String,
    },
    /// Set the start and stop times for a schedule (HH:MM format).
    #[clap(name = "set-time")]
    SetTime {
        #[arg(help = "Start time in HH:MM format (e.g., 08:30)")]
        start_time: String,
        #[arg(help = "Stop time in HH:MM format (e.g., 17:00)")]
        stop_time: String,
    },
    /// Set the days a schedule is enabled (e.g., Mon, Tue, Wed).
    #[clap(name = "set-days")]
    SetDays {
        #[arg(num_args = 1.., help = "Days to enable (e.g., Mon Tue Fri)")]
        days: Vec<String>,
    },
    /// Set the AC mode and/or fan speed for a schedule.
    #[clap(name = "set-ac")]
    SetAc {
        #[arg(long, help = "AC mode (auto, cool, heat, vent, dry)")]
        mode: Option<String>,
        #[arg(long, help = "Fan speed (low, medium, high, auto, top, nongasheat)")]
        fan: Option<String>,
    },
    /// Enable a schedule.
    Enable,
    /// Disable a schedule.
    Disable,
    /// Set specific zone modes and setpoints within a schedule.
    #[clap(name = "set-zones")]
    SetZones {
        #[arg(
            num_args = 1..,
            help = "Zone settings: <zone_name>:<mode_val>:<setpoint> (e.g., kitchen:3:2250)"
        )]
        zone_settings: Vec<String>,
    },
}


fn main() {
    let cli = Cli::parse();

    unsafe {
        constants::VERBOSE = cli.verbose;
    }

    let client = Client::new();

    match cli.command {
        Commands::On => {
            system::turn_on_ac(&client);
        }
        Commands::Off => {
            system::turn_off_ac(&client);
        }
        Commands::Status => {
            system::get_system_status(&client);
        }
        Commands::SystemTemp => {
            system::get_system_temperature(&client);
        }
        Commands::Mode(mode_wrapper) => {
            let mode_string = match mode_wrapper.action {
                ModeArgs::Auto => "auto",
                ModeArgs::Cool => "cool",
                ModeArgs::Heat => "heat",
                ModeArgs::Vent => "vent",
                ModeArgs::Dry => "dry",
            };
            system::set_system_mode(&client, mode_string);
        }
        Commands::Zone(args) => {
            match args.action {
                ZoneAction::Status => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "status", None);
                    } else {
                        eprintln!("Error: 'izone zone status' requires a zone name. Did you mean 'izone zone summary'?");
                        exit(1);
                    }
                }
                ZoneAction::Temp => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "temp", None);
                    } else {
                        eprintln!("Error: 'izone zone temp' requires a zone name.");
                        exit(1);
                    }
                }
                ZoneAction::On => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "on", None);
                    } else {
                        eprintln!("Error: 'izone zone on' requires a zone name.");
                        exit(1);
                    }
                }
                ZoneAction::Off => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "off", None);
                    } else {
                        eprintln!("Error: 'izone zone off' requires a zone name.");
                        exit(1);
                    }
                }
                ZoneAction::Open => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "open", None);
                    } else {
                        eprintln!("Error: 'izone zone open' requires a zone name.");
                        exit(1);
                    }
                }
                ZoneAction::Auto => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "auto", None);
                    } else {
                        eprintln!("Error: 'izone zone auto' requires a zone name.");
                        exit(1);
                    }
                }
                ZoneAction::Override => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "override", None);
                    } else {
                        eprintln!("Error: 'izone zone override' requires a zone name.");
                        exit(1);
                    }
                }
                ZoneAction::Constant => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "constant", None);
                    } else {
                        eprintln!("Error: 'izone zone constant' requires a zone name.");
                        exit(1);
                    }
                }
                ZoneAction::SetSetpoint { temperature } => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "set_setpoint", Some(&temperature));
                    } else {
                        eprintln!("Error: 'izone zone set-setpoint' requires a zone name.");
                        exit(1);
                    }
                }
                ZoneAction::SetMaxAir { percentage } => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "set_max_air", Some(&percentage));
                    } else {
                        eprintln!("Error: 'izone zone set-max-air' requires a zone name.");
                        exit(1);
                    }
                }
                ZoneAction::SetMinAir { percentage } => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "set_min_air", Some(&percentage));
                    } else {
                        eprintln!("Error: 'izone zone set-min-air' requires a zone name.");
                        exit(1);
                    }
                }
                ZoneAction::SetName { new_name } => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "set_name", Some(&new_name));
                    } else {
                        eprintln!("Error: 'izone zone set-name' requires a zone name.");
                        exit(1);
                    }
                }
                ZoneAction::Summary => {
                    if args.name.is_some() {
                        eprintln!("Error: 'izone zone summary' does not take a zone name argument.");
                        exit(1);
                    }
                    zones::get_all_zones_summary(&client);
                }
            }
        }
        // New: Handle schedule commands
        Commands::Schedule(args) => {
            match args.action {
                ScheduleAction::Status => {
                    if let Some(index) = args.index {
                        schedules::get_schedule_status(&client, index);
                    } else {
                        schedules::get_all_schedules_summary(&client);
                    }
                }
                ScheduleAction::SetName { new_name } => {
                    if let Some(index) = args.index {
                        schedules::set_schedule_name(&client, index, &new_name);
                    } else {
                        eprintln!("{}", "Error: 'izone schedule set-name' requires a schedule index (-i <index>).".red());
                        exit(1);
                    }
                }
                ScheduleAction::SetTime { start_time, stop_time } => {
                    if let Some(index) = args.index {
                        let parse_time = |time_str: &str| -> Result<(u8, u8), String> {
                            let parts: Vec<&str> = time_str.split(':').collect();
                            if parts.len() == 2 {
                                let h = parts[0].parse::<u8>().map_err(|_| "Invalid hour format".to_string())?;
                                let m = parts[1].parse::<u8>().map_err(|_| "Invalid minute format".to_string())?;
                                if h <= 23 && m <= 59 {
                                    Ok((h, m))
                                } else {
                                    Err("Time values out of range (HH:0-23, MM:0-59)".to_string())
                                }
                            } else {
                                Err("Time format must be HH:MM".to_string())
                            }
                        };

                        let (start_h, start_m) = parse_time(&start_time).unwrap_or_else(|e| {
                            eprintln!("Error parsing start time: {}", e.red());
                            exit(1);
                        });
                        let (stop_h, stop_m) = parse_time(&stop_time).unwrap_or_else(|e| {
                            eprintln!("Error parsing stop time: {}", e.red());
                            exit(1);
                        });

                        schedules::set_schedule_time(&client, index, start_h, start_m, stop_h, stop_m);
                    } else {
                        eprintln!("{}", "Error: 'izone schedule set-time' requires a schedule index (-i <index>).".red());
                        exit(1);
                    }
                }
                ScheduleAction::SetDays { days } => {
                    if let Some(index) = args.index {
                        schedules::set_schedule_days(&client, index, days);
                    } else {
                        eprintln!("{}", "Error: 'izone schedule set-days' requires a schedule index (-i <index>).".red());
                        exit(1);
                    }
                }
                ScheduleAction::SetAc { mode, fan } => {
                    if let Some(index) = args.index {
                        let mode_str = mode.as_deref(); // Convert Option<String> to Option<&str>
                        let fan_str = fan.as_deref(); // Convert Option<String> to Option<&str>
                        schedules::set_schedule_mode_fan(&client, index, mode_str, fan_str);
                    } else {
                        eprintln!("{}", "Error: 'izone schedule set-ac' requires a schedule index (-i <index>).".red());
                        exit(1);
                    }
                }
                ScheduleAction::Enable => {
                    if let Some(index) = args.index {
                        schedules::enable_schedule(&client, index);
                    } else {
                        eprintln!("{}", "Error: 'izone schedule enable' requires a schedule index (-i <index>).".red());
                        exit(1);
                    }
                }
                ScheduleAction::Disable => {
                    if let Some(index) = args.index {
                        schedules::disable_schedule(&client, index);
                    } else {
                        eprintln!("{}", "Error: 'izone schedule disable' requires a schedule index (-i <index>).".red());
                        exit(1);
                    }
                }
                ScheduleAction::SetZones { zone_settings } => {
                    if let Some(index) = args.index {
                        let mut parsed_zone_settings = Vec::new();
                        for setting_str in zone_settings {
                            let parts: Vec<&str> = setting_str.split(':').collect();
                            if parts.len() == 3 {
                                let zone_name = parts[0].to_string();
                                let mode_val = parts[1].parse::<u8>().unwrap_or_else(|_| {
                                    eprintln!("Error: Invalid mode value in '{}'. Must be an integer.", setting_str.red());
                                    exit(1);
                                });
                                let setpoint = parts[2].parse::<u32>().unwrap_or_else(|_| {
                                    eprintln!("Error: Invalid setpoint value in '{}'. Must be an integer (e.g., 2250 for 22.5).", setting_str.red());
                                    exit(1);
                                });
                                parsed_zone_settings.push((zone_name, mode_val, setpoint));
                            } else {
                                eprintln!("Error: Invalid zone setting format '{}'. Expected format: <zone_name>:<mode_val>:<setpoint> (e.g., kitchen:3:2250)", setting_str.red());
                                exit(1);
                            }
                        }
                        schedules::set_schedule_zones(&client, index, parsed_zone_settings);
                    } else {
                        eprintln!("{}", "Error: 'izone schedule set-zones' requires a schedule index (-i <index>).".red());
                        exit(1);
                    }
                }
            }
        }
    }
}