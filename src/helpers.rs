// izone/src/helpers.rs

use colored::Colorize;
use serde::{Deserializer, Deserialize};
use serde::de::Error; // Import Error trait from serde::de

/// Formats temperature from iZone raw (e.g., 2100 -> 21.0)
pub fn format_temp<T: Into<f64>>(temp_raw: T) -> String {
    format!("{:.1}", temp_raw.into() / 100.0)
}

/// Converts raw system mode integer to human-readable colored text.
pub fn get_colored_system_mode(sys_mode: u8) -> String {
    match sys_mode {
        1 => "Cool".blue().to_string(),
        2 => "Heat".red().to_string(),
        3 => "Vent".white().to_string(),
        4 => "Dry".yellow().to_string(),
        5 => "Auto".cyan().to_string(),
        6 => "Exhaust".white().to_string(),
        7 => "PumpOnly".white().to_string(),
        _ => format!("Mode({})", sys_mode).normal().to_string(),
    }
}

/// Converts human-readable system mode string to its corresponding u8 value.
/// Limited to Auto, Cool, Heat, Vent, Dry.
pub fn get_system_mode_value(mode_name: &str) -> Option<u8> {
    match mode_name.to_lowercase().as_str() {
        "auto" => Some(5), // Auto is 5
        "cool" => Some(1), // Cool is 1
        "heat" => Some(2), // Heat is 2
        "vent" => Some(3), // Vent is 3
        "dry" => Some(4),  // Dry is 4
        _ => None,
    }
}

/// Converts raw fan integer to human-readable text.
pub fn get_fan_speed_text(sys_fan: u8) -> String {
    match sys_fan {
        1 => "Low".to_string(),
        2 => "Medium".to_string(),
        3 => "High".to_string(),
        4 => "Auto".to_string(),
        5 => "Top".to_string(),
        99 => "NonGasHeat".to_string(),
        _ => format!("Fan({})", sys_fan),
    }
}

/// Converts human-readable fan speed string to its corresponding u8 value.
/// Low, Medium, High, Auto, Top (SysFan_e).
pub fn get_fan_speed_value(fan_speed_name: &str) -> Option<u8> {
    match fan_speed_name.to_lowercase().as_str() {
        "low" => Some(1),
        "medium" => Some(2),
        "high" => Some(3),
        "auto" => Some(4),
        "top" => Some(5),
        _ => None,
    }
}

/// Converts ZoneType_e to human-readable text.
pub fn get_zone_type_text(type_code: u8) -> String {
    match type_code {
        1 => "Open/Close".to_string(),
        2 => "Constant".to_string(),
        3 => "Auto".to_string(),
        _ => format!("Type({})", type_code),
    }
}

/// Converts BatteryLevel_e (0 = Full, 1 = Half, 2 = Empty) to colored text.
pub fn get_battery_level_text(batt_code: u8) -> String {
    match batt_code {
        0 => "Full".green().to_string(),
        1 => "Half".yellow().to_string(),
        2 => "Empty".red().to_string(),
        _ => format!("Unknown({})", batt_code).normal().to_string(),
    }
}

/// Converts RfSignalLevel_e (0 = Full, 1 = Half, 2 = Quarter, 3 = None) to colored text.
pub fn get_rf_signal_text(level: u8) -> String {
    match level {
        0 => "Full".green().to_string(),
        1 => "Half".yellow().to_string(),
        2 => "Quarter".yellow().to_string(),
        3 => "None".red().to_string(),
        _ => format!("Unknown({})", level).normal().to_string(),
    }
}

/// Converts sensor fault code to human-readable colored text.
pub fn get_sensor_fault_text(fault_code: u8) -> String {
    if fault_code == 0 {
        "OK".green().to_string()
    } else {
        format!("FLT:{}", fault_code).red().to_string()
    }
}

/// Custom deserializer for booleans that are represented as 0 or 1 integers.
pub fn deserialize_int_as_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let i = i8::deserialize(deserializer)?;
    match i {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(D::Error::custom(format!("invalid boolean integer: {}", i))),
    }
}

