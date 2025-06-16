// izone/src/main.rs

use clap::{Args, Parser}; // Import Args
use reqwest::blocking::Client;

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
    /// Get a summary of all zones.
    Zonesummary,
    /// Control or query a specific zone.
    #[clap(name = "zone")] // Use #[clap] for subcommand naming
    Zone(ZoneArgs),
    /// Control the main Aircon system mode (e.g., Cool, Heat, Vent).
    #[clap(name = "mode")] // Use #[clap] for subcommand naming
    Mode(ModeActionWrapper), // Use a wrapper struct for subcommand nesting
}

#[derive(Args, Debug)]
struct ZoneArgs {
    /// The name of the zone (e.g., "kitchen", "work").
    #[arg(help = "The name of the zone (e.g., \"kitchen\", \"work\").")] // Corrected quotes
    name: String,

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
        Commands::Zonesummary => {
            zones::get_all_zones_summary(&client); // Adjusted function name to match existing code
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
            let zone_name = args.name.to_lowercase(); // Convert zone name to lowercase for consistent lookup
            match args.action {
                ZoneAction::Status => {
                    zones::control_zone(&client, &zone_name, "status", None);
                }
                ZoneAction::Temp => {
                    zones::control_zone(&client, &zone_name, "temp", None);
                }
                ZoneAction::On => {
                    zones::control_zone(&client, &zone_name, "on", None);
                }
                ZoneAction::Off => {
                    zones::control_zone(&client, &zone_name, "off", None);
                }
                ZoneAction::Open => {
                    zones::control_zone(&client, &zone_name, "open", None);
                }
                ZoneAction::Auto => {
                    zones::control_zone(&client, &zone_name, "auto", None);
                }
                ZoneAction::Override => {
                    zones::control_zone(&client, &zone_name, "override", None);
                }
                ZoneAction::Constant => {
                    zones::control_zone(&client, &zone_name, "constant", None);
                }
                ZoneAction::SetSetpoint { temperature } => {
                    zones::control_zone(&client, &zone_name, "set_setpoint", Some(&temperature));
                }
                ZoneAction::SetMaxAir { percentage } => {
                    zones::control_zone(&client, &zone_name, "set_max_air", Some(&percentage));
                }
                ZoneAction::SetMinAir { percentage } => {
                    zones::control_zone(&client, &zone_name, "set_min_air", Some(&percentage));
                }
                ZoneAction::SetName { new_name } => {
                    zones::control_zone(&client, &zone_name, "set_name", Some(&new_name));
                }
            }
        }
    }
}