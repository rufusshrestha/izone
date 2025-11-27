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

    // Try each config path
    for path_option in config_paths {
        if let Some(path) = path_option {
            if path.exists() {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(config) = toml::from_str::<Config>(&contents) {
                        if let Some(ip) = config.izone_ip {
                            return ip;
                        }
                    }
                }
            }
        }
    }

    // Fallback to default
    DEFAULT_IZONE_IP.to_string()
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