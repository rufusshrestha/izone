// izone/src/main.rs

use clap::{Args, Parser}; // Import Args
use reqwest::blocking::Client;

#[macro_use]
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
    #[command(name = "zone")]
    Zone(ZoneArgs),
}

#[derive(Args, Debug)]
struct ZoneArgs {
    /// Name of the zone to control (e.g., kitchen, master)
    #[arg(help = "Name of the zone (e.g., kitchen, master)")]
    name: String,

    #[command(subcommand)]
    action: ZoneAction,
}

#[derive(clap::Subcommand, Debug)]
enum ZoneAction {
    /// Get detailed status for the zone.
    Status,
    /// Get only the current temperature for the zone.
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
    SetSetpoint {
        #[arg(help = "Temperature in Celsius (e.g., 22.5)")]
        temperature: String,
    },
    /// Set zone max airflow (e.g., 90).
    SetMaxAir {
        #[arg(help = "Percentage (0-100)")]
        percentage: String,
    },
    /// Set zone min airflow (e.g., 5).
    SetMinAir {
        #[arg(help = "Percentage (0-100)")]
        percentage: String,
    },
    /// Set zone name (max 15 chars).
    SetName {
        #[arg(help = "New name for the zone (max 15 characters)")]
        new_name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    // Set the global VERBOSE flag
    unsafe {
        constants::VERBOSE = cli.verbose;
    }

    let client = Client::new();

    match cli.command {
        Commands::On => system::turn_on_ac(&client),
        Commands::Off => system::turn_off_ac(&client),
        Commands::Status => system::get_system_status(&client),
        Commands::SystemTemp => system::get_system_temp(&client),
        Commands::Zonesummary => zones::get_all_zones_summary(&client),
        Commands::Zone(args) => {
            let zone_name = args.name.to_lowercase();
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
