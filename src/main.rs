mod ioreg;
mod models;
mod pd_vdo;
mod port_summary;
mod vendor_db;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "whatcable")]
#[command(about = "Show USB-C cable capabilities on macOS", long_about = None)]
#[command(version)]
struct Args {
    /// Show all ports, including those with nothing connected
    #[arg(short, long)]
    all: bool,

    /// Show technical details (raw IOKit properties)
    #[arg(short, long)]
    technical: bool,

    /// Output in JSON format
    #[arg(short, long)]
    json: bool,
}

#[cfg(not(target_os = "macos"))]
compile_error!("This tool only works on macOS");
fn main() -> Result<()> {
    if !cfg!(target_os = "macos") {
        eprintln!("Error: whatcable only works on macOS");
        std::process::exit(1);
    }
    let args = Args::parse();

    // Fetch all USB-C port data from IOKit via ioreg
    let ports = ioreg::fetch_usbc_ports()?;
    let power_sources = ioreg::fetch_power_sources()?;
    let identities = ioreg::fetch_pd_identities()?;
    let usb_devices = ioreg::fetch_usb_devices()?;

    // Filter ports if --all is not specified
    let ports_to_show: Vec<_> = if args.all {
        ports.iter().collect()
    } else {
        ports
            .iter()
            .filter(|p| p.connection_active.unwrap_or(false))
            .collect()
    };

    if ports_to_show.is_empty() {
        if args.all {
            println!("No USB-C ports found on this system.");
        } else {
            println!("No cables connected. Use --all to show all ports.");
        }
        return Ok(());
    }

    if args.json {
        // JSON output
        let output = serde_json::json!({
            "ports": ports_to_show,
            "power_sources": power_sources,
            "identities": identities,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Human-readable output
        for (i, port) in ports_to_show.iter().enumerate() {
            if i > 0 {
                println!();
            }

            // Match power sources and identities for this port
            let port_sources = port_summary::find_port_sources(port, &power_sources);
            let port_identities = port_summary::find_port_identities(port, &identities);

            let summary = port_summary::PortSummary::new(port, &port_sources, &port_identities);

            println!("━━━ {} ━━━", port.service_name);
            println!("📍 {}", summary.headline);
            println!("   {}", summary.subtitle);

            if !summary.bullets.is_empty() {
                println!();
                for bullet in &summary.bullets {
                    println!("   • {}", bullet);
                }
            }

            if args.technical {
                println!("\n   Technical Details:");
                println!(
                    "   Port Number: {}",
                    port.port_number
                        .map_or("N/A".to_string(), |v| v.to_string())
                );
                println!(
                    "   Connection Active: {}",
                    port.connection_active
                        .map_or("N/A".to_string(), |v| v.to_string())
                );
                println!(
                    "   Active Cable: {}",
                    port.active_cable
                        .map_or("N/A".to_string(), |v| v.to_string())
                );
                println!("   Transports Active: {:?}", port.transports_active);
                println!("   Transports Supported: {:?}", port.transports_supported);
                println!("   USB Mode Type: {}", port.usb_mode_type_label());
                println!("   Plug Orientation: {}", port.plug_orientation_label());
                println!(
                    "   Bus Index: {}",
                    port.bus_index.map_or("N/A".to_string(), |v| v.to_string())
                );

                // Show PD identity details if available
                if !port_identities.is_empty() {
                    println!("   PD Identities:");
                    for identity in &port_identities {
                        println!(
                            "     {:?}: VID=0x{:04X} PID=0x{:04X}",
                            identity.endpoint, identity.vendor_id, identity.product_id
                        );
                        if let Some(header) = identity.id_header() {
                            if header.usb_comm_host {
                                println!("       ↳ USB host capable");
                            }
                            if header.usb_comm_device {
                                println!("       ↳ USB device capable");
                            }
                            if header.modal_operation {
                                println!("       ↳ Supports alternate modes");
                            }
                        }
                    }
                }
            }

            // Show USB devices connected to this port (if we can determine bus index)
            if let Some(port_bus) = port.bus_index {
                let port_devices: Vec<_> = usb_devices
                    .iter()
                    .filter(|d| d.bus_index == Some(port_bus))
                    .collect();

                if !port_devices.is_empty() {
                    println!("\n   Connected USB Devices:");
                    for device in port_devices {
                        println!("   • {} — {}", device.display_name(), device.speed_label());
                    }
                }
            }
        }
    }

    Ok(())
}
