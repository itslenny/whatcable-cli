use anyhow::Result;
use plist::Value;
use std::fs;
use std::io::Cursor;

#[test]
fn test_parse_usbc_ports_from_fixture() -> Result<()> {
    let fixture_path = "tests/fixtures/usbc_ports_raw.xml";

    if !std::path::Path::new(fixture_path).exists() {
        eprintln!("Skipping test: fixture not found at {}", fixture_path);
        return Ok(());
    }

    let data = fs::read(fixture_path)?;
    let value: Value = Value::from_reader(Cursor::new(&data))?;

    match value {
        Value::Array(entries) => {
            assert!(!entries.is_empty(), "Expected at least one port entry");
            // Fixture contains internal port entries, expect at least 2 physical ports
            assert!(
                entries.len() >= 2,
                "Expected at least 2 port entries in fixture (found {})",
                entries.len()
            );
        }
        _ => panic!("Expected array of port entries"),
    }

    Ok(())
}

#[test]
fn test_parse_usb_devices_from_fixture() -> Result<()> {
    let fixture_path = "tests/fixtures/usb_devices_raw.xml";

    if !std::path::Path::new(fixture_path).exists() {
        eprintln!("Skipping test: fixture not found at {}", fixture_path);
        return Ok(());
    }

    let data = fs::read(fixture_path)?;
    let value: Value = Value::from_reader(Cursor::new(&data))?;

    match value {
        Value::Array(entries) => {
            assert!(!entries.is_empty(), "Expected at least one device entry");
            // USB device fixture has many nested entries - just verify we have some
            assert!(
                entries.len() >= 3,
                "Expected at least 3 device entries in fixture (found {})",
                entries.len()
            );
        }
        _ => panic!("Expected array of device entries"),
    }

    Ok(())
}

#[test]
fn test_fixture_data_structure() -> Result<()> {
    let fixture_path = "tests/fixtures/usbc_ports_raw.xml";

    if !std::path::Path::new(fixture_path).exists() {
        eprintln!("Skipping test: fixture not found at {}", fixture_path);
        return Ok(());
    }

    let data = fs::read(fixture_path)?;
    let value: Value = Value::from_reader(Cursor::new(&data))?;

    if let Value::Array(entries) = value {
        if let Some(entry) = entries.first() {
            if let Value::Dictionary(dict) = entry {
                assert!(
                    dict.contains_key("IORegistryEntryID"),
                    "Missing IORegistryEntryID"
                );
                assert!(
                    dict.contains_key("IORegistryEntryName"),
                    "Missing IORegistryEntryName"
                );
                assert!(dict.contains_key("IOObjectClass"), "Missing IOObjectClass");
            }
        }
    }

    Ok(())
}

#[test]
fn test_anonymized_data_consistency() -> Result<()> {
    let fixture_path = "tests/fixtures/usb_devices_raw.xml";

    if !std::path::Path::new(fixture_path).exists() {
        eprintln!("Skipping test: fixture not found at {}", fixture_path);
        return Ok(());
    }

    let data = fs::read(fixture_path)?;
    let text = String::from_utf8_lossy(&data);

    // Verify anonymization was applied
    assert!(
        text.contains("FAKE-PRODUCT-"),
        "Expected anonymized product names"
    );
    assert!(
        text.contains("FAKE-VENDOR-"),
        "Expected anonymized vendor names"
    );
    assert!(
        text.contains("FAKE-SERIAL-"),
        "Expected anonymized serial numbers"
    );

    // Verify incremental IDs are present
    assert!(text.contains("FAKE-PRODUCT-1"), "Expected FAKE-PRODUCT-1");
    assert!(text.contains("FAKE-VENDOR-1"), "Expected FAKE-VENDOR-1");

    Ok(())
}

#[test]
fn test_usb_device_has_required_fields() -> Result<()> {
    let fixture_path = "tests/fixtures/usb_devices_raw.xml";

    if !std::path::Path::new(fixture_path).exists() {
        eprintln!("Skipping test: fixture not found at {}", fixture_path);
        return Ok(());
    }

    let data = fs::read(fixture_path)?;
    let value: Value = Value::from_reader(Cursor::new(&data))?;

    if let Value::Array(entries) = value {
        let mut found_device_with_speed = false;

        for entry in entries.iter().take(20) {
            if let Value::Dictionary(dict) = entry {
                // Look for an entry that has speed data (actual USB device, not hub)
                if dict.contains_key("Device Speed") {
                    found_device_with_speed = true;

                    // Verify it has key USB device fields
                    assert!(
                        dict.contains_key("idVendor") || dict.contains_key("USB Vendor Name"),
                        "Device missing vendor information"
                    );
                    assert!(
                        dict.contains_key("idProduct") || dict.contains_key("USB Product Name"),
                        "Device missing product information"
                    );
                    break;
                }
            }
        }

        assert!(
            found_device_with_speed,
            "No USB devices with speed data found in fixture"
        );
    }

    Ok(())
}

#[test]
fn test_port_has_transport_info() -> Result<()> {
    let fixture_path = "tests/fixtures/usbc_ports_raw.xml";

    if !std::path::Path::new(fixture_path).exists() {
        eprintln!("Skipping test: fixture not found at {}", fixture_path);
        return Ok(());
    }

    let data = fs::read(fixture_path)?;
    let value: Value = Value::from_reader(Cursor::new(&data))?;

    if let Value::Array(entries) = value {
        let mut found_port_with_transports = false;

        for entry in &entries {
            if let Value::Dictionary(dict) = entry {
                // Check if this is a real port with transport info
                if dict.contains_key("TransportsSupported") || dict.contains_key("TransportsActive")
                {
                    found_port_with_transports = true;

                    // Verify it has other port-related fields
                    assert!(
                        dict.contains_key("IORegistryEntryID"),
                        "Port missing IORegistryEntryID"
                    );
                    break;
                }
            }
        }

        assert!(
            found_port_with_transports,
            "No ports with transport info found in fixture"
        );
    }

    Ok(())
}
