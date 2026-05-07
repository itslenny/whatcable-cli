use anyhow::Result;
use clap::Parser;
use regex::Regex;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "export-fixtures")]
#[command(about = "Export ioreg data as test fixtures")]
struct Args {
    /// Output directory for fixtures
    #[arg(short, long, default_value = "tests/fixtures")]
    output: PathBuf,

    /// Anonymize the exported data
    #[arg(short, long)]
    anonymize: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !cfg!(target_os = "macos") {
        eprintln!("Error: This tool only works on macOS");
        std::process::exit(1);
    }

    fs::create_dir_all(&args.output)?;

    println!("Exporting ioreg data to test fixtures...");

    // Export USB-C ports (try all known classes)
    let port_classes = [
        "AppleHPMInterfaceType10",
        "AppleHPMInterfaceType11",
        "AppleHPMInterfaceType12",
        "AppleTCControllerType10",
    ];

    let mut ports_data = Vec::new();
    for class in &port_classes {
        let data = whatcable::ioreg::fetch_raw_ioreg_output(class)?;
        if !data.is_empty() {
            println!("  Found {} ports", class);
            ports_data = data;
            break;
        }
    }

    if ports_data.is_empty() {
        println!("  No USB-C ports found");
        ports_data = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\"><array></array></plist>\n".to_vec();
    }

    // Export power sources
    println!("  Exporting power sources...");
    let power_data = whatcable::ioreg::fetch_raw_ioreg_output("IOPortFeaturePowerSource")?;

    // Export PD identities
    println!("  Exporting PD identities...");
    let pd_data = whatcable::ioreg::fetch_raw_ioreg_output("IOPortTransportComponentCCUSBPDSOP")?;

    // Export USB devices
    println!("  Exporting USB devices...");
    let devices_data = whatcable::ioreg::fetch_raw_ioreg_output("IOUSBHostDevice")?;

    // Anonymize if requested
    let (ports_data, power_data, pd_data, devices_data) = if args.anonymize {
        println!("\nAnonymizing fixture data...");
        (
            anonymize_data(&ports_data),
            anonymize_data(&power_data),
            anonymize_data(&pd_data),
            anonymize_data(&devices_data),
        )
    } else {
        (ports_data, power_data, pd_data, devices_data)
    };

    // Write fixtures
    fs::write(args.output.join("usbc_ports_raw.xml"), ports_data)?;
    fs::write(args.output.join("power_sources_raw.xml"), power_data)?;
    fs::write(args.output.join("pd_identities_raw.xml"), pd_data)?;
    fs::write(args.output.join("usb_devices_raw.xml"), devices_data)?;

    println!("\n✓ Fixtures exported to {}", args.output.display());

    if args.anonymize {
        println!("  Data has been anonymized");
    } else {
        println!("  Real data exported - run with --anonymize to anonymize");
    }

    Ok(())
}

fn anonymize_data(data: &[u8]) -> Vec<u8> {
    let mut text = String::from_utf8_lossy(data).to_string();

    // Define anonymization patterns with their replacement prefix
    let patterns = vec![
        // Product names - catch all product-related string fields
        (
            r"<key>Product</key>\s*<string>[^<]+</string>",
            "Product",
            "PRODUCT",
        ),
        (
            r"<key>USB Product Name</key>\s*<string>[^<]+</string>",
            "USB Product Name",
            "PRODUCT",
        ),
        (
            r"<key>kUSBProductString</key>\s*<string>[^<]+</string>",
            "kUSBProductString",
            "PRODUCT",
        ),
        (
            r"<key>kUSBString</key>\s*<string>[^<]+</string>",
            "kUSBString",
            "PRODUCT",
        ),
        (
            r"<key>IOAudioDeviceModelID</key>\s*<string>[^<]+</string>",
            "IOAudioDeviceModelID",
            "PRODUCT",
        ),
        (
            r"<key>IOAudioDeviceName</key>\s*<string>[^<]+</string>",
            "IOAudioDeviceName",
            "PRODUCT",
        ),
        (
            r"<key>IOAudioEngineDescription</key>\s*<string>[^<]+</string>",
            "IOAudioEngineDescription",
            "PRODUCT",
        ),
        (
            r"<key>IOAudioEngineGlobalUniqueID</key>\s*<string>[^<]+</string>",
            "IOAudioEngineGlobalUniqueID",
            "UNIQUE-ID",
        ),
        (
            r"<key>IORegistryEntryName</key>\s*<string>[^<]+</string>",
            "IORegistryEntryName",
            "ENTRY",
        ),
        // Vendor names
        (
            r"<key>Manufacturer</key>\s*<string>[^<]+</string>",
            "Manufacturer",
            "VENDOR",
        ),
        (
            r"<key>IOAudioDeviceManufacturerName</key>\s*<string>[^<]+</string>",
            "IOAudioDeviceManufacturerName",
            "VENDOR",
        ),
        (
            r"<key>USB Vendor Name</key>\s*<string>[^<]+</string>",
            "USB Vendor Name",
            "VENDOR",
        ),
        (
            r"<key>kUSBVendorString</key>\s*<string>[^<]+</string>",
            "kUSBVendorString",
            "VENDOR",
        ),
        // Serial numbers
        (
            r"<key>USB Serial Number</key>\s*<string>[^<]+</string>",
            "USB Serial Number",
            "SERIAL",
        ),
        (
            r"<key>kUSBSerialNumberString</key>\s*<string>[^<]+</string>",
            "kUSBSerialNumberString",
            "SERIAL",
        ),
        (
            r"<key>Serial Number</key>\s*<string>[^<]+</string>",
            "Serial Number",
            "SERIAL",
        ),
    ];

    for (pattern, key_name, prefix) in patterns {
        let re = Regex::new(pattern).unwrap();
        let mut counter = 1;
        text = re
            .replace_all(&text, |_caps: &regex::Captures| {
                let replacement = format!(
                    "<key>{}</key><string>FAKE-{}-{}</string>",
                    key_name, prefix, counter
                );
                counter += 1;
                replacement
            })
            .to_string();
    }

    text.into_bytes()
}
