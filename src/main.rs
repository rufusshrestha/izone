// izone/src/main.rs

use clap::{Args, CommandFactory, FromArgMatches, Parser};
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
#[command(name = "izone", author = "Rufus P. Shrestha", version = env!("CARGO_PKG_VERSION"))]
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
    /// Get the overall Aircon status (detailed). (status|s)
    #[clap(name = "status", aliases = &["s"])]
    Status,
    /// Get current master controller temperature. (controller-temp|ct)
    #[clap(name = "controller-temp", alias = "ct")] // Changed from SystemTemp
    ControllerTemp, // Changed from SystemTemp
    /// Control or query a specific zone. (zone|z)
    #[clap(name = "zone", alias = "z")]
    Zone(ZoneArgs),
    /// Control the main Aircon system mode (e.g., Cool, Heat, Vent). (mode|m)
    #[clap(name = "mode", alias = "m")]
    Mode(ModeActionWrapper),
    /// Control the main Aircon system fan speed (e.g., Low, Medium, High, Auto). (fan|f)
    #[clap(name = "fan", alias = "f")] // Added new command for fan speed
    Fan(FanActionWrapper), // Added new subcommand variant for fan
    /// Manage favourites / schedules. (fav|schedule)
    #[clap(name = "fav", aliases = &["schedule"])]
    Schedule(ScheduleArgs),
    /// System configuration commands (config|cfg)
    #[clap(name = "config", alias = "cfg")]
    Config(ConfigArgs),
    /// Coolbreeze evaporative cooling control (coolbreeze|cb)
    #[clap(name = "coolbreeze", alias = "cb")]
    Coolbreeze(CoolbreezeArgs),
    /// Ventilation system control (ventilation|vent)
    #[clap(name = "ventilation", alias = "vent")]
    Ventilation(VentilationArgs),
}

#[derive(Args, Debug)]
struct ZoneArgs {
    /// The name of the zone (e.g., "kitchen", "work"). Leave empty for summary of all zones.
    #[arg(help = "The name of the zone (e.g., \"kitchen\", \"work\"). Leave empty for summary of all zones.")]
    name: Option<String>,

    /// Defaults to `summary` with no zone name, or `status` with a zone name.
    #[command(subcommand)]
    action: Option<ZoneAction>,
}