#[allow(dead_code)] // Added this attribute to suppress the warning
pub fn get_visible_length(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        match c {
            '\x1b' => in_escape = true, // Start of an ANSI escape sequence
            'm' if in_escape => in_escape = false, // End of an ANSI escape sequence
            _ if !in_escape => len += 1, // Count only non-escape characters
            _ => {},
        }
    }
    len
}
// ==================== USAGE EXAMPLES ON ERROR ====================

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

static COMMAND_PATH: OnceLock<String> = OnceLock::new();
static NETWORK_ERROR: AtomicBool = AtomicBool::new(false);

/// Examples keyed by canonical command path ("" = top level, "zone", "fav set-days", ...).
/// Shown after every usage/validation error and in `--help`.
pub const EXAMPLES: &[(&str, &[&str])] = &[
    ("", &["izone status", "izone zone sum", "izone zone kitchen set-setpoint 22.5", "izone fav status", "izone fav -i 0 set-days Mon Tue Wed Thu Fri"]),
    ("on", &["izone on"]),
    ("off", &["izone off"]),
    ("status", &["izone status", "izone -v status   # also print raw JSON"]),
    ("controller-temp", &["izone controller-temp", "izone ct"]),
    ("mode", &["izone mode             # show current mode", "izone mode cool", "izone mode heat", "izone mode auto"]),
    ("mode auto", &["izone mode auto"]), ("mode cool", &["izone mode cool"]), ("mode heat", &["izone mode heat"]),
    ("mode vent", &["izone mode vent"]), ("mode dry", &["izone mode dry"]),
    ("fan", &["izone fan              # show current fan speed", "izone fan auto", "izone fan low", "izone fan high"]),
    ("fan auto", &["izone fan auto"]), ("fan low", &["izone fan low"]), ("fan medium", &["izone fan medium"]),
    ("fan high", &["izone fan high"]), ("fan top", &["izone fan top"]),
    // zone: name comes before the action
    ("zone status", &["izone zone kitchen status", "izone zone 0 status       # by controller index (Idx in zone sum)", "izone zone kitchen            # same (status is the default)"]),
    ("zone temp", &["izone zone kitchen temp"]),
    ("zone on", &["izone zone kitchen on   # climate control (auto)"]),
    ("zone off", &["izone zone kitchen off   # close", "izone zone 0 off         # same, by controller index (Idx in zone sum)"]),
    ("zone open", &["izone zone kitchen open"]),
    ("zone auto", &["izone zone kitchen auto"]),
    ("zone override", &["izone zone kitchen override"]),
    ("zone constant", &["izone zone kitchen constant"]),
    ("zone set-setpoint", &["izone zone kitchen set-setpoint 22.5   # 15.0-30.0, 0.5 steps"]),
    ("zone set-max-air", &["izone zone kitchen set-max-air 80   # 0-100, steps of 5"]),
    ("zone set-min-air", &["izone zone kitchen set-min-air 10   # 0-100, steps of 5"]),
    ("zone set-name", &["izone zone kitchen set-name \"Kitchen\"   # max 15 chars"]),
    ("zone summary", &["izone zone summary", "izone zone sum", "izone zone                    # same (summary is the default)"]),
    ("zone set-balance-max", &["izone zone kitchen set-balance-max 90   # steps of 5, > balance min"]),
    ("zone set-balance-min", &["izone zone kitchen set-balance-min 10   # steps of 5, < balance max"]),
    ("zone set-damper-skip", &["izone zone kitchen set-damper-skip true", "izone zone kitchen set-damper-skip off"]),
    ("zone set-sensor-calibration", &["izone zone kitchen set-sensor-calibration -0.5   # -5.0 to +5.0 °C"]),
    ("zone set-bypass", &["izone zone kitchen set-bypass true"]),
    ("zone set-area", &["izone zone kitchen set-area 20   # m², 1-255"]),
    // fav: -i <index> may go before or after the action
    ("fav status", &["izone fav status          # all schedules (or: izone fav s, or just: izone fav)", "izone fav -i 3 status     # one schedule (0-8) (or: izone fav 3)"]),
    ("fav set-name", &["izone fav -i 3 set-name \"Night\"   # max 15 chars"]),
    ("fav set-time", &["izone fav -i 3 set-time 21:00 23:00", "izone fav -i 3 set-time 2100 2300   # 24-hour HHMM also works", "izone fav -i 0 set-time N/A 08:00   # stop only", "izone fav -i 4 set-time 00:20 N/A   # start only"]),
    ("fav set-days", &["izone fav -i 0 set-days Mon Tue Wed Thu Fri", "izone fav -i 5 set-days Sat Sun", "izone fav -i 6 set-days none"]),
    ("fav set-ac", &["izone fav -i 3 set-ac --mode cool --fan auto", "izone fav -i 3 set-ac --mode heat"]),
    ("fav enable", &["izone fav -i 3 enable"]),
    ("fav disable", &["izone fav -i 3 disable"]),
    ("fav delete", &["izone fav -i 6 delete      # asks to confirm; disables, clears name, times + days", "izone fav 6 delete         # index can also go after fav or after delete", "izone fav -i 6 clear -y    # same, no confirmation"]),
    ("fav set-zones", &["izone fav -i 3 set-zones master:3:2350 kitchen:2:2300", "izone fav -i 3 set-zones 3:3:2350 0:2:2300   # zones by controller index", "   # <zone>:<mode>:<setpoint x100>, mode 1=open 2=close 3=auto 4=override 5=constant"]),
    // config
    ("config set-setpoint", &["izone config set-setpoint 23.5   # 15.0-30.0, 0.5 steps"]),
    ("config set-sleep-timer", &["izone config set-sleep-timer 2h", "izone config set-sleep-timer 30m", "izone config set-sleep-timer 0   # off"]),
    ("config set-economy-lock", &["izone config set-economy-lock true --min 18 --max 27", "izone config set-economy-lock false"]),
    ("config set-filter-warning", &["izone config set-filter-warning 3   # 0, 3, 6 or 12 months"]),
    ("config reset-warning", &["izone config reset-warning filter"]),
    ("config set-damper-time", &["izone config set-damper-time 0   # 0 = automatic"]),
    ("config set-auto-deadband", &["izone config set-auto-deadband 1.5   # 0.75-5.0 °C"]),
    ("config set-airflow-lock", &["izone config set-airflow-lock true"]),
    ("config set-airflow-min-lock", &["izone config set-airflow-min-lock false"]),
    ("config set-static-pressure", &["izone config set-static-pressure 2   # 0-4"]),
    ("config set-open-dampers-when-off", &["izone config set-open-dampers-when-off true"]),
    ("config set-scrooge-mode", &["izone config set-scrooge-mode false"]),
    ("config set-reverse-dampers", &["izone config set-reverse-dampers false"]),
    ("config set-constant-control-area", &["izone config set-constant-control-area true --area 54"]),
    // coolbreeze
    ("coolbreeze set-fan-speed", &["izone coolbreeze set-fan-speed 60   # 1-100"]),
    ("coolbreeze set-rh-setpoint", &["izone coolbreeze set-rh-setpoint 50   # 10-90"]),
    ("coolbreeze set-prewash", &["izone coolbreeze set-prewash true --time 5   # 1-60 min"]),
    ("coolbreeze set-drain-after-prewash", &["izone coolbreeze set-drain-after-prewash true"]),
    ("coolbreeze set-drain-cycle", &["izone coolbreeze set-drain-cycle true --period 24   # 1-50 hours"]),
    ("coolbreeze set-postwash", &["izone coolbreeze set-postwash true --time 10   # 5-30 min"]),
    ("coolbreeze set-drain-before-postwash", &["izone coolbreeze set-drain-before-postwash true"]),
    ("coolbreeze set-inverter", &["izone coolbreeze set-inverter true"]),
    ("coolbreeze set-resume-last", &["izone coolbreeze set-resume-last true"]),
    ("coolbreeze set-fan-max-auto", &["izone coolbreeze set-fan-max-auto 80   # 1-100"]),
    ("coolbreeze set-fan-max", &["izone coolbreeze set-fan-max 100   # 1-100"]),
    ("coolbreeze set-exhaust-max", &["izone coolbreeze set-exhaust-max 70   # 1-100"]),
    ("coolbreeze set-exhaust-enable", &["izone coolbreeze set-exhaust-enable true"]),
    ("coolbreeze set-control-sensor", &["izone coolbreeze set-control-sensor screen", "izone coolbreeze set-control-sensor remote"]),
    ("coolbreeze set-temp-calibration", &["izone coolbreeze set-temp-calibration -10   # -50 to 50 = -5.0 to +5.0 °C"]),
    ("coolbreeze set-temp-deadband", &["izone coolbreeze set-temp-deadband 150   # 100-500 = 1.0-5.0 °C"]),
    ("coolbreeze set-auto-fan-max-time", &["izone coolbreeze set-auto-fan-max-time 30   # 0-60 min"]),
    // ventilation
    ("ventilation set-rh-setpoint", &["izone ventilation set-rh-setpoint 50   # 5-95"]),
    ("ventilation set-vocs-setpoint", &["izone ventilation set-vocs-setpoint 500   # 50-2500 ppb"]),
    ("ventilation set-eco2-setpoint", &["izone ventilation set-eco2-setpoint 800   # 500-1500 ppm"]),
    ("ventilation set-fan-stage-delay", &["izone ventilation set-fan-stage-delay 20   # 3-240 min"]),
    ("ventilation set-cycle-fan-off", &["izone ventilation set-cycle-fan-off false"]),
    ("ventilation set-use-rh-control", &["izone ventilation set-use-rh-control true"]),
    ("ventilation set-use-vocs-control", &["izone ventilation set-use-vocs-control true"]),
    ("ventilation set-use-eco2-control", &["izone ventilation set-use-eco2-control true"]),
];

