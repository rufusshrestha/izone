// izone/src/constants.rs

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::Deserialize;

// Default IP address (fallback if no config file exists)
pub const DEFAULT_IZONE_IP: &str = "http://192.168.1.130";
pub const QUERY_URL_SUFFIX: &str = "/iZoneRequestV2";
pub const COMMAND_URL_SUFFIX: &str = "/iZoneCommandV2";

// Config file structure
#[derive(Deserialize, Debug)]
pub struct Config {
    pub izone_ip: Option<String>,
}

// Load configuration from file
pub fn load_config() -> String {
    // Try config locations in order of priority
    let config_paths = vec![
        // 1. ~/.config/izone/config.toml (XDG standard)
        dirs::config_dir().map(|mut p| {
            p.push("izone");
            p.push("config.toml");
            p
        }),
        // 2. ./izone.toml (current directory)
        Some(PathBuf::from("izone.toml")),
        // 3. ~/.izone.toml (home directory)
        dirs::home_dir().map(|mut p| {
            p.push(".izone.toml");
            p
        }),
    ];

    // Use the first config file found. If it exists but is broken, fail loudly rather than
    // silently falling back to the default IP (which could be someone else's device).
    for path in config_paths.into_iter().flatten() {
        if !path.exists() {
            continue;
        }
        let fail = |msg: String| -> ! {
            eprintln!("Error: invalid config file {}: {}", path.display(), msg);
            eprintln!("Expected e.g.: izone_ip = \"http://192.168.1.130\"");
            std::process::exit(1);
        };
        let contents = fs::read_to_string(&path).unwrap_or_else(|e| fail(e.to_string()));
        let config = toml::from_str::<Config>(&contents).unwrap_or_else(|e| fail(e.to_string()));
        let ip = config.izone_ip.unwrap_or_else(|| fail("missing 'izone_ip' setting".to_string()));
        return normalise_ip(&ip).unwrap_or_else(|| fail(format!("'izone_ip' is empty or invalid: {:?}", ip)));
    }

    // No config file: use default
    DEFAULT_IZONE_IP.to_string()
}

/// Accept "192.168.1.130", "http://192.168.1.130" or "http://192.168.1.130/":
/// add http:// if no scheme is given and strip trailing slashes.
fn normalise_ip(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() || trimmed.contains(char::is_whitespace) {
        return None;
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        Some(trimmed.to_string())
    } else if trimmed.contains("://") {
        None // unsupported scheme
    } else {
        Some(format!("http://{}", trimmed))
    }
}

// Lazy static to hold the loaded IP address
lazy_static::lazy_static! {
    pub static ref IZONE_IP: String = load_config();
}

// Define zones and their corresponding API indices.
lazy_static::lazy_static! {
    pub static ref ZONES: HashMap<&'static str, u8> = {
        let mut m = HashMap::new();
        m.insert("kitchen", 0);
        m.insert("theatre", 1);
        m.insert("living", 2);
        m.insert("master", 3);
        m.insert("work", 4);
        m.insert("guest", 5);
        m.insert("rayden", 6);
        m.insert("rumpus", 7);
        m
    };
}

// Global verbose flag (needs to be pub for external access)
pub static mut VERBOSE: bool = false;