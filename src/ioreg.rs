use anyhow::{Context, Result};
use core_foundation::base::TCFType;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use io_kit_sys::*;
use plist::Value;
use std::ffi::CStr;
use std::io::Cursor;
use std::process::Command;

use crate::models::{PDEndpoint, PDIdentity, PowerOption, PowerSource, USBCPort, USBDevice};

/// Capture raw ioreg output for test fixtures
#[allow(dead_code)]
pub fn fetch_raw_ioreg_output(class: &str) -> Result<Vec<u8>> {
    let output = Command::new("ioreg")
        .args(["-c", class, "-r", "-l", "-a"])
        .output()
        .context(format!("Failed to execute ioreg for class {}", class))?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    Ok(output.stdout)
}

/// Fetch all USB-C / MagSafe port controllers from IOKit.
/// Matches classes: AppleHPMInterfaceType10/11/12 (M3+) and AppleTCControllerType10 (M1/M2).
pub fn fetch_usbc_ports() -> Result<Vec<USBCPort>> {
    let classes = [
        "AppleHPMInterfaceType10",
        "AppleHPMInterfaceType11",
        "AppleHPMInterfaceType12",
        "AppleTCControllerType10",
    ];

    let mut all_ports = Vec::new();

    for class in &classes {
        let output = Command::new("ioreg")
            .args(["-c", class, "-r", "-l", "-a"])
            .output()
            .context("Failed to execute ioreg for USB-C ports")?;

        if !output.status.success() || output.stdout.is_empty() {
            continue; // This class may not exist on this system
        }

        let value = match Value::from_reader(Cursor::new(&output.stdout)) {
            Ok(v) => v,
            Err(_) => continue, // Skip if parsing fails for this class
        };

        if let Value::Array(entries) = value {
            for entry in entries {
                if let Some(port) = parse_usbc_port(&entry) {
                    // Only include if it's a real physical port
                    if is_physical_port(&port) {
                        all_ports.push(port);
                    }
                }
            }
        }
    }

    Ok(all_ports)
}

fn is_physical_port(port: &USBCPort) -> bool {
    // Real ports have PortTypeDescription and service name starts with "Port-"
    let has_port_type = port.port_type_description.is_some();
    let is_named_port = port.service_name.starts_with("Port-");
    has_port_type && is_named_port
}

fn parse_usbc_port(entry: &Value) -> Option<USBCPort> {
    let dict = entry.as_dictionary()?;

    let id = dict
        .get("IORegistryEntryID")?
        .as_unsigned_integer()
        .unwrap_or(0);

    let service_name = dict
        .get("IORegistryEntryName")?
        .as_string()
        .unwrap_or("")
        .to_string();

    let class_name = dict
        .get("IOObjectClass")?
        .as_string()
        .unwrap_or("")
        .to_string();

    let bus_index = calculate_port_bus_index(id);

    Some(USBCPort {
        id,
        service_name,
        class_name,
        port_description: get_string(dict, "PortDescription"),
        port_type_description: get_string(dict, "PortTypeDescription"),
        port_number: get_integer(dict, "PortNumber"),
        connection_active: get_boolean(dict, "ConnectionActive"),
        active_cable: get_boolean(dict, "ActiveCable"),
        optical_cable: get_boolean(dict, "OpticalCable"),
        usb_active: get_boolean(dict, "IOAccessoryUSBActive"),
        super_speed_active: get_boolean(dict, "IOAccessoryUSBSuperSpeedActive"),
        usb_mode_type: get_integer(dict, "IOAccessoryUSBModeType"),
        usb_connect_string: get_string(dict, "IOAccessoryUSBConnectString"),
        transports_supported: get_string_array(dict, "TransportsSupported"),
        transports_active: get_string_array(dict, "TransportsActive"),
        transports_provisioned: get_string_array(dict, "TransportsProvisioned"),
        plug_orientation: get_integer(dict, "PlugOrientation"),
        plug_event_count: get_integer(dict, "Plug Event Count"),
        connection_count: get_integer(dict, "ConnectionCount"),
        overcurrent_count: get_integer(dict, "Overcurrent Count"),
        bus_index,
    })
}