/// Examples for a command path. A group ("zone", "fav", ...) without its own entry
/// gets the first example of each of its subcommands.
pub fn examples_for(path: &str) -> Vec<&'static str> {
    if let Some((_, ex)) = EXAMPLES.iter().find(|(p, _)| *p == path) {
        return ex.to_vec();
    }
    let prefix = format!("{} ", path);
    EXAMPLES
        .iter()
        .filter(|(p, _)| p.starts_with(&prefix))
        .filter_map(|(_, ex)| ex.first().copied())
        .collect()
}

pub fn format_examples(path: &str) -> Option<String> {
    let ex = examples_for(path);
    if ex.is_empty() {
        return None;
    }
    Some(format!("Examples:\n{}", ex.iter().map(|e| format!("  {}", e)).collect::<Vec<_>>().join("\n")))
}

pub fn print_examples(path: &str) {
    if let Some(text) = format_examples(path) {
        eprintln!("\n{}", text);
    }
}

pub fn set_command_path(path: String) {
    let _ = COMMAND_PATH.set(path);
}

/// Mark that a request failed at the network level; examples aren't useful then.
pub fn mark_network_error() {
    NETWORK_ERROR.store(true, Ordering::Relaxed);
}

/// Exit with status 1, first showing usage examples for the current command
/// (unless the failure was a network/controller error).
pub fn exit_with_usage() -> ! {
    if !NETWORK_ERROR.load(Ordering::Relaxed) {
        if let Some(path) = COMMAND_PATH.get() {
            print_examples(path);
        }
    }
    std::process::exit(1)
}

/// Resolve a zone given by name (case-insensitive) or by controller index (e.g. "1")
/// to its configured zone key. Returns the input lowercased if it can't be resolved,
/// so the caller's "Unknown zone" error still reports what the user typed.
pub fn resolve_zone(input: &str) -> String {
    let lower = input.to_lowercase();
    if crate::constants::ZONES.contains_key(lower.as_str()) {
        return lower;
    }
    if !lower.is_empty() && lower.chars().all(|c| c.is_ascii_digit()) {
        let index: u8 = match lower.parse() { Ok(i) => i, Err(_) => return lower };
        if let Some((name, _)) = crate::constants::ZONES.iter().find(|(_, i)| **i == index) {
            return name.to_string();
        }
    }
    lower
}

/// "0=kitchen, 1=theatre, ..." in controller index order, for error messages.
pub fn available_zones() -> String {
    let mut zones: Vec<(&u8, &&str)> = crate::constants::ZONES.iter().map(|(n, i)| (i, n)).collect();
    zones.sort();
    zones.iter().map(|(i, n)| format!("{}={}", i, n)).collect::<Vec<_>>().join(", ")
}
