# Changelog

All notable changes to the iZone CLI project.

**Author:** Rufus Shrestha
**License:** MIT

## [1.1.3] - 2026-11-04

### Changed
- Web UI: signing in (and the forced password change) always opens the Climate home page.
- Web UI: clicking the iZone logo goes to the Climate home page.

## [1.1.2] - 2026-11-04

### Changed
- Airflow locks are respected: when "Airflow lock" (min and max) or "Minimum airflow lock" is on,
  the web UI greys out the locked sliders with a note, the web server rejects the change, and
  `zone <name> set-min-air` / `set-max-air` refuse with the command to unlock first.

## [1.1.1] - 2026-11-04

### Changed
- Web UI: zones are listed alphabetically (left to right), including in the schedule editor.

## [1.1.0] - 2026-11-03

### Added
- `izone web`: web UI (default http://localhost:8080) for system power/mode/fan/setpoint/sleep timer,
  zones (mode, setpoint, airflow, rename), schedules (times, days, AC mode/fan, per-zone settings, clear)
  and installer settings. Works on phones; light/dark/system theme toggle.
- Sign-in: built-in default password stored only as a salted Argon2id hash (`izone web hash-password`);
  forced password change on first sign-in; 5 failed attempts lock an address for 15 minutes
  (20 across all addresses lock everyone); HttpOnly/SameSite=Strict session cookies.
- Changed password stored as an Argon2id hash encrypted with AES-256-GCM; key in macOS Keychain,
  Windows Credential Manager, or a 0600 key file on Linux. No sudo needed.
- `izone web reset-password`, `izone web install-service` (systemd user service, Linux),
  `izone web uninstall` (removes the service, stored password, key and empty folders).
- Dockerfile and docker-compose.yml (non-root, read-only filesystem, data volume); `justfile`
  (`just up` prompts for anything missing; `just clean` removes everything).
- `IZONE_IP` environment variable overrides the config file.
- `izone status` shows warnings and the filter reminder period.

### Changed
- Warnings that flicker on and off (seen on some controllers while the wall controller shows none)
  are not reported as warnings: the web UI requires 30 s of consistent reports; `izone status`
  re-checks and shows "Intermittent" for mixed readings.
- The "Top" fan speed is offered only for 4-speed or variable-speed fans; `izone fan top` warns otherwise.

## [1.0.6] - 2026-11-03

### Fixed
- `fav delete` failed with `{ERROR}` on an already-disabled schedule: the controller rejects
  `SchedEnable` for a schedule that isn't set up, so the disable step is now skipped when not needed
- `fav disable` on an already-disabled schedule reports that instead of sending a command
- `-i` after `status`/`enable`/`disable`/`delete` works again

### Changed
- `fav delete` also clears the schedule's name (to empty) and only sends the steps needed,
  reporting each one; a schedule that's already clear is reported without prompting
- Schedule index can be given positionally, like zones: `izone fav 6 delete`, `izone fav delete 6`,
  `izone fav 3` (conflicting indexes are rejected)
- Rejected commands show the command sent: `the iZone controller rejected the command {...}`

## [1.0.5] - 2026-11-03

### Added
- Zones can be given by controller index as well as name: `izone zone 1 off`,
  `izone fav -i 3 set-zones 1:3:2350 0:2:2300`
- `zone sum` shows an IDX column; `izone zone <name>` shows the zone's index
- `izone fav -i <n> delete` (alias `clear`): disables a schedule and removes its times and days,
  after a `[y/N]` confirmation (`-y` to skip; refuses without a terminal unless `-y`)
- `-v` can be given anywhere in the command

### Changed
- Unknown-zone errors list zones as `index=name`

## [1.0.4] - 2026-11-03

### Changed
- `zone sum` shows a MIN AIR column instead of DAMPER. The controller's damper position
  (`DmpPos`) isn't a live reading on all systems (it reported a fixed 50% for every zone,
  including closed ones), so the summary now shows the min/max airflow settings side by side.

## [1.0.3] - 2026-11-03

### Added
- Read-only default actions when no action is given:
  - `izone zone` → zone summary; `izone zone <name>` → that zone's status
  - `izone fav` → all schedules; `izone fav -i <n>` → that schedule's status
  - `izone mode` → current mode; `izone fan` → current fan speed
- Help is still available via `-h` / `--help` / `izone help <command>`

## [1.0.2] - 2026-11-03

### Fixed
- `fav set-time` no longer clears the schedule's days, and `fav set-days` no longer clears its times
- `fav set-days` sends times the controller reports as unset (255:255, 19:63) as 31:63 instead of passing them through
- `fav set-zones` sends the full 14-zone list (by position) so unlisted zones keep their settings; all entries are validated before anything is sent
- `zone set-max-air` / `set-min-air` used a non-existent `ZoneAirflow` command (controller replied `{ERROR}`); now `ZoneMaxAir` / `ZoneMinAir`
- `{ERROR}` replies from the controller were reported as success; they are now errors
- All 21 commands taking `true`/`false` were unusable (and sent "off" when no value was given)
- Negative calibration values (`-0.5`, `-10`) were rejected as unknown flags
- Zone status read `SensorFault`, `DamperSkip`, `iSense`, `RfSignal` under the wrong names (always 0, so sensor faults never showed)
- Zone calibration is signed; RF signal and battery now show Full/Half/Empty; low-battery warning in `zone sum` now works
- `fav status` showed per-zone modes using AC mode names (e.g. climate control as "Vent")
- Unset schedule times (e.g. `19:63`) now show as `N/A`; unset schedule mode/fan show as `N/A`
- Panics on network errors and invalid input replaced with clear error messages
- `NaN` no longer bypasses range checks
- `set-days` with an unknown day no longer clears all days
- A broken config file no longer silently falls back to the default IP
- Display alignment of `°C` in status and zone summary boxes
- README `set-zones` docs described AC modes instead of zone modes

### Added
- `fav set-time` accepts `N/A` for no start/stop time, and 24-hour `HHMM` (e.g. `2100 2300`)
- Usage examples shown after every usage/validation error and in `--help`
- `-i` can be given before or after the `fav` action
- `fav s` shortcut for `fav status`; `fan top`; `mode 5` / `fan 4` aliases
- Setpoints validated to 0.5°C steps (zone, schedule, system, economy); balance and economy min/max ordering checked
- `izone_ip` accepts a bare IP (adds `http://`) and ignores a trailing `/`
- 5s connect / 15s request timeouts (was 30s hang when unreachable)

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
