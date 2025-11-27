# Changelog

All notable changes to the iZone CLI project.

**Author:** Rufus Shrestha
**License:** MIT

## [1.0.0] - 2024-11-27

### Added - Complete iZone API Implementation

#### Models
- Added complete `SystemV2` model with all 68+ fields from API specification
- Added nested structs: `UnitOpt`, `Temperzone`, `GasHeat`, `Ventilation`, `Coolbreeze`
- Full support for all iZone system features

#### Zone Configuration Commands (6 new)
- `zone <name> set-balance-max <pct>` - Set zone balance maximum (0-100%, 5% steps)
- `zone <name> set-balance-min <pct>` - Set zone balance minimum (0-100%, 5% steps)
- `zone <name> set-damper-skip <bool>` - Enable/disable damper skip
- `zone <name> set-sensor-calibration <temp>` - Calibrate zone sensor (-5.0 to +5.0°C)
- `zone <name> set-bypass <bool>` - Enable/disable bypass mode
- `zone <name> set-area <m2>` - Set zone area in square meters

#### System Configuration Commands (15 new)
- `config set-setpoint <temp>` - Set system temperature setpoint
- `config set-sleep-timer <minutes>` - Set sleep timer
- `config set-economy-lock <bool> [--min <temp>] [--max <temp>]` - Economy mode with limits
- `config set-filter-warning <months>` - Filter maintenance reminder
- `config reset-warning <type>` - Reset warnings
- `config set-damper-time <seconds>` - Damper control timing
- `config set-auto-deadband <temp>` - Auto mode temperature deadband
- `config set-airflow-lock <bool>` - Lock airflow adjustment
- `config set-airflow-min-lock <bool>` - Lock minimum airflow only
- `config set-static-pressure <level>` - Static pressure (0-4)
- `config set-open-dampers-when-off <bool>` - Damper behavior when off
- `config set-scrooge-mode <bool>` - Economy mode
- `config set-reverse-dampers <bool>` - Reverse damper operation
- `config set-constant-control-area <bool> [--area <m2>]` - Constant control by area

#### Coolbreeze Commands (17 new)
Complete evaporative cooling system control:
- `coolbreeze set-fan-speed <pct>` - Fan speed control (1-100%)
- `coolbreeze set-rh-setpoint <pct>` - Humidity setpoint (10-90%)
- `coolbreeze set-prewash <bool> [--time <min>]` - Prewash cycle configuration
- `coolbreeze set-drain-after-prewash <bool>` - Drain control
- `coolbreeze set-drain-cycle <bool> [--period <hours>]` - Drain cycle timing
- `coolbreeze set-postwash <bool> [--time <min>]` - Postwash cycle
- `coolbreeze set-drain-before-postwash <bool>` - Drain control
- `coolbreeze set-inverter <bool>` - Inverter mode
- `coolbreeze set-resume-last <bool>` - Resume last state
- `coolbreeze set-fan-max-auto <pct>` - Max fan speed in auto mode
- `coolbreeze set-fan-max <pct>` - Max fan speed manual mode
- `coolbreeze set-exhaust-max <pct>` - Max exhaust speed
- `coolbreeze set-exhaust-enable <bool>` - Exhaust mode
- `coolbreeze set-control-sensor <type>` - Control sensor selection
- `coolbreeze set-temp-calibration <value>` - Temperature calibration
- `coolbreeze set-temp-deadband <value>` - Temperature deadband
- `coolbreeze set-auto-fan-max-time <minutes>` - Auto fan max time

#### Ventilation Commands (8 new)
Air quality monitoring and control:
- `ventilation set-rh-setpoint <pct>` - Humidity setpoint (5-95%)
- `ventilation set-vocs-setpoint <ppb>` - VOCs setpoint (50-2500 ppb)
- `ventilation set-eco2-setpoint <ppm>` - eCO2 setpoint (500-1500 ppm)
- `ventilation set-fan-stage-delay <minutes>` - Fan stage delay (3-240 min)
- `ventilation set-cycle-fan-off <bool>` - Cycle fan off control
- `ventilation set-use-rh-control <bool>` - Enable RH control
- `ventilation set-use-vocs-control <bool>` - Enable VOCs control
- `ventilation set-use-eco2-control <bool>` - Enable eCO2 control