#[derive(clap::Subcommand, Debug)]
enum ZoneAction {
    /// Get detailed status for the zone. (status|s)
    #[clap(name = "status", aliases = &["s"])]
    Status,
    /// Get only the current temperature for the zone. (temp|t)
    #[clap(name = "temp", alias = "t")]
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
    /// Set zone setpoint (e.g., 22.5). (set-point | setpoint | sp)
    #[clap(name = "set-setpoint", aliases = &["sp", "setpoint"])]
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
    /// Get a summary of all zones. (summary | sum)
    #[clap(name = "summary", aliases = &["sum"])]
    Summary,
    /// Set zone balance max percentage (0-100 in steps of 5).
    #[clap(name = "set-balance-max")]
    SetBalanceMax {
        #[arg(help = "Percentage (0-100, steps of 5)")]
        percentage: u8,
    },
    /// Set zone balance min percentage (0-100 in steps of 5).
    #[clap(name = "set-balance-min")]
    SetBalanceMin {
        #[arg(help = "Percentage (0-100, steps of 5)")]
        percentage: u8,
    },
    /// Enable or disable damper skip for zone.
    #[clap(name = "set-damper-skip")]
    SetDamperSkip {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Set zone sensor calibration (-5.0 to +5.0°C).
    #[clap(name = "set-sensor-calibration", alias = "set-calib")]
    SetSensorCalibration {
        #[arg(help = "Calibration in degrees Celsius (-5.0 to +5.0)", allow_negative_numbers = true)]
        calibration: f32,
    },
    /// Enable or disable zone bypass mode.
    #[clap(name = "set-bypass")]
    SetBypass {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Set zone area in square meters.
    #[clap(name = "set-area")]
    SetArea {
        #[arg(help = "Area in square meters")]
        area: u8,
    },
}

#[derive(Args, Debug)]
struct ModeActionWrapper {
    /// Defaults to showing the current mode.
    #[command(subcommand)]
    action: Option<ModeArgs>,
}

#[derive(clap::Subcommand, Debug)]
enum ModeArgs {
    /// Set the system mode to Auto.
    #[clap(name = "auto", aliases = &["0", "5", "a"])]
    Auto,
    /// Set the system mode to Cool.
    #[clap(name = "cool", aliases = &["1", "c"])]
    Cool,
    /// Set the system mode to Heat.
    #[clap(name = "heat", aliases = &["2", "h"])]
    Heat,
    /// Set the system mode to Vent.
    #[clap(name = "vent", aliases = &["3", "v"])]
    Vent,
    /// Set the system mode to Dry.
    #[clap(name = "dry", aliases = &["4", "d"])]
    Dry,
}

// Updated: FanActionWrapper for the 'fan' command
#[derive(Args, Debug)]
struct FanActionWrapper {
    /// Defaults to showing the current fan speed.
    #[command(subcommand)]
    action: Option<FanArgs>,
}

// Updated: FanArgs enum for fan subcommands with aliases
#[derive(clap::Subcommand, Debug)]
enum FanArgs {
    /// Set the system fan speed to Auto. (or 4)
    #[clap(name = "auto", aliases = &["0", "4", "Auto", "a"])]
    Auto,
    /// Set the system fan speed to Low. (or 1)
    #[clap(name = "low", aliases = &["1", "Low", "l"])]
    Low,
    /// Set the system fan speed to Medium. (or med | 2)
    #[clap(name = "medium", aliases = &["2", "Med", "med", "m"])]
    Medium,
    /// Set the system fan speed to High. (or 3)
    #[clap(name = "high", aliases = &["3", "Hi", "hi", "High", "h"])]
    High,
    /// Set the system fan speed to Top (units that support it). (or 5)
    #[clap(name = "top", aliases = &["5", "Top", "t"])]
    Top,
}

// New: ScheduleArgs for the 'schedule' command
#[derive(Args, Debug)]
struct ScheduleArgs {
    /// The index of the schedule (0-8). May be given before or after the action.
    #[arg(short = 'i', long = "index", global = true, help = "The index of the schedule (0-8). May be given before or after the action.")]
    index: Option<u8>,

    /// Defaults to `status`.
    #[command(subcommand)]
    action: Option<ScheduleAction>,
}

// New: ScheduleAction enum for schedule subcommands
#[derive(clap::Subcommand, Debug)]
enum ScheduleAction {
    /// Get status for a specific schedule, or a summary of all favourites / schedules if no index is provided. (status|s)
    #[clap(name = "status", aliases = &["s"])]
    Status,
    /// Set the name of a schedule.
    #[clap(name = "set-name")]
    SetName {
        #[arg(help = "New name for the schedule (max 15 characters)")]
        new_name: String,
    },
    /// Set the start and stop times for a schedule (HH:MM or 24-hour HHMM, or N/A for no time).
    #[clap(name = "set-time")]
    SetTime {
        #[arg(help = "Start time as HH:MM or HHMM (e.g., 08:30 or 0830), or N/A for no start time")]
        start_time: String,
        #[arg(help = "Stop time as HH:MM or HHMM (e.g., 17:00 or 1700), or N/A for no stop time")]
        stop_time: String,
    },
    /// Set the days a schedule is enabled (e.g., Mon, Tue, Wed).
    #[clap(name = "set-days")]
    SetDays {
        #[arg(num_args = 1.., required = true, help = "Days to enable: Mon Tue Wed Thu Fri Sat Sun (or none)")]
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
    /// Enable a schedule. E.g, izone fav -i 0 enable
    Enable,
    /// Disable a schedule. E.g, izone fav -i 0 disable
    Disable,
    /// Set specific zone modes and setpoints within a schedule.
    #[clap(name = "set-zones")]
    SetZones {
        #[arg(
            num_args = 1..,
            required = true,
            help = "Zone settings: <zone_name>:<mode_val>:<setpoint> (e.g., kitchen:3:2250)"
        )]
        zone_settings: Vec<String>,
    },
}

// Config command arguments
#[derive(Args, Debug)]
struct ConfigArgs {
    #[command(subcommand)]
    action: ConfigAction,
}

#[derive(clap::Subcommand, Debug)]
enum ConfigAction {
    /// Set system setpoint temperature (15.0-30.0°C, 0.5°C steps).
    #[clap(name = "set-setpoint")]
    SetSetpoint {
        #[arg(help = "Temperature in Celsius (15.0-30.0, in 0.5 steps)")]
        temperature: f32,
    },
    /// Set sleep timer duration (0=off, 0.5h/30m, 1h/60m, 2h/120m, 4h/240m, 6h/360m, 8h/480m, 12h/720m).
    #[clap(name = "set-sleep-timer")]
    SetSleepTimer {
        #[arg(help = "Duration (e.g., 0, 30, 60, 120, 240, 360, 480, 720 or 0.5h, 1h, 2h, 4h, 6h, 8h, 12h)")]
        duration: String,
    },
    /// Configure economy lock and temperature limits.
    #[clap(name = "set-economy-lock")]
    SetEconomyLock {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
        #[arg(long, help = "Minimum temperature (15.0-30.0°C)")]
        min: Option<f32>,
        #[arg(long, help = "Maximum temperature (15.0-30.0°C)")]
        max: Option<f32>,
    },
    /// Set filter warning period in months (0, 3, 6, or 12).
    #[clap(name = "set-filter-warning")]
    SetFilterWarning {
        #[arg(help = "Months (0=disabled, 3, 6, or 12)")]
        months: u8,
    },
    /// Reset a warning (e.g., filter).
    #[clap(name = "reset-warning")]
    ResetWarning {
        #[arg(help = "Warning type (e.g., 'filter')")]
        warning_type: String,
    },
    /// Set damper control time in seconds (0 for automatic).
    #[clap(name = "set-damper-time")]
    SetDamperTime {
        #[arg(help = "Seconds (0 for automatic)")]
        seconds: u8,
    },
    /// Set auto mode deadband temperature (0.75-5.0°C).
    #[clap(name = "set-auto-deadband")]
    SetAutoModeDeadband {
        #[arg(help = "Temperature deadband in Celsius (0.75-5.0)")]
        deadband: f32,
    },
    /// Lock or unlock airflow adjustment (both min and max).
    #[clap(name = "set-airflow-lock")]
    SetAirflowLock {
        #[arg(help = "true to lock, false to unlock", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Lock or unlock minimum airflow adjustment only.
    #[clap(name = "set-airflow-min-lock")]
    SetAirflowMinLock {
        #[arg(help = "true to lock, false to unlock", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Set static pressure level (0-4: lowest to highest).
    #[clap(name = "set-static-pressure")]
    SetStaticPressure {
        #[arg(help = "Level (0-4)")]
        level: u8,
    },
    /// Open or close dampers when AC is off.
    #[clap(name = "set-open-dampers-when-off")]
    SetOpenDampersWhenOff {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Enable or disable scrooge mode.
    #[clap(name = "set-scrooge-mode")]
    SetScroogeMode {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Enable or disable reverse dampers.
    #[clap(name = "set-reverse-dampers")]
    SetReverseDampers {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Configure constant control by area.
    #[clap(name = "set-constant-control-area")]
    SetConstantControlArea {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
        #[arg(long, help = "Area in square meters")]
        area: Option<u16>,
    },
}

// Coolbreeze command arguments
#[derive(Args, Debug)]
struct CoolbreezeArgs {
    #[command(subcommand)]
    action: CoolbreezeAction,
}

#[derive(clap::Subcommand, Debug)]
enum CoolbreezeAction {
    /// Set fan speed (1-100%).
    #[clap(name = "set-fan-speed")]
    SetFanSpeed {
        #[arg(help = "Speed percentage (1-100)")]
        speed: u8,
    },
    /// Set humidity setpoint (10-90%).
    #[clap(name = "set-rh-setpoint")]
    SetRhSetpoint {
        #[arg(help = "Humidity percentage (10-90)")]
        rh: u8,
    },
    /// Configure prewash settings.
    #[clap(name = "set-prewash")]
    SetPrewash {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
        #[arg(long, help = "Time in minutes (1-60)")]
        time: Option<u8>,
    },
    /// Enable or disable drain after prewash.
    #[clap(name = "set-drain-after-prewash")]
    SetDrainAfterPrewash {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Configure drain cycle settings.
    #[clap(name = "set-drain-cycle")]
    SetDrainCycle {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
        #[arg(long, help = "Period in hours (1-50)")]
        period: Option<u16>,
    },
    /// Configure postwash settings.
    #[clap(name = "set-postwash")]
    SetPostwash {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
        #[arg(long, help = "Time in minutes (5-30)")]
        time: Option<u8>,
    },
    /// Enable or disable drain before postwash.
    #[clap(name = "set-drain-before-postwash")]
    SetDrainBeforePostwash {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Enable or disable inverter mode.
    #[clap(name = "set-inverter")]
    SetInverter {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Enable or disable resume last state.
    #[clap(name = "set-resume-last")]
    SetResumeLast {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Set maximum fan speed in auto mode (1-100%).
    #[clap(name = "set-fan-max-auto")]
    SetFanMaxAuto {
        #[arg(help = "Speed percentage (1-100)")]
        speed: u8,
    },
    /// Set maximum fan speed in manual mode (1-100%).
    #[clap(name = "set-fan-max")]
    SetFanMax {
        #[arg(help = "Speed percentage (1-100)")]
        speed: u8,
    },
    /// Set maximum exhaust fan speed (1-100%).
    #[clap(name = "set-exhaust-max")]
    SetExhaustMax {
        #[arg(help = "Speed percentage (1-100)")]
        speed: u8,
    },
    /// Enable or disable exhaust mode.
    #[clap(name = "set-exhaust-enable")]
    SetExhaustEnable {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Set control sensor (screen or remote).
    #[clap(name = "set-control-sensor")]
    SetControlSensor {
        #[arg(help = "Sensor type: 'screen' or 'remote'")]
        sensor_type: String,
    },
    /// Set temperature calibration (-5.0 to +5.0°C).
    #[clap(name = "set-temp-calibration")]
    SetTempCalibration {
        #[arg(help = "Calibration value (-50 to 50, divide by 10)", allow_negative_numbers = true)]
        calibration: i16,
    },
    /// Set temperature deadband (1.0-5.0°C).
    #[clap(name = "set-temp-deadband")]
    SetTempDeadband {
        #[arg(help = "Deadband value (100-500, divide by 100)")]
        deadband: u16,
    },
    /// Set auto fan max time in minutes (0-60).
    #[clap(name = "set-auto-fan-max-time")]
    SetAutoFanMaxTime {
        #[arg(help = "Time in minutes (0-60)")]
        time: u8,
    },
}

// Ventilation command arguments
#[derive(Args, Debug)]
struct VentilationArgs {
    #[command(subcommand)]
    action: VentilationAction,
}

#[derive(clap::Subcommand, Debug)]
enum VentilationAction {
    /// Set humidity setpoint (5-95%).
    #[clap(name = "set-rh-setpoint")]
    SetRhSetpoint {
        #[arg(help = "Humidity percentage (5-95)")]
        rh: u8,
    },
    /// Set VOCs setpoint (50-2500 ppb).
    #[clap(name = "set-vocs-setpoint")]
    SetVocsSetpoint {
        #[arg(help = "VOCs in ppb (50-2500)")]
        vocs: u16,
    },
    /// Set eCO2 setpoint (500-1500 ppm).
    #[clap(name = "set-eco2-setpoint")]
    SetEco2Setpoint {
        #[arg(help = "eCO2 in ppm (500-1500)")]
        eco2: u16,
    },
    /// Set fan stage delay in minutes (3-240).
    #[clap(name = "set-fan-stage-delay")]
    SetFanStageDelay {
        #[arg(help = "Delay in minutes (3-240)")]
        delay: u8,
    },
    /// Enable or disable cycle fan off.
    #[clap(name = "set-cycle-fan-off")]
    SetCycleFanOff {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Enable or disable RH control.
    #[clap(name = "set-use-rh-control")]
    SetUseRhControl {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Enable or disable VOCs control.
    #[clap(name = "set-use-vocs-control")]
    SetUseVocsControl {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
    /// Enable or disable eCO2 control.
    #[clap(name = "set-use-eco2-control")]
    SetUseEco2Control {
        #[arg(help = "true or false", action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
}


/// Attach usage examples (from helpers::EXAMPLES) to every command's --help.
fn with_examples(mut cmd: clap::Command, path: &str) -> clap::Command {
    if let Some(text) = helpers::format_examples(path) {
        cmd = cmd.after_help(text);
    }
    let names: Vec<String> = cmd.get_subcommands().map(|c| c.get_name().to_string()).collect();
    for name in names {
        let child_path = if path.is_empty() { name.clone() } else { format!("{} {}", path, name) };
        cmd = cmd.mut_subcommand(&name, |c| with_examples(c, &child_path));
    }
    cmd
}

/// Best-effort command path ("fav set-days") from raw args, resolving aliases.
/// Used when clap fails to parse, so we can still show the right examples.
fn command_path_from_args(cmd: &clap::Command, args: &[String]) -> String {
    let mut path = Vec::new();
    let mut current = cmd;
    let mut skip_next = false;
    for arg in args.iter().skip(1) {
        if skip_next { skip_next = false; continue; }
        if arg == "-i" || arg == "--index" { skip_next = true; continue; }
        if arg.starts_with('-') { continue; }
        if let Some(sub) = current.find_subcommand(arg) {
            path.push(sub.get_name().to_string());
            current = sub;
        } else if current.has_subcommands() && path.len() == 1 && path[0] == "zone" {
            continue; // zone name positional
        }
    }
    path.join(" ")
}

/// Command path from successfully parsed matches.
fn command_path_from_matches(m: &clap::ArgMatches) -> String {
    let mut path = Vec::new();
    let mut current = m;
    while let Some((name, sub)) = current.subcommand() {
        path.push(name.to_string());
        current = sub;
    }
    path.join(" ")
}

fn main() {
    let cmd = with_examples(Cli::command(), "");
    let args: Vec<String> = std::env::args().collect();
    let matches = match cmd.clone().try_get_matches_from(&args) {
        Ok(m) => m,
        Err(e) => {
            use clap::error::ErrorKind;
            let show_examples = !matches!(
                e.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            );
            let _ = e.print();
            if show_examples {
                helpers::print_examples(&command_path_from_args(&cmd, &args));
            }
            exit(e.exit_code());
        }
    };
    helpers::set_command_path(command_path_from_matches(&matches));
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    unsafe {
        constants::VERBOSE = cli.verbose;
    }

    let client = api::build_client();

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
        Commands::ControllerTemp => { // Changed from SystemTemp
            system::get_system_temperature(&client);
        }
        Commands::Mode(mode_wrapper) => {
            let Some(mode_action) = mode_wrapper.action else {
                system::show_system_mode_fan(&client, true);
                return;
            };
            let mode_string = match mode_action {
                ModeArgs::Auto => "auto",
                ModeArgs::Cool => "cool",
                ModeArgs::Heat => "heat",
                ModeArgs::Vent => "vent",
                ModeArgs::Dry => "dry",
            };
            system::set_system_mode(&client, mode_string);
        }
        Commands::Fan(fan_wrapper) => { // New: Handle fan command
            let Some(fan_action) = fan_wrapper.action else {
                system::show_system_mode_fan(&client, false);
                return;
            };
            let fan_speed_string = match fan_action {
                FanArgs::Auto => "auto",
                FanArgs::Low => "low",
                FanArgs::Medium => "medium",
                FanArgs::High => "high",
                FanArgs::Top => "top",
            };
            system::set_system_fan(&client, fan_speed_string);
        }
        Commands::Zone(args) => {
            // No action: summary of all zones, or status of the named zone.
            let action = args.action.unwrap_or(if args.name.is_some() { ZoneAction::Status } else { ZoneAction::Summary });
            match action {
                ZoneAction::Status => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "status", None);
                    } else {
                        eprintln!("Error: 'izone zone status' requires a zone name. Did you mean 'izone zone summary'?");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::Temp => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "temp", None);
                    } else {
                        eprintln!("Error: 'izone zone temp' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::On => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "on", None);
                    } else {
                        eprintln!("Error: 'izone zone on' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::Off => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "off", None);
                    } else {
                        eprintln!("Error: 'izone zone off' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::Open => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "open", None);
                    } else {
                        eprintln!("Error: 'izone zone open' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::Auto => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "auto", None);
                    } else {
                        eprintln!("Error: 'izone zone auto' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::Override => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "override", None);
                    } else {
                        eprintln!("Error: 'izone zone override' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::Constant => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "constant", None);
                    } else {
                        eprintln!("Error: 'izone zone constant' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::SetSetpoint { temperature } => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "set_setpoint", Some(&temperature));
                    } else {
                        eprintln!("Error: 'izone zone set-setpoint' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::SetMaxAir { percentage } => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "set_max_air", Some(&percentage));
                    } else {
                        eprintln!("Error: 'izone zone set-max-air' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::SetMinAir { percentage } => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "set_min_air", Some(&percentage));
                    } else {
                        eprintln!("Error: 'izone zone set-min-air' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::SetName { new_name } => {
                    if let Some(zone_name) = args.name {
                        zones::control_zone(&client, &zone_name.to_lowercase(), "set_name", Some(&new_name));
                    } else {
                        eprintln!("Error: 'izone zone set-name' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::Summary => {
                    if args.name.is_some() {
                        eprintln!("Error: 'izone zone summary' does not take a zone name argument.");
                        helpers::exit_with_usage();
                    }
                    zones::get_all_zones_summary(&client);
                }
                ZoneAction::SetBalanceMax { percentage } => {
                    if let Some(zone_name) = args.name {
                        zones::set_zone_balance_max(&client, &zone_name.to_lowercase(), percentage);
                    } else {
                        eprintln!("Error: 'izone zone set-balance-max' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::SetBalanceMin { percentage } => {
                    if let Some(zone_name) = args.name {
                        zones::set_zone_balance_min(&client, &zone_name.to_lowercase(), percentage);
                    } else {
                        eprintln!("Error: 'izone zone set-balance-min' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::SetDamperSkip { enable } => {
                    if let Some(zone_name) = args.name {
                        zones::set_zone_damper_skip(&client, &zone_name.to_lowercase(), enable);
                    } else {
                        eprintln!("Error: 'izone zone set-damper-skip' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::SetSensorCalibration { calibration } => {
                    if let Some(zone_name) = args.name {
                        if !calibration.is_finite() || !(-5.0..=5.0).contains(&calibration) {
                            eprintln!("{}", "Error: Calibration must be between -5.0°C and +5.0°C".red());
                            helpers::exit_with_usage();
                        }
                        let calibration_val = (calibration * 10.0).round() as i8;
                        zones::set_zone_sensor_calibration(&client, &zone_name.to_lowercase(), calibration_val);
                    } else {
                        eprintln!("Error: 'izone zone set-sensor-calibration' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::SetBypass { enable } => {
                    if let Some(zone_name) = args.name {
                        zones::set_zone_bypass(&client, &zone_name.to_lowercase(), enable);
                    } else {
                        eprintln!("Error: 'izone zone set-bypass' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
                ZoneAction::SetArea { area } => {
                    if let Some(zone_name) = args.name {
                        zones::set_zone_area(&client, &zone_name.to_lowercase(), area);
                    } else {
                        eprintln!("Error: 'izone zone set-area' requires a zone name.");
                        helpers::exit_with_usage();
                    }
                }
            }
        }
        // New: Handle schedule commands
        Commands::Schedule(args) => {
            match args.action.unwrap_or(ScheduleAction::Status) {
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
                        eprintln!("{}", "Error: 'izone fav set-name' requires a schedule index (-i <0-8>).".red());
                        helpers::exit_with_usage();
                    }
                }
                ScheduleAction::SetTime { start_time, stop_time } => {
                    if let Some(index) = args.index {
                        let parse_time = |time_str: &str| -> Result<(u8, u8), String> {
                            // "N/A" (or "none"/"-") means no time; 31:63 is the iZone "not set" sentinel.
                            if matches!(time_str.to_lowercase().as_str(), "n/a" | "na" | "none" | "-") {
                                return Ok((31, 63));
                            }
                            // Accept HH:MM (21:00, 8:30) or 24-hour HHMM / HMM (2100, 0830, 830).
                            let (h_str, m_str) = if let Some((h, m)) = time_str.split_once(':') {
                                (h, m)
                            } else if (3..=4).contains(&time_str.len()) && time_str.chars().all(|c| c.is_ascii_digit()) {
                                time_str.split_at(time_str.len() - 2)
                            } else {
                                return Err(format!("'{}' is not a valid time. Use HH:MM, HHMM (24-hour) or N/A", time_str));
                            };
                            let parts = [h_str, m_str];
                            if !parts[1].contains(':') && parts[1].len() == 2 {
                                let h = parts[0].parse::<u8>().map_err(|_| format!("Invalid hour in '{}'", time_str))?;
                                let m = parts[1].parse::<u8>().map_err(|_| format!("Invalid minute in '{}'", time_str))?;
                                if h <= 23 && m <= 59 {
                                    Ok((h, m))
                                } else {
                                    Err("Time values out of range (HH:0-23, MM:0-59)".to_string())
                                }
                            } else {
                                Err(format!("'{}' is not a valid time. Use HH:MM, HHMM (24-hour) or N/A", time_str))
                            }
                        };

                        let (start_h, start_m) = parse_time(&start_time).unwrap_or_else(|e| {
                            eprintln!("Error parsing start time: {}", e.red());
                            helpers::exit_with_usage();
                        });
                        let (stop_h, stop_m) = parse_time(&stop_time).unwrap_or_else(|e| {
                            eprintln!("Error parsing stop time: {}", e.red());
                            helpers::exit_with_usage();
                        });

                        schedules::set_schedule_time(&client, index, start_h, start_m, stop_h, stop_m);
                    } else {
                        eprintln!("{}", "Error: 'izone fav set-time' requires a schedule index (-i <0-8>).".red());
                        helpers::exit_with_usage();
                    }
                }
                ScheduleAction::SetDays { days } => {
                    if let Some(index) = args.index {
                        schedules::set_schedule_days(&client, index, days);
                    } else {
                        eprintln!("{}", "Error: 'izone fav set-days' requires a schedule index (-i <0-8>).".red());
                        helpers::exit_with_usage();
                    }
                }
                ScheduleAction::SetAc { mode, fan } => {
                    if let Some(index) = args.index {
                        let mode_str = mode.as_deref(); // Convert Option<String> to Option<&str>
                        let fan_str = fan.as_deref(); // Convert Option<String> to Option<&str>
                        schedules::set_schedule_mode_fan(&client, index, mode_str, fan_str);
                    } else {
                        eprintln!("{}", "Error: 'izone fav set-ac' requires a schedule index (-i <0-8>).".red());
                        helpers::exit_with_usage();
                    }
                }
                ScheduleAction::Enable => {
                    if let Some(index) = args.index {
                        schedules::enable_schedule(&client, index);
                    } else {
                        eprintln!("{}", "Error: 'izone fav enable' requires a schedule index (-i <0-8>).".red());
                        helpers::exit_with_usage();
                    }
                }
                ScheduleAction::Disable => {
                    if let Some(index) = args.index {
                        schedules::disable_schedule(&client, index);
                    } else {
                        eprintln!("{}", "Error: 'izone fav disable' requires a schedule index (-i <0-8>).".red());
                        helpers::exit_with_usage();
                    }
                }
                ScheduleAction::SetZones { zone_settings } => {
                    if let Some(index) = args.index {
                        let mut parsed_zone_settings = Vec::new();
                        for setting_str in zone_settings {
                            let parts: Vec<&str> = setting_str.split(':').collect();
                            if parts.len() == 3 {
                                let zone_name = parts[0].to_lowercase();
                                let mode_val = parts[1].parse::<u8>().unwrap_or_else(|_| {
                                    eprintln!("Error: Invalid mode value in '{}'. Must be an integer.", setting_str.red());
                                    helpers::exit_with_usage();
                                });
                                let setpoint = parts[2].parse::<u32>().unwrap_or_else(|_| {
                                    eprintln!("Error: Invalid setpoint value in '{}'. Must be an integer (e.g., 2250 for 22.5).", setting_str.red());
                                    helpers::exit_with_usage();
                                });
                                parsed_zone_settings.push((zone_name, mode_val, setpoint));
                            } else {
                                eprintln!("Error: Invalid zone setting format '{}'. Expected format: <zone_name>:<mode_val>:<setpoint> (e.g., kitchen:3:2250)", setting_str.red());
                                helpers::exit_with_usage();
                            }
                        }
                        schedules::set_schedule_zones(&client, index, parsed_zone_settings);
                    } else {
                        eprintln!("{}", "Error: 'izone fav set-zones' requires a schedule index (-i <0-8>).".red());
                        helpers::exit_with_usage();
                    }
                }
            }
        }
        // Config commands
        Commands::Config(args) => {
            match args.action {
                ConfigAction::SetSetpoint { temperature } => {
                    system::set_system_setpoint(&client, temperature);
                }
                ConfigAction::SetSleepTimer { duration } => {
                    system::set_sleep_timer(&client, &duration);
                }
                ConfigAction::SetEconomyLock { enable, min, max } => {
                    system::set_economy_lock(&client, enable, min, max);
                }
                ConfigAction::SetFilterWarning { months } => {
                    system::set_filter_warning(&client, months);
                }
                ConfigAction::ResetWarning { warning_type } => {
                    system::reset_warning(&client, &warning_type);
                }
                ConfigAction::SetDamperTime { seconds } => {
                    system::set_damper_time(&client, seconds);
                }
                ConfigAction::SetAutoModeDeadband { deadband } => {
                    system::set_auto_mode_deadband(&client, deadband);
                }
                ConfigAction::SetAirflowLock { enable } => {
                    system::set_airflow_lock(&client, enable);
                }
                ConfigAction::SetAirflowMinLock { enable } => {
                    system::set_airflow_min_lock(&client, enable);
                }
                ConfigAction::SetStaticPressure { level } => {
                    system::set_static_pressure(&client, level);
                }
                ConfigAction::SetOpenDampersWhenOff { enable } => {
                    system::set_open_dampers_when_off(&client, enable);
                }
                ConfigAction::SetScroogeMode { enable } => {
                    system::set_scrooge_mode(&client, enable);
                }
                ConfigAction::SetReverseDampers { enable } => {
                    system::set_reverse_dampers(&client, enable);
                }
                ConfigAction::SetConstantControlArea { enable, area } => {
                    system::set_constant_control_by_area(&client, enable, area);
                }
            }
        }
        // Coolbreeze commands
        Commands::Coolbreeze(args) => {
            match args.action {
                CoolbreezeAction::SetFanSpeed { speed } => {
                    system::set_coolbreeze_fan_speed(&client, speed);
                }
                CoolbreezeAction::SetRhSetpoint { rh } => {
                    system::set_coolbreeze_rh_setpoint(&client, rh);
                }
                CoolbreezeAction::SetPrewash { enable, time } => {
                    system::set_coolbreeze_prewash(&client, enable, time);
                }
                CoolbreezeAction::SetDrainAfterPrewash { enable } => {
                    system::set_coolbreeze_drain_after_prewash(&client, enable);
                }
                CoolbreezeAction::SetDrainCycle { enable, period } => {
                    system::set_coolbreeze_drain_cycle(&client, enable, period);
                }
                CoolbreezeAction::SetPostwash { enable, time } => {
                    system::set_coolbreeze_postwash(&client, enable, time);
                }
                CoolbreezeAction::SetDrainBeforePostwash { enable } => {
                    system::set_coolbreeze_drain_before_postwash(&client, enable);
                }
                CoolbreezeAction::SetInverter { enable } => {
                    system::set_coolbreeze_inverter(&client, enable);
                }
                CoolbreezeAction::SetResumeLast { enable } => {
                    system::set_coolbreeze_resume_last(&client, enable);
                }
                CoolbreezeAction::SetFanMaxAuto { speed } => {
                    system::set_coolbreeze_fan_max_auto(&client, speed);
                }
                CoolbreezeAction::SetFanMax { speed } => {
                    system::set_coolbreeze_fan_max(&client, speed);
                }
                CoolbreezeAction::SetExhaustMax { speed } => {
                    system::set_coolbreeze_exhaust_max(&client, speed);
                }
                CoolbreezeAction::SetExhaustEnable { enable } => {
                    system::set_coolbreeze_exhaust_enable(&client, enable);
                }
                CoolbreezeAction::SetControlSensor { sensor_type } => {
                    system::set_coolbreeze_control_sensor(&client, &sensor_type);
                }
                CoolbreezeAction::SetTempCalibration { calibration } => {
                    system::set_coolbreeze_temp_calibration(&client, calibration);
                }
                CoolbreezeAction::SetTempDeadband { deadband } => {
                    system::set_coolbreeze_temp_deadband(&client, deadband);
                }
                CoolbreezeAction::SetAutoFanMaxTime { time } => {
                    system::set_coolbreeze_auto_fan_max_time(&client, time);
                }
            }
        }
        // Ventilation commands
        Commands::Ventilation(args) => {
            match args.action {
                VentilationAction::SetRhSetpoint { rh } => {
                    system::set_ventilation_rh_setpoint(&client, rh);
                }
                VentilationAction::SetVocsSetpoint { vocs } => {
                    system::set_ventilation_vocs_setpoint(&client, vocs);
                }
                VentilationAction::SetEco2Setpoint { eco2 } => {
                    system::set_ventilation_eco2_setpoint(&client, eco2);
                }
                VentilationAction::SetFanStageDelay { delay } => {
                    system::set_ventilation_fan_stage_delay(&client, delay);
                }
                VentilationAction::SetCycleFanOff { enable } => {
                    system::set_ventilation_cycle_fan_off(&client, enable);
                }
                VentilationAction::SetUseRhControl { enable } => {
                    system::set_ventilation_use_rh_control(&client, enable);
                }
                VentilationAction::SetUseVocsControl { enable } => {
                    system::set_ventilation_use_vocs_control(&client, enable);
                }
                VentilationAction::SetUseEco2Control { enable } => {
                    system::set_ventilation_use_eco2_control(&client, enable);
                }
            }
        }
    }
}