/// Fetch all power source services from IOKit.
pub fn fetch_power_sources() -> Result<Vec<PowerSource>> {
    let output = Command::new("ioreg")
        .args(["-c", "IOPortFeaturePowerSource", "-r", "-l", "-a"])
        .output()
        .context("Failed to execute ioreg for power sources")?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let value = Value::from_reader(Cursor::new(&output.stdout))
        .context("Failed to parse ioreg output for power sources")?;

    let mut sources = Vec::new();

    if let Value::Array(entries) = value {
        for entry in entries {
            if let Some(source) = parse_power_source(&entry) {
                sources.push(source);
            }
        }
    }

    Ok(sources)
}

fn parse_power_source(entry: &Value) -> Option<PowerSource> {
    let dict = entry.as_dictionary()?;

    let id = dict
        .get("IORegistryEntryID")?
        .as_unsigned_integer()
        .unwrap_or(0);

    let name = get_string(dict, "PowerSourceName").unwrap_or_else(|| "Unknown".to_string());
    let parent_port_type = get_integer(dict, "ParentPortType").unwrap_or(0);
    let parent_port_number = get_integer(dict, "ParentPortNumber").unwrap_or(0);

    let options = parse_power_options(dict.get("PowerSourceOptions"));
    let winning = dict
        .get("WinningPowerSourceOption")
        .and_then(parse_power_option);

    Some(PowerSource {
        id,
        name,
        parent_port_type,
        parent_port_number,
        options,
        winning,
    })
}

fn parse_power_options(value: Option<&Value>) -> Vec<PowerOption> {
    let array = match value {
        Some(Value::Array(a)) => a,
        _ => return Vec::new(),
    };

    array.iter().filter_map(parse_power_option).collect()
}

fn parse_power_option(value: &Value) -> Option<PowerOption> {
    let dict = value.as_dictionary()?;

    let voltage_mv = get_integer(dict, "Voltage (mV)").unwrap_or(0);
    let max_current_ma = get_integer(dict, "Max Current (mA)").unwrap_or(0);
    let max_power_mw =
        get_integer(dict, "Max Power (mW)").unwrap_or_else(|| (voltage_mv * max_current_ma) / 1000);

    if voltage_mv <= 0 {
        return None;
    }

    Some(PowerOption {
        voltage_mv,
        max_current_ma,
        max_power_mw,
    })
}

/// Fetch all PD identity services from IOKit.
pub fn fetch_pd_identities() -> Result<Vec<PDIdentity>> {
    let output = Command::new("ioreg")
        .args(["-c", "IOPortTransportComponentCCUSBPDSOP", "-r", "-l", "-a"])
        .output()
        .context("Failed to execute ioreg for PD identities")?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let value = Value::from_reader(Cursor::new(&output.stdout))
        .context("Failed to parse ioreg output for PD identities")?;

    let mut identities = Vec::new();

    if let Value::Array(entries) = value {
        for entry in entries {
            if let Some(identity) = parse_pd_identity(&entry) {
                identities.push(identity);
            }
        }
    }

    Ok(identities)
}

