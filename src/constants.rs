// izone/src/constants.rs

use std::collections::HashMap;

// Constants for iZone API
pub const IZONE_IP: &str = "http://192.168.1.130";
pub const QUERY_URL: &str = "http://192.168.1.130/iZoneRequestV2";
pub const COMMAND_URL: &str = "http://192.168.1.130/iZoneCommandV2";

// Define zones and their corresponding API indices.
lazy_static::lazy_static! { // Use the fully qualified path here
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
// Note: Using static mut is generally discouraged due to thread safety
// but acceptable for a single-threaded CLI script.
pub static mut VERBOSE: bool = false;