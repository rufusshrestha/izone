# Configuration Guide

**Author:** Rufus Shrestha
**License:** MIT

## Overview

The iZone CLI supports flexible configuration through TOML config files. This allows you to customize the iZone controller IP address without modifying the source code.

## Config File Locations

The program searches for configuration files in the following order (first found wins):

1. **`~/.config/izone/config.toml`** (Recommended - XDG standard)
   - Standard location for user configuration on Unix-like systems
   - Keeps config organized with other application configs

2. **`./izone.toml`** (Current directory)
   - Useful for project-specific or temporary configurations
   - Great for testing different controller IPs

3. **`~/.izone.toml`** (Home directory)
   - Traditional Unix dot-file location
   - Alternative if you prefer configs in home directory

4. **Default fallback**: `http://192.168.1.130`
   - Used when no config file is found
   - Hardcoded in `src/constants.rs`

## Setting Up Config File

### Quick Setup (Recommended Method)

Create the config directory and file:

```bash
# Create config directory
mkdir -p ~/.config/izone

# Create config file with your IP
cat > ~/.config/izone/config.toml << 'EOF'
# iZone Controller Configuration
izone_ip = "http://192.168.1.130"
EOF
```

### Alternative Locations

**Current Directory (for testing):**
```bash
# Copy example file
cp izone.toml.example izone.toml

# Edit with your IP
nano izone.toml
```

**Home Directory:**
```bash
echo 'izone_ip = "http://192.168.1.100"' > ~/.izone.toml
```

## Config File Format

The config file uses TOML format (simple key-value syntax):

```toml
# iZone controller IP address
# Must include http:// or https://
izone_ip = "http://192.168.1.130"
```

### Valid Examples

```toml
# Standard IP address
izone_ip = "http://192.168.1.130"

# Different subnet
izone_ip = "http://10.0.0.50"

# HTTPS (if supported by your controller)
izone_ip = "https://192.168.1.130"

# Hostname (if DNS configured)
izone_ip = "http://izone.local"
```

### Invalid Examples

```toml
# ❌ Missing http://
izone_ip = "192.168.1.130"

# ❌ Incorrect key name
controller_ip = "http://192.168.1.130"

# ❌ Wrong format
izone_ip: "http://192.168.1.130"
```

## Finding Your Controller IP

### Method 1: Router Admin Panel
1. Log into your router's admin interface
2. Look for connected devices or DHCP client list
3. Find device named "iZone" or similar
4. Note the IP address

### Method 2: Network Scan
```bash
# Using nmap (if installed)
nmap -sn 192.168.1.0/24 | grep -B 2 "iZone"

# Or try common ports
nmap -p 80 192.168.1.0/24 --open
```

### Method 3: iZone App
1. Open the official iZone mobile app
2. Go to settings or about section
3. Controller IP should be displayed

### Method 4: Test Common Addresses
```bash
# Test default
curl -s http://192.168.1.130/iZoneRequestV2

# Test common alternatives
for ip in 130 131 132 140 150; do
    echo -n "Testing 192.168.1.$ip ... "
    curl -s --connect-timeout 2 http://192.168.1.$ip/iZoneRequestV2 && echo "Found!" || echo "Not found"
done
```

## Verifying Configuration

### Check Which Config is Loaded

Since the program loads the first config file it finds, you can test by:

1. Create a test config with a unique IP:
```bash
echo 'izone_ip = "http://192.168.1.200"' > ~/.config/izone/config.toml
```

2. Run with verbose mode:
```bash
izone -v status 2>&1 | grep -i "192.168.1.200"
```

3. Check the error message - it will show which IP it's trying to connect to

### Priority Testing

Create different config files to test priority:

```bash
# Create configs with different IPs
echo 'izone_ip = "http://192.168.1.100"' > ~/.config/izone/config.toml
echo 'izone_ip = "http://192.168.1.101"' > ./izone.toml
echo 'izone_ip = "http://192.168.1.102"' > ~/.izone.toml

# Run command - should use .100 (highest priority)
izone status

# Remove highest priority
rm ~/.config/izone/config.toml

# Run again - should use .101 (next priority)
izone status
```

## Troubleshooting

### Config File Not Loading

**Problem**: Changes to config file don't take effect

**Solutions**:

1. **Check file location**:
   ```bash
   ls -la ~/.config/izone/config.toml
   ```

2. **Verify file syntax**:
   ```bash
   cat ~/.config/izone/config.toml
   # Should show: izone_ip = "http://..."
   ```

3. **Check file permissions**:
   ```bash
   chmod 644 ~/.config/izone/config.toml
   ```

