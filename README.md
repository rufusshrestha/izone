# iZone CLI Controller

A comprehensive command-line interface for controlling Airstream iZone air conditioning systems. This tool provides complete access to all iZone API features including zone control, system configuration, schedules/favourites, Coolbreeze evaporative cooling, and ventilation systems.

**Author:** Rufus Shrestha
**License:** MIT (See LICENSE.md for Terms and Conditions of usage)
**Version:** 1.0.0

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
  - [Basic System Control](#basic-system-control)
  - [Zone Control](#zone-control)
  - [System Modes](#system-modes)
  - [Fan Speed Control](#fan-speed-control)
  - [Schedules / Favourites](#schedules--favourites)
  - [System Configuration](#system-configuration)
  - [Coolbreeze Evaporative Cooling](#coolbreeze-evaporative-cooling)
  - [Ventilation Control](#ventilation-control)
- [Examples](#examples)
- [Zone Configuration](#zone-configuration)
- [Troubleshooting](#troubleshooting)

## Features

- **Complete Zone Control**: Individual zone temperature, mode, and airflow management
- **System Control**: Power, mode (Auto/Cool/Heat/Vent/Dry), fan speed
- **Schedules/Favourites**: Full management of 9 programmable schedules
- **Advanced Configuration**: Economy lock, filter warnings, damper control, static pressure
- **Coolbreeze Integration**: Complete control of evaporative cooling systems (17 commands)
- **Ventilation Control**: Air quality management (RH, VOCs, eCO2)
- **Zone Configuration**: Balance control, sensor calibration, bypass modes, area settings
- **Formatted Output**: Clean, color-coded box-style output for all commands

## Installation

### From Source

```bash
git clone https://github.com/rufusshrestha/izone.git
cd izone
cargo build --release
sudo cp target/release/izone /usr/local/bin/
```

### Prerequisites

- Rust 1.70+ (for building from source)
- Access to iZone controller on local network

## Configuration

### Config File Setup

The iZone CLI supports configuration files for customizing the controller IP address. The program searches for config files in the following order:

1. `~/.config/izone/config.toml` (recommended - XDG standard)
2. `./izone.toml` (current directory)
3. `~/.izone.toml` (home directory)

If no config file is found, it defaults to `http://192.168.1.130`.

#### Creating a Config File

**Option 1: XDG Standard Location (Recommended)**
```bash
mkdir -p ~/.config/izone
cat > ~/.config/izone/config.toml << 'EOF'
# iZone controller IP address
izone_ip = "http://192.168.1.130"
EOF
```

**Option 2: Current Directory**
```bash
cp izone.toml.example izone.toml
# Edit izone.toml with your IP address
```

**Option 3: Home Directory**
```bash
cat > ~/.izone.toml << 'EOF'
izone_ip = "http://192.168.1.100"
EOF
```

#### Config File Format

```toml
# iZone Configuration File
# Set your iZone controller IP address (including http://)
izone_ip = "http://192.168.1.130"
```

### Zone Name Configuration

Edit `src/constants.rs` to customize zone names for your installation:

```rust
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
```

After editing, rebuild: `cargo build --release`

## Usage

### Basic System Control

#### Power Control
```bash
# Turn on the AC system
izone on

# Turn off the AC system
izone off
```

#### Status Information
```bash
# Get overall system status (detailed)
izone status
izone s  # Short alias

# Get controller temperature only
izone controller-temp
izone ct  # Short alias

# Get summary of all zones
izone zone summary
izone z sum  # Short aliases
```

#### Verbose Mode
```bash
# Show full JSON API responses
izone -v status
izone --verbose zone kitchen status
```

### Zone Control

#### Individual Zone Status
```bash
# Get detailed zone status
izone zone kitchen status
izone z kitchen s  # Short aliases

# Get only zone temperature
izone zone living temp
izone z living t  # Short aliases
```

#### Zone Modes
```bash
# Turn zone on (Auto mode)
izone zone kitchen on
izone zone kitchen auto  # Same as 'on'

# Turn zone off (Close damper)
izone zone master off

# Open zone (Manual open - max airflow)
izone zone theatre open

# Set zone to Override mode
izone zone work override

# Set zone to Constant mode
izone zone guest constant
```

#### Zone Temperature Control
```bash
# Set zone setpoint (accepts decimal values)
izone zone kitchen set-setpoint 22.5
izone z kitchen sp 23.0  # Short aliases

# Examples with different temperatures
izone zone master set-setpoint 20
izone zone living set-setpoint 24.5
```

#### Zone Airflow Control
```bash
# Set maximum airflow (0-100%)
izone zone kitchen set-max-air 90

# Set minimum airflow (0-100%)
izone zone kitchen set-min-air 10

# Example: Set airflow range for a zone
izone zone master set-min-air 15
izone zone master set-max-air 85
```

#### Zone Configuration
```bash
# Set zone name (max 15 characters)
izone zone kitchen set-name "Kitchen"

# Set zone balance max (0-100%, steps of 5%)
izone zone kitchen set-balance-max 95

# Set zone balance min (0-100%, steps of 5%)
izone zone kitchen set-balance-min 5

# Enable/disable damper skip
izone zone theatre set-damper-skip true
izone zone living set-damper-skip false

# Calibrate zone sensor (-5.0 to +5.0°C)
izone zone master set-sensor-calibration 1.5
izone zone guest set-sensor-calibration -0.5

# Enable/disable bypass mode
izone zone work set-bypass true
izone zone rumpus set-bypass false

# Set zone area (square meters)
izone zone kitchen set-area 25
izone zone living set-area 40
```

### System Modes

```bash
# Set system to Auto mode
izone mode auto
izone m auto  # Short alias
izone m a     # Even shorter
izone m 0     # Numeric

# Set to Cool mode
izone mode cool
izone m cool
izone m c
izone m 1

# Set to Heat mode
izone mode heat
izone m heat
izone m h
izone m 2

# Set to Vent mode
izone mode vent
izone m vent
izone m v
izone m 3

# Set to Dry mode
izone mode dry
izone m dry
izone m d
izone m 4
```

### Fan Speed Control

```bash
# Set fan to Auto
izone fan auto
izone f auto  # Short alias
izone f a
izone f 0

# Set fan to Low
izone fan low
izone f low
izone f l
izone f 1

# Set fan to Medium
izone fan medium
izone f med
izone f m
izone f 2

# Set fan to High
izone fan high
izone f high
izone f h
izone f 3
```

### Schedules / Favourites

iZone supports 9 programmable schedules (0-8).

#### View Schedule Status
```bash
# Get summary of all schedules
izone fav status
izone schedule status  # Alternative command name

# Get status of specific schedule
izone fav -i 0 status
izone schedule -i 2 status
```

#### Configure Schedules
```bash
# Set schedule name
izone fav -i 0 set-name "Morning Warmup"
izone fav -i 1 set-name "Work Day"

# Set schedule times (HH:MM format)
izone fav -i 0 set-time 06:00 08:30
izone fav -i 1 set-time 08:00 17:00

# Set days enabled (Mon, Tue, Wed, Thu, Fri, Sat, Sun)
izone fav -i 0 set-days Mon Tue Wed Thu Fri
izone fav -i 1 set-days Mon Wed Fri
izone fav -i 2 set-days Sat Sun

# Set AC mode and fan speed for schedule
izone fav -i 0 set-ac --mode heat --fan medium
izone fav -i 1 set-ac --mode cool --fan auto
izone fav -i 2 set-ac --mode vent

# Enable/disable schedule
izone fav -i 0 enable
izone fav -i 1 disable

# Set zone-specific settings for schedule
# Format: zone_name:mode:setpoint
# Example: kitchen:3:2250 (mode 3=vent, setpoint 22.50°C)
izone fav -i 0 set-zones kitchen:1:2200 master:1:2000
izone fav -i 1 set-zones living:2:2400 work:2:2300
```

#### Complete Schedule Example
```bash
# Configure a complete "Morning Warmup" schedule
izone fav -i 0 set-name "Morning Warmup"
izone fav -i 0 set-time 06:00 08:30
izone fav -i 0 set-days Mon Tue Wed Thu Fri
izone fav -i 0 set-ac --mode heat --fan low
izone fav -i 0 set-zones kitchen:2:2100 master:2:2000 living:2:2200
izone fav -i 0 enable
```

### System Configuration

All system configuration commands use the `config` command (alias: `cfg`).

#### Temperature Settings
```bash
# Set system setpoint (15.0-30.0°C)
izone config set-setpoint 22.5
izone cfg set-setpoint 24.0

# Set sleep timer (minutes, 0 to disable)
izone config set-sleep-timer 60
izone config set-sleep-timer 0  # Disable

# Configure economy lock with temperature limits
izone config set-economy-lock true --min 18.0 --max 26.0
izone config set-economy-lock false
```

#### Maintenance
```bash
# Set filter warning period (0=disabled, 3, 6, or 12 months)
izone config set-filter-warning 6
izone config set-filter-warning 0  # Disable

# Reset filter warning
izone config reset-warning filter
```

#### System Timing
```bash
# Set damper control time (seconds, 0=automatic)
izone config set-damper-time 45
izone config set-damper-time 0  # Automatic

# Set auto mode deadband (0.75-5.0°C)
izone config set-auto-deadband 2.0
```

#### Airflow Control
```bash
# Lock/unlock airflow adjustment (both min and max)
izone config set-airflow-lock true   # Lock
izone config set-airflow-lock false  # Unlock

# Lock/unlock minimum airflow adjustment only
izone config set-airflow-min-lock true
izone config set-airflow-min-lock false

# Set static pressure level (0-4: lowest to highest)
izone config set-static-pressure 0  # Lowest
izone config set-static-pressure 2  # Medium
izone config set-static-pressure 4  # Highest
```

#### Advanced Settings
```bash
# Open/close dampers when AC is off
izone config set-open-dampers-when-off true
izone config set-open-dampers-when-off false

# Enable/disable scrooge mode (economy mode)
izone config set-scrooge-mode true
izone config set-scrooge-mode false

# Enable/disable reverse dampers
izone config set-reverse-dampers true
izone config set-reverse-dampers false

# Configure constant control by area
izone config set-constant-control-area true --area 150
izone config set-constant-control-area false
```

### Coolbreeze Evaporative Cooling

Complete control of Coolbreeze evaporative cooling systems. Use `coolbreeze` command (alias: `cb`).

#### Fan Control
```bash
# Set fan speed (1-100%)
izone coolbreeze set-fan-speed 75
izone cb set-fan-speed 50

# Set maximum fan speed in auto mode
izone coolbreeze set-fan-max-auto 85

# Set maximum fan speed in manual mode
izone coolbreeze set-fan-max 95

# Set auto fan max time (0-60 minutes)
izone coolbreeze set-auto-fan-max-time 30
```

#### Humidity Control
```bash
# Set humidity setpoint (10-90%)
izone coolbreeze set-rh-setpoint 65
izone cb set-rh-setpoint 70
```

#### Wash Cycles
```bash
# Configure prewash (enable + optional time in minutes)
izone coolbreeze set-prewash true --time 15
izone coolbreeze set-prewash false

# Enable/disable drain after prewash
izone coolbreeze set-drain-after-prewash true
izone coolbreeze set-drain-after-prewash false

# Configure drain cycle (enable + optional period in hours)
izone coolbreeze set-drain-cycle true --period 24
izone coolbreeze set-drain-cycle false

# Configure postwash (enable + optional time in minutes)
izone coolbreeze set-postwash true --time 20
izone coolbreeze set-postwash false

# Enable/disable drain before postwash
izone coolbreeze set-drain-before-postwash true
```

#### Exhaust Control
```bash
# Set maximum exhaust fan speed (1-100%)
izone coolbreeze set-exhaust-max 80

# Enable/disable exhaust mode
izone coolbreeze set-exhaust-enable true
izone coolbreeze set-exhaust-enable false
```

#### Advanced Settings
```bash
# Enable/disable inverter mode
izone coolbreeze set-inverter true

# Enable/disable resume last state on startup
izone coolbreeze set-resume-last true

# Set control sensor (screen or remote)
izone coolbreeze set-control-sensor screen
izone coolbreeze set-control-sensor remote

# Set temperature calibration (-50 to 50, divide by 10 for °C)
izone coolbreeze set-temp-calibration 15   # +1.5°C
izone coolbreeze set-temp-calibration -10  # -1.0°C

# Set temperature deadband (100-500, divide by 100 for °C)
izone coolbreeze set-temp-deadband 200  # 2.0°C
```

### Ventilation Control

Control ventilation systems for air quality management. Use `ventilation` command (alias: `vent`).

#### Air Quality Setpoints
```bash
# Set humidity setpoint (5-95%)
izone ventilation set-rh-setpoint 60
izone vent set-rh-setpoint 65

# Set VOCs setpoint (50-2500 ppb)
izone ventilation set-vocs-setpoint 500
izone vent set-vocs-setpoint 800

# Set eCO2 setpoint (500-1500 ppm)
izone ventilation set-eco2-setpoint 1000
izone vent set-eco2-setpoint 900
```

#### Fan Control
```bash
# Set fan stage delay (3-240 minutes)
izone ventilation set-fan-stage-delay 10
izone vent set-fan-stage-delay 15

# Enable/disable cycle fan off
izone ventilation set-cycle-fan-off true
izone ventilation set-cycle-fan-off false
```

#### Control Toggles
```bash
# Enable/disable RH (humidity) control
izone ventilation set-use-rh-control true
izone vent set-use-rh-control false

# Enable/disable VOCs control
izone ventilation set-use-vocs-control true
izone vent set-use-vocs-control false

# Enable/disable eCO2 control
izone ventilation set-use-eco2-control true
izone vent set-use-eco2-control false
```

## Examples

### Morning Routine Automation
```bash
#!/bin/bash
# Morning warmup script

# Turn on AC
izone on

# Set to heat mode
izone mode heat

# Configure zones
izone zone kitchen on
izone zone kitchen set-setpoint 22
izone zone master on
izone zone master set-setpoint 20
izone zone living on
izone zone living set-setpoint 21

# Set fan to low for quiet operation
izone fan low

echo "Morning warmup activated!"
```

### Evening Cool Down
```bash
#!/bin/bash
# Evening cool down script

# Set to cool mode
izone mode cool

# Configure living areas
izone zone kitchen set-setpoint 23
izone zone living set-setpoint 22.5

# Turn off bedrooms for now
izone zone master off
izone zone guest off

# Set fan to medium
izone fan medium

echo "Evening cool down activated!"
```

### Energy Saver Mode
```bash
#!/bin/bash
# Enable economy settings for energy saving

# Enable economy lock with wider temperature range
izone config set-economy-lock true --min 19.0 --max 25.0

# Enable scrooge mode
izone config set-scrooge-mode true

# Set lower fan speed
izone fan low

# Reduce zone airflows
izone zone kitchen set-max-air 70
izone zone living set-max-air 70
izone zone master set-max-air 70

echo "Energy saver mode enabled!"
```

### Coolbreeze Summer Setup
```bash
#!/bin/bash
# Configure Coolbreeze for summer operation

# Set humidity control
izone coolbreeze set-rh-setpoint 65

# Configure wash cycles
izone coolbreeze set-prewash true --time 15
izone coolbreeze set-drain-cycle true --period 24
izone coolbreeze set-postwash true --time 20

# Set fan speeds
izone coolbreeze set-fan-max-auto 85
izone coolbreeze set-fan-max 95

# Enable exhaust
izone coolbreeze set-exhaust-enable true
izone coolbreeze set-exhaust-max 80

echo "Coolbreeze summer setup complete!"
```

### Air Quality Monitoring
```bash
#!/bin/bash
# Configure ventilation for optimal air quality

# Set air quality thresholds
izone ventilation set-rh-setpoint 60
izone ventilation set-vocs-setpoint 500
izone ventilation set-eco2-setpoint 900

# Enable all controls
izone ventilation set-use-rh-control true
izone ventilation set-use-vocs-control true
izone ventilation set-use-eco2-control true

# Set fan timing
izone ventilation set-fan-stage-delay 10

echo "Air quality monitoring configured!"
```

## Zone Configuration

### Customizing Zone Names

1. Edit `src/constants.rs`:
```rust
pub static ref ZONES: HashMap<&'static str, u8> = {
    let mut m = HashMap::new();
    m.insert("your_zone_1", 0);
    m.insert("your_zone_2", 1);
    // ... add up to 8 zones
    m
};
```

2. Rebuild the project:
```bash
cargo build --release
```

### Zone Index Mapping

Zone indices in the iZone API are 0-based:
- Zone 0 = First zone
- Zone 1 = Second zone
- ...
- Zone 7 = Eighth zone

## Troubleshooting

### Connection Issues

**Problem**: Cannot connect to iZone controller
```bash
# Check controller IP is correct
curl http://192.168.1.130/iZoneRequestV2

# Verify controller is on same network
ping 192.168.1.130

# Try using verbose mode to see API responses
izone -v status
```

**Solution**:
- Ensure controller IP is correct in config file
- Verify network connectivity
- Check firewall settings

### Config File Not Loading

**Problem**: Config file changes not taking effect

**Solution**:
1. Check config file location:
```bash
# Check if config exists
ls -la ~/.config/izone/config.toml
ls -la ~/.izone.toml
ls -la ./izone.toml
```

2. Verify config file syntax:
```bash
# Config must be valid TOML format
cat ~/.config/izone/config.toml
```

3. Check file permissions:
```bash
chmod 644 ~/.config/izone/config.toml
```

### Zone Names Not Working

**Problem**: "Unknown zone" error

**Solution**:
- Zone names must match exactly what's defined in `src/constants.rs`
- Zone names are case-sensitive (use lowercase)
- Rebuild after editing constants.rs

### Temperature Values

**Problem**: Confusion about temperature format

**Note**:
- User-facing commands use decimal degrees (e.g., 22.5°C)
- API uses integers (2250 = 22.50°C)
- CLI handles conversion automatically
- Setpoint range: 15.0-30.0°C

### Schedule Settings

**Problem**: Zone settings in schedules not working

**Format**: `zone_name:mode:setpoint`
- mode: 0=auto, 1=cool, 2=heat, 3=vent, 4=dry
- setpoint: temperature × 100 (e.g., 2250 for 22.5°C)

**Example**:
```bash
izone fav -i 0 set-zones kitchen:2:2200 master:2:2100
# Sets kitchen to heat mode 22.0°C, master to heat mode 21.0°C
```

## Command Reference

### Quick Command List

#### System
- `izone on` - Turn on
- `izone off` - Turn off
- `izone status` - Get status
- `izone controller-temp` - Get temperature

#### Zones
- `izone zone <name> status` - Zone status
- `izone zone <name> on/off` - Zone control
- `izone zone <name> set-setpoint <temp>` - Set temperature
- `izone zone summary` - All zones

#### Modes
- `izone mode auto/cool/heat/vent/dry` - Set mode

#### Fan
- `izone fan auto/low/medium/high` - Set fan

#### Schedules
- `izone fav status` - All schedules
- `izone fav -i <N> status` - Specific schedule
- `izone fav -i <N> enable/disable` - Enable/disable

#### Configuration
- `izone config <subcommand>` - 15 config commands

#### Coolbreeze
- `izone coolbreeze <subcommand>` - 17 coolbreeze commands

#### Ventilation
- `izone ventilation <subcommand>` - 8 ventilation commands

### Help Commands

```bash
# Main help
izone --help

# Command-specific help
izone zone --help
izone config --help
izone coolbreeze --help
izone ventilation --help
izone fav --help
```

## License

MIT License

Copyright (c) 2024 Rufus Shrestha

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Credits

**Authors:** Rufus Shrestha

Built for Airstream iZone air conditioning controllers.

**Acknowledgments:**
- Airstream iZone for the excellent HVAC system
- The Rust community for amazing tools and libraries
