// izone/src/main.rs

use clap::Parser;
use reqwest::blocking::Client;

// Import the lazy_static macro at the crate root
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
use crate::constants::VERBOSE; // Import VERBOSE for initial setting

/// Command-line arguments using Clap
#[derive(clap::Parser, Debug)]
#[command(name = "izone", author = "Your Name <your_email@example.com>", version = "2.0")]
#[command(about = "Airstream Control Script", long_about = None)]
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
    /// Get the overall system status (detailed).
    Status,
    /// Get only the current system temperature.
    SystemTemp,
    /// Get a summary of all zones.
    ZonesSummary,
    /// Control or query a specific zone.
    #[command(subcommand)]
    Zone(ZoneCommands),
}

#[derive(clap::Subcommand, Debug)]
enum ZoneCommands {
    /// Get detailed status for a specific zone.
    Status {
        #[arg(help = "Name of the zone (e.g., kitchen, master)")]
        name: String,
    },
    /// Get only the current temperature for a specific zone.
    Temp {
        #[arg(help = "Name of the zone (e.g., kitchen, master)")]
        name: String,
    },
    /// Set zone to Auto mode (typically 'ON').
    On {
        #[arg(help = "Name of the zone (e.g., kitchen, master)")]
        name: String,
    },
    /// Set zone to Close mode (typically 'OFF').
    Off {
        #[arg(help = "Name of the zone (e.g., kitchen, master)")]
        name: String,
    },
    /// Set zone to Open mode (manual open).
    Open {
        #[arg(help = "Name of the zone (e.g., kitchen, master)")]
        name: String,
    },
    /// Set zone to Auto mode (same as 'on').
    Auto {
        #[arg(help = "Name of the zone (e.g., kitchen, master)")]
        name: String,
    },
    /// Set zone to Override mode.
    Override {
        #[arg(help = "Name of the zone (e.g., kitchen, master)")]
        name: String,
    },
    /// Set zone to Constant mode.
    Constant {
        #[arg(help = "Name of the zone (e.g., kitchen, master)")]
        name: String,
    },
    /// Set zone setpoint (e.g., 22.5).
    SetSetpoint {
        #[arg(help = "Name of the zone (e.g., kitchen, master)")]
        name: String,
        #[arg(help = "Temperature in Celsius (e.g., 22.5)")]
        temperature: String,
    },
    /// Set zone max airflow (e.g., 90).
    SetMaxAir {
        #[arg(help = "Name of the zone (e.g., kitchen, master)")]
        name: String,
        #[arg(help = "Percentage (0-100)")]
        percentage: String,
    },
    /// Set zone min airflow (e.g., 5).
    SetMinAir {
        #[arg(help = "Name of the zone (e.g., kitchen, master)")]
        name: String,
        #[arg(help = "Percentage (0-100)")]
        percentage: String,
    },
    /// Set zone name (max 15 chars).
    SetName {
        #[arg(help = "Name of the zone (e.g., kitchen, master)")]
        name: String,
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
            system::get_system_temp(&client);
        }
        Commands::ZonesSummary => {
            zones::get_all_zones_summary(&client);
        }
        Commands::Zone(zone_command) => match zone_command {
            ZoneCommands::Status { name } => {
                zones::control_zone(&client, &name.to_lowercase(), "status", None);
            }
            ZoneCommands::Temp { name } => {
                zones::control_zone(&client, &name.to_lowercase(), "temp", None);
            }
            ZoneCommands::On { name } => {
                zones::control_zone(&client, &name.to_lowercase(), "on", None);
            }
            ZoneCommands::Off { name } => {
                zones::control_zone(&client, &name.to_lowercase(), "off", None);
            }
            ZoneCommands::Open { name } => {
                zones::control_zone(&client, &name.to_lowercase(), "open", None);
            }
            ZoneCommands::Auto { name } => {
                zones::control_zone(&client, &name.to_lowercase(), "auto", None);
            }
            ZoneCommands::Override { name } => {
                zones::control_zone(&client, &name.to_lowercase(), "override", None);
            }
            ZoneCommands::Constant { name } => {
                zones::control_zone(&client, &name.to_lowercase(), "constant", None);
            }
            ZoneCommands::SetSetpoint { name, temperature } => {
                zones::control_zone(
                    &client,
                    &name.to_lowercase(),
                    "set_setpoint",
                    Some(&temperature),
                );
            }
            ZoneCommands::SetMaxAir { name, percentage } => {
                zones::control_zone(
                    &client,
                    &name.to_lowercase(),
                    "set_max_air",
                    Some(&percentage),
                );
            }
            ZoneCommands::SetMinAir { name, percentage } => {
                zones::control_zone(
                    &client,
                    &name.to_lowercase(),
                    "set_min_air",
                    Some(&percentage),
                );
            }
            ZoneCommands::SetName { name, new_name } => {
                zones::control_zone(
                    &client,
                    &name.to_lowercase(),
                    "set_name",
                    Some(&new_name),
                );
            }
        },
    }
}