fn parse_pd_identity(entry: &Value) -> Option<PDIdentity> {
    let dict = entry.as_dictionary()?;

    let id = dict
        .get("IORegistryEntryID")?
        .as_unsigned_integer()
        .unwrap_or(0);

    let endpoint_name = get_string(dict, "ComponentName")
        .or_else(|| get_string(dict, "AddressDescription"))
        .or_else(|| get_string(dict, "Address Description"))
        .unwrap_or_else(|| "Unknown".to_string());

    let endpoint = PDEndpoint::from_string(&endpoint_name);

    let parent_port_type = get_integer(dict, "ParentPortType").unwrap_or(0);
    let parent_port_number = get_integer(dict, "ParentPortNumber").unwrap_or(0);
    let spec_revision = get_integer(dict, "Specification Revision").unwrap_or(0);

    // Try to get vendor ID and product ID from Metadata or directly
    let metadata = dict.get("Metadata").and_then(|v| v.as_dictionary());

    let vendor_id = metadata
        .and_then(|m| get_integer(m, "Vendor ID"))
        .or_else(|| get_integer(dict, "Vendor ID"))
        .unwrap_or(0);

    let product_id = metadata
        .and_then(|m| get_integer(m, "Product ID"))
        .or_else(|| get_integer(dict, "Product ID"))
        .unwrap_or(0);

    let bcd_device = metadata
        .and_then(|m| get_integer(m, "bcdDevice"))
        .unwrap_or(0);

    // Parse VDOs from Metadata
    let vdos = metadata
        .and_then(|m| m.get("VDOs"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|val| {
                    if let Some(data) = val.as_data() {
                        vdo_from_data(data)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_else(Vec::new);

    Some(PDIdentity {
        id,
        endpoint,
        parent_port_type,
        parent_port_number,
        vendor_id,
        product_id,
        bcd_device,
        vdos,
        spec_revision,
    })
}

/// Decode a VDO from a 4-byte little-endian Data blob.
fn vdo_from_data(data: &[u8]) -> Option<u32> {
    if data.len() < 4 {
        return None;
    }
    Some(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
}

// Helper functions to extract values from plist dictionaries

fn get_string(dict: &plist::Dictionary, key: &str) -> Option<String> {
    dict.get(key)?.as_string().map(|s| s.to_string())
}

fn get_integer(dict: &plist::Dictionary, key: &str) -> Option<i64> {
    match dict.get(key)? {
        Value::Integer(val) => Some(val.as_signed()?),
        Value::Real(val) => Some(*val as i64),
        Value::Boolean(b) => Some(if *b { 1 } else { 0 }),
        _ => None,
    }
}

fn get_boolean(dict: &plist::Dictionary, key: &str) -> Option<bool> {
    match dict.get(key)? {
        Value::Boolean(b) => Some(*b),
        Value::Integer(val) => Some(val.as_signed()? != 0),
        _ => None,
    }
}

fn get_string_array(dict: &plist::Dictionary, key: &str) -> Vec<String> {
    let array = match dict.get(key) {
        Some(Value::Array(a)) => a,
        _ => return Vec::new(),
    };

    array
        .iter()
        .filter_map(|v| v.as_string().map(|s| s.to_string()))
        .collect()
}

/// Fetch all USB devices from IOKit.
pub fn fetch_usb_devices() -> Result<Vec<USBDevice>> {
    let output = Command::new("ioreg")
        .args(["-c", "IOUSBHostDevice", "-r", "-l", "-a"])
        .output()
        .context("Failed to execute ioreg for USB devices")?;

    if !output.status.success() || output.stdout.is_empty() {
        return Ok(Vec::new());
    }

    let value = Value::from_reader(Cursor::new(&output.stdout))
        .context("Failed to parse ioreg output for USB devices")?;

    let mut devices = Vec::new();

    if let Value::Array(entries) = value {
        for entry in entries {
            collect_usb_devices_recursive(&entry, &mut devices);
        }
    }

    // Deduplicate by location_id + product_name (IOKit creates multiple interface entries)
    // Keep only entries that have a valid speed (these are the actual device entries, not interfaces)
    devices.retain(|d| d.speed_raw.is_some());

    // Further deduplicate by (location_id, vendor_id, product_id) combination
    devices.sort_by_key(|d| (d.location_id, d.vendor_id, d.product_id));
    devices.dedup_by_key(|d| (d.location_id, d.vendor_id, d.product_id));

    // Sort by product name
    devices.sort_by_key(|a| a.display_name());

    Ok(devices)
}

/// Recursively collect USB devices from the IORegistry tree.
/// Devices can be nested within hubs, so we need to traverse IORegistryEntryChildren.
fn collect_usb_devices_recursive(entry: &Value, devices: &mut Vec<USBDevice>) {
    // Try to parse this entry as a USB device
    if let Some(device) = parse_usb_device(entry) {
        devices.push(device);
    }

    // Recursively process children
    if let Some(dict) = entry.as_dictionary() {
        if let Some(Value::Array(children)) = dict.get("IORegistryEntryChildren") {
            for child in children {
                collect_usb_devices_recursive(child, devices);
            }
        }
    }
}

fn parse_usb_device(entry: &Value) -> Option<USBDevice> {
    let dict = entry.as_dictionary()?;

    let id = dict
        .get("IORegistryEntryID")?
        .as_unsigned_integer()
        .unwrap_or(0);

    let location_id = get_integer(dict, "locationID").unwrap_or(0) as u32;
    let vendor_id = get_integer(dict, "idVendor").unwrap_or(0) as u16;
    let product_id = get_integer(dict, "idProduct").unwrap_or(0) as u16;

    let speed_raw = get_integer(dict, "Device Speed").map(|v| v as u8);
    let bcd_usb = get_integer(dict, "bcdUSB").map(|v| v as u16);
    let bus_power = get_integer(dict, "Bus Power Available").map(|v| v * 2);
    let current = get_integer(dict, "Requested Power").map(|v| v * 2);

    let usb_version = bcd_usb.map(format_bcd);

    let vendor_name = get_string(dict, "USB Vendor Name");
    let product_name = get_string(dict, "USB Product Name");

    // Skip devices without meaningful identification
    if vendor_name.is_none() && product_name.is_none() && vendor_id == 0 {
        return None;
    }

    let bus_index = calculate_device_bus_index(id, location_id);

    Some(USBDevice {
        id,
        location_id,
        vendor_id,
        product_id,
        vendor_name,
        product_name,
        serial_number: get_string(dict, "USB Serial Number"),
        usb_version,
        speed_raw,
        bus_power_ma: bus_power,
        current_ma: current,
        bus_index,
    })
}

fn format_bcd(value: u16) -> String {
    let major = (value >> 8) & 0xFF;
    let minor = (value >> 4) & 0xF;
    let sub = value & 0xF;
    if sub == 0 {
        format!("{}.{}", major, minor)
    } else {
        format!("{}.{}.{}", major, minor, sub)
    }
}

/// Calculate bus index for a USB-C port by walking IOKit parent chain
/// looking for hpm<N> ancestor. Returns N if found (M3+ machines).
fn calculate_port_bus_index(entry_id: u64) -> Option<i64> {
    unsafe {
        let mut service =
            IOServiceGetMatchingService(kIOMasterPortDefault, IORegistryEntryIDMatching(entry_id));
        if service == 0 {
            return None;
        }

        for _ in 0..8 {
            let mut parent = 0;
            let result = IORegistryEntryGetParentEntry(service, c"IOService".as_ptr(), &mut parent);
            IOObjectRelease(service);
            if result != 0 {
                return None;
            }
            service = parent;

            let mut name_buf = [0i8; 128];
            IORegistryEntryGetName(service, name_buf.as_mut_ptr());
            let name = CStr::from_ptr(name_buf.as_ptr()).to_str().unwrap_or("");

            if let Some(after_hpm) = name.strip_prefix("hpm") {
                // Extract number after "hpm" - could be "hpm0" or "hpm0@..."
                let digit_str: String = after_hpm
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();

                if let Ok(n) = digit_str.parse::<i64>() {
                    IOObjectRelease(service);
                    return Some(n);
                }
            }
        }
        IOObjectRelease(service);
        None
    }
}

/// Calculate bus index for a USB device by walking IOKit parent chain
/// to find AppleT*USBXHCI ancestor and reading its locationID upper byte.
/// Falls back to device's own locationID upper byte.
fn calculate_device_bus_index(entry_id: u64, location_id: u32) -> Option<i64> {
    unsafe {
        let mut service =
            IOServiceGetMatchingService(kIOMasterPortDefault, IORegistryEntryIDMatching(entry_id));
        if service == 0 {
            // Fallback: use device's own locationID upper byte
            return Some(((location_id >> 24) & 0xFF) as i64);
        }

        for _ in 0..8 {
            let mut parent = 0;
            let result = IORegistryEntryGetParentEntry(service, c"IOService".as_ptr(), &mut parent);
            IOObjectRelease(service);
            if result != 0 {
                break;
            }
            service = parent;

            let mut class_buf = [0i8; 128];
            IOObjectGetClass(service, class_buf.as_mut_ptr());
            let class_name = CStr::from_ptr(class_buf.as_ptr()).to_str().unwrap_or("");

            if class_name.starts_with("AppleT") && class_name.ends_with("USBXHCI") {
                // Found XHCI controller, read its locationID
                let loc_key = CFString::from("locationID");
                let prop = IORegistryEntryCreateCFProperty(
                    service,
                    loc_key.as_concrete_TypeRef(),
                    std::ptr::null_mut(),
                    0,
                );
                IOObjectRelease(service);

                if !prop.is_null() {
                    let num = CFNumber::wrap_under_create_rule(prop as *const _);
                    if let Some(val) = num.to_i64() {
                        return Some((val >> 24) & 0xFF);
                    }
                }
                break;
            }
        }
        IOObjectRelease(service);

        // Fallback: use device's own locationID upper byte
        Some(((location_id >> 24) & 0xFF) as i64)
    }
}
