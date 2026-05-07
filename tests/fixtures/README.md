# Test Fixtures

This directory contains anonymized IOKit registry data exported from a real macOS system. These fixtures are used for integration tests and can run on any platform (not just macOS).

## Files

- `usbc_ports_raw.xml` - USB-C port controller data
- `power_sources_raw.xml` - Power source data
- `pd_identities_raw.xml` - USB Power Delivery identity data
- `usb_devices_raw.xml` - Connected USB device data

## Regenerating Fixtures

To regenerate the fixtures from your current hardware setup:

```bash
# Export real data
cargo run --bin export-fixtures

# Export anonymized data
cargo run --bin export-fixtures -- --anonymize
```

## Data Privacy

All fixtures in this directory have been anonymized to protect hardware-specific identifying information while preserving the data structure needed for testing.
