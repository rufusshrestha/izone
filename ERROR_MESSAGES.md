# Error Message Examples

**Author:** Rufus Shrestha
**License:** MIT

This document shows the improved error messages when connecting to iZone controllers.

## Connection Errors

### Scenario 1: Controller Not Reachable

When the iZone controller is offline or the IP address is incorrect:

**Old Error Message:**
```
Error querying system status: Failed to send query request: error sending request for url (http://192.168.1.130/iZoneRequestV2): error trying to connect: tcp connect error: Connection refused (os error 61)
```

**New Error Message:**
```
Failed to connect to iZone controller at http://192.168.1.130.
Ensure the IP address is correct and the controller is reachable in your network.
Error details: error sending request for url (http://192.168.1.130/iZoneRequestV2): error trying to connect: tcp connect error: Connection refused (os error 61)
```

### Scenario 2: Wrong IP Address (Device exists but not iZone)

When connecting to a device that responds but doesn't return valid iZone JSON:

**Old Error Message:**
```
Error querying system status: Failed to parse JSON: error decoding response body: expected value at line 1 column 1
```

**New Error Message:**
```
Unexpected response from iZone controller at http://192.168.1.130.
Ensure your configuration has the correct iZone IP and the controller is reachable in your network.
Error details: error decoding response body: expected value at line 1 column 1
```

### Scenario 3: Invalid JSON Response

When the controller returns malformed JSON:

**Old Error Message:**
```
thread 'main' panicked at 'Failed to parse system status response: missing field `SystemV2` at line 1 column 123'
```

**New Error Message:**
```
Unexpected response from iZone controller at http://192.168.1.130.
Ensure your configuration has the correct iZone IP and the controller is reachable in your network.
Error details: Failed to parse system status response - missing field `SystemV2` at line 1 column 123
```

## Error Message Features

### 1. User-Friendly Language
- Avoids technical jargon where possible
- Clearly states what went wrong
- Provides actionable advice

### 2. Shows IP Address
- Displays the configured IP address being used
- Helps users quickly identify configuration issues
- Makes it easy to verify the correct controller is targeted

### 3. Configuration Guidance
- Reminds users where configuration is set
- Suggests checking network connectivity
- Points to potential causes

### 4. Technical Details Available
- Still includes detailed error information
- Useful for debugging specific issues
- Helps with support requests

## Troubleshooting Based on Error Messages

### "Failed to connect to iZone controller"

**Possible Causes:**
1. iZone controller is powered off
2. Controller is not connected to network
3. Incorrect IP address in configuration
4. Network connectivity issues
5. Firewall blocking connection

**Solutions:**
```bash
# Check if controller IP is pingable
ping 192.168.1.130

# Verify configuration
cat ~/.config/izone/config.toml

# Test HTTP connection
curl http://192.168.1.130/iZoneRequestV2

# Check network connectivity
ip route get 192.168.1.130
```

### "Unexpected response from iZone controller"

**Possible Causes:**
1. Connected to wrong device (not an iZone controller)
2. Controller firmware issue
3. Controller returning error response
4. Network proxy interfering with connection

**Solutions:**
```bash
# Verify the IP actually points to iZone controller
curl -v http://192.168.1.130/iZoneRequestV2

# Check what's responding at that IP
nmap -sV 192.168.1.130

# Try with verbose mode to see raw response
izone -v status

# Verify controller in iZone mobile app
# Open app and check controller IP matches config
```

## Testing Error Messages

### Test Connection Failure

```bash
# Set incorrect IP temporarily
echo 'izone_ip = "http://192.168.1.254"' > ./izone.toml

# Run command to see error
izone status

# Clean up
rm ./izone.toml
```

### Test Invalid Response

```bash
# Point to a web server that returns HTML instead of JSON
echo 'izone_ip = "http://google.com"' > ./izone.toml

# Run command to see error
izone status

# Clean up
rm ./izone.toml
```

## Verbose Mode

For detailed debugging, use verbose mode to see full API responses:

```bash
# Enable verbose output
izone -v status
izone --verbose zone kitchen status

# Shows:
# - Request URL
# - Request Payload
# - Response Status
# - Response Body
```

This helps diagnose:
- Connection issues
- API response problems
- Configuration errors
- Network problems

## Error Message Design Principles

1. **Clear and Concise**: Main message explains what went wrong
2. **Actionable**: Suggests what user should check
3. **Contextual**: Shows relevant information (IP address)
4. **Progressive**: Simple message first, details available
5. **Consistent**: All errors follow same pattern

## Comparison: Old vs New

| Aspect | Old Messages | New Messages |
|--------|--------------|--------------|
| Clarity | Technical, developer-focused | User-friendly, clear |
| Context | Missing IP address | Shows IP being used |
| Guidance | No suggestions | Actionable advice |
| Details | Sometimes hidden in panic | Always available but secondary |
| Consistency | Varied formats | Standardized format |

## Future Enhancements

Potential improvements for future versions:

1. **Auto-detection**: Scan network for iZone controllers
2. **Config validation**: Pre-flight check before commands
3. **Retry logic**: Automatic retry with backoff
4. **Better diagnostics**: Network path testing
5. **Help command**: Built-in troubleshooting wizard

## Related Documentation

- [Configuration Guide](CONFIG.md) - Setting up IP address
- [README](README.md) - General usage and examples
- [Troubleshooting](README.md#troubleshooting) - Common issues and solutions
