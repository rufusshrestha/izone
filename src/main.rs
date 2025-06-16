// izone/src/main.rs

use clap::{Args, Parser}; // Import Args
use reqwest::blocking::Client;
use std::process::exit; // Import the exit function from std::process

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

/// Command-line arguments using Clap
#[derive(Parser, Debug)]
// Corrected the syntax for the #[command] attribute: removed misplaced ')' and ']'
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
    // Removed Zonesummary from top-level commands
    /// Control or query a specific zone.
    #[clap(name = "zone")] // Use #[clap] for subcommand naming
    Zone(ZoneArgs),
    /// Control the main Aircon system mode (e.g., Cool, Heat, Vent).
    #[clap(name = "mode")] // Use #[clap] for subcommand naming
    Mode(ModeActionWrapper), // Use a wrapper struct for subcommand nesting
}

#[derive(Args, Debug)]
struct ZoneArgs {
    /// The name of the zone (e.g., "kitchen", "work"). Leave empty for summary of all zones.
    #[arg(help = "The name of the zone (e.g., \"kitchen\", \"work\"). Leave empty for summary of all zones.")] // Corrected quotes
    name: Option<String>, // Made optional to allow for 'izone zone summary'

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
    #[clap(name = "set-setpoint")] // Corrected attribute
    SetSetpoint {
        #[arg(help = "Temperature in Celsius (e.g., 22.5)")]
        temperature: String,
    },
    /// Set zone max airflow (e.g., 90).
    #[clap(name = "set-max-air")] // Corrected attribute
    SetMaxAir {
        #[arg(help = "Percentage (0-100)")]
        percentage: String,
    },
    /// Set zone min airflow (e.g., 5).
    #[clap(name = "set-min-air")] // Corrected attribute
    SetMinAir {
        #[arg(help = "Percentage (0-100)")]
        percentage: String,
    },
    /// Set zone name (max 15 chars).
    #[clap(name = "set-name")] // Corrected attribute
    SetName {
        #[arg(help = "New name for the zone (max 15 characters)")]
        new_name: String,
    },
    /// Get a summary of all zones.
    Summary, // Added Summary subcommand under ZoneAction
}

// Wrapper struct to hold ModeArgs as a subcommand
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


fn main() {
    let cli = Cli::parse();

    unsafe {
        // Set the global VERBOSE flag based on CLI input
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
        Commands::Mode(mode_wrapper) => { // Destructure the new wrapper
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
                    // This now handles both 'izone zone <name> status' and 'izone zone status' (if name is None)
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "status", None);
                    } else {
                        // If no zone name is provided, default to getting summary.
                        // This might be better handled as a separate subcommand 'summary'
                        // but for now, it's mapped here.
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
                    // The 'name' argument should be None for 'summary'
                    if args.name.is_some() {
                        eprintln!("Error: 'izone zone summary' does not take a zone name argument.");
                        exit(1);
                    }
                    zones::get_all_zones_summary(&client);
                }
            }
        }
    }
}