#### Configuration System
- Added TOML-based configuration file support
- Config file priority: XDG config → current dir → home dir → default
- Multiple config location support for flexibility
- Automatic config file discovery
- Default fallback to `http://192.168.1.130`

#### Dependencies
- Added `toml = "0.8"` for configuration file parsing
- Added `dirs = "5.0"` for standard directory paths

#### Documentation
- Comprehensive README.md with usage examples
- Detailed CONFIG.md for configuration setup
- ERROR_MESSAGES.md explaining error handling
- Example config file (izone.toml.example)
- Complete command reference and troubleshooting guide
- MIT License added (LICENSE file)
- Author information updated (Rufus Shrestha)

### Changed

#### Error Handling
- Improved error messages for connection failures
- User-friendly error messages showing IP address being used
- Better guidance for troubleshooting connectivity issues
- Replaced technical JSON parsing errors with actionable messages
- Consistent error message format across all commands

**Before:**
```
Error querying system status: Failed to parse JSON: error decoding response body
```

**After:**
```
Unexpected response from iZone controller at http://192.168.1.130.
Ensure your configuration has the correct iZone IP and the controller is reachable in your network.
Error details: error decoding response body: expected value at line 1 column 1
```

#### API Layer
- Enhanced `make_query_request()` with better error messages
- Enhanced `make_command_request()` with better error messages
- Connection errors now show IP address
- JSON parsing errors provide helpful context

#### Constants
- Changed `IZONE_IP` from const to lazy_static String
- Loads IP from config file or uses default
- Maintains backward compatibility

### Fixed
- JSON parsing errors now handled gracefully instead of panicking
- Connection errors provide clear next steps
- Missing `.expect()` calls replaced with proper error handling

## Summary Statistics

- **Total New Commands**: 46
  - 6 zone configuration commands
  - 15 system configuration commands
  - 17 Coolbreeze commands
  - 8 ventilation commands

- **API Coverage**: 100% of iZone_JSON_datastrings.h specification

- **Files Modified**:
  - `src/constants.rs` - Config system
  - `src/api.rs` - Error handling
  - `src/commands/system.rs` - New commands + error handling
  - `src/commands/zones.rs` - New zone config commands
  - `src/main.rs` - CLI argument parsing
  - `Cargo.toml` - New dependencies

- **Files Added**:
  - `README.md` - Complete usage guide
  - `CONFIG.md` - Configuration guide
  - `ERROR_MESSAGES.md` - Error handling documentation
  - `CHANGELOG.md` - This file
  - `izone.toml.example` - Example config file

## Migration Guide

### For Existing Users

No breaking changes. The tool remains backward compatible:

1. **Without Config File**: Works exactly as before with default IP
2. **With Config File**: Optional - add config for custom IP

### Setting Up Config File

```bash
mkdir -p ~/.config/izone
cat > ~/.config/izone/config.toml << 'EOF'
izone_ip = "http://192.168.1.130"
EOF
```

### Error Messages

Error messages are now more helpful. If you see:
```
Unexpected response from iZone controller at http://X.X.X.X
```

Check:
1. IP address in config file matches your controller
2. Controller is powered on and connected to network
3. Network connectivity to controller IP

Use verbose mode for debugging: `izone -v status`

## Development

- Build: `cargo build --release`
- Location: `target/release/izone`
- Version: `1.0.0`
- Binary size: ~5 MB

## Credits

Built for Airstream iZone air conditioning controllers.
Complete implementation of iZone API v2 specification.

**Authors:** Rufus Shrestha
**License:** MIT License - see LICENSE file for details