4. **Validate TOML syntax**:
   ```bash
   # Install toml-cli if available
   toml validate ~/.config/izone/config.toml
   ```

### Wrong IP Being Used

**Problem**: Program uses unexpected IP address

**Cause**: Higher priority config file exists

**Solution**:
```bash
# Check all possible config locations
echo "Checking config files..."
[ -f ~/.config/izone/config.toml ] && echo "Found: ~/.config/izone/config.toml" && cat ~/.config/izone/config.toml
[ -f ./izone.toml ] && echo "Found: ./izone.toml" && cat ./izone.toml
[ -f ~/.izone.toml ] && echo "Found: ~/.izone.toml" && cat ~/.izone.toml
```

Remove or rename conflicting config files:
```bash
# Rename unwanted configs
mv ~/.config/izone/config.toml ~/.config/izone/config.toml.bak
```

### Connection Refused

**Problem**: "Connection refused" error even with correct IP

**Possible Causes**:
1. iZone controller is offline
2. Wrong IP address
3. Network connectivity issues
4. Firewall blocking connection

**Solutions**:
```bash
# Test connectivity
ping 192.168.1.130

# Test HTTP connection
curl -v http://192.168.1.130/iZoneRequestV2

# Check if controller is responding on port 80
telnet 192.168.1.130 80
```

## Advanced Configuration

### Multiple Controllers

If you have multiple iZone controllers, use directory-specific configs:

```bash
# Controller 1 (upstairs)
mkdir ~/izone-upstairs
cd ~/izone-upstairs
echo 'izone_ip = "http://192.168.1.130"' > izone.toml

# Controller 2 (downstairs)
mkdir ~/izone-downstairs
cd ~/izone-downstairs
echo 'izone_ip = "http://192.168.1.131"' > izone.toml

# Use each
cd ~/izone-upstairs && izone status
cd ~/izone-downstairs && izone status
```

### Wrapper Scripts

Create wrapper scripts for different controllers:

```bash
# ~/bin/izone-upstairs
#!/bin/bash
cd ~/izone-upstairs
izone "$@"

# ~/bin/izone-downstairs
#!/bin/bash
cd ~/izone-downstairs
izone "$@"

# Make executable
chmod +x ~/bin/izone-upstairs ~/bin/izone-downstairs

# Use
izone-upstairs status
izone-downstairs zone kitchen temp
```

### Dynamic IP Resolution

If your controller IP changes frequently, create a script:

```bash
#!/bin/bash
# update-izone-ip.sh

# Find iZone controller on network
IZONE_IP=$(nmap -sn 192.168.1.0/24 | grep -B 2 "iZone" | grep "Nmap scan" | awk '{print $5}')

if [ -n "$IZONE_IP" ]; then
    echo "izone_ip = \"http://$IZONE_IP\"" > ~/.config/izone/config.toml
    echo "Updated config to: $IZONE_IP"
else
    echo "iZone controller not found on network"
    exit 1
fi
```

## Environment Variables (Future Enhancement)

*Note: Currently not implemented, but could be added*

Future versions might support:
```bash
# Override config with environment variable
export IZONE_IP="http://192.168.1.140"
izone status

# Command-line override
izone --ip http://192.168.1.140 status
```

## Best Practices

1. **Use XDG config location** (`~/.config/izone/config.toml`)
   - Standard, organized, widely adopted

2. **Keep config simple**
   - Just the IP address is needed
   - Don't add unnecessary settings

3. **Document your IP**
   - Add comment in config file noting where IP came from
   ```toml
   # Controller IP from router DHCP reservation
   # MAC: 00:11:22:33:44:55
   izone_ip = "http://192.168.1.130"
   ```

4. **Use DHCP reservations**
   - Configure router to always assign same IP to controller
   - Prevents IP changes and config updates

5. **Back up your config**
   ```bash
   cp ~/.config/izone/config.toml ~/.config/izone/config.toml.backup
   ```

## Example Configurations

### Home Network
```toml
# ~/.config/izone/config.toml
# Home iZone controller - living room system
izone_ip = "http://192.168.1.130"
```

### Office Network
```toml
# ~/.config/izone/config.toml
# Office iZone controller - VLAN 10
izone_ip = "http://10.10.10.50"
```

### Development/Testing
```toml
# ./izone.toml
# Test controller on bench
izone_ip = "http://192.168.100.10"
```

## Summary

- **Default**: `http://192.168.1.130` (if no config file)
- **Recommended**: `~/.config/izone/config.toml`
- **Format**: `izone_ip = "http://YOUR_IP_HERE"`
- **Priority**: XDG config → current dir → home dir → default
- **Validation**: Test with `izone status` or `izone -v status`

For more help, see the main README.md or run `izone --help`.
