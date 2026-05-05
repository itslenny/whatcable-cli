use crate::models::{PDEndpoint, PDIdentity, PowerSource, USBCPort};
use crate::vendor_db;

pub struct PortSummary {
    pub headline: String,
    pub subtitle: String,
    pub bullets: Vec<String>,
}

impl PortSummary {
    pub fn new(port: &USBCPort, sources: &[&PowerSource], identities: &[&PDIdentity]) -> Self {
        let connected = port.connection_active.unwrap_or(false);
        let active = &port.transports_active;
        let _supported = &port.transports_supported;

        let has_usb3 =
            active.contains(&"USB3".to_string()) || port.super_speed_active.unwrap_or(false);
        let has_usb2 = active.contains(&"USB2".to_string());
        let has_tb = active.contains(&"CIO".to_string()); // Thunderbolt = Converged I/O
        let has_dp = active.contains(&"DisplayPort".to_string());
        let has_emarker = port.active_cable.unwrap_or(false);
        let port_label = port
            .port_description
            .as_deref()
            .unwrap_or(&port.service_name);

        if !connected {
            return PortSummary {
                headline: "Nothing connected".to_string(),
                subtitle: format!("Plug a cable into {} to see what it can do.", port_label),
                bullets: Vec::new(),
            };
        }

        let mut bullets = Vec::new();

        // Speed
        if has_tb {
            bullets.push("Thunderbolt / USB4 link active".to_string());
        } else if has_usb3 {
            bullets.push("SuperSpeed USB (5 Gbps or faster)".to_string());
        } else if has_usb2 {
            bullets.push("USB 2.0 only (480 Mbps) — no high-speed data".to_string());
        }

        if has_dp {
            bullets.push("Carrying DisplayPort video".to_string());
        }

        // E-marker
        if has_emarker {
            bullets.push("Cable has an e-marker chip (advertises its capabilities)".to_string());
        } else if !active.is_empty() {
            bullets.push("Cable does not advertise an e-marker (basic cable)".to_string());
        }

        if port.optical_cable.unwrap_or(false) {
            bullets.push("Optical cable".to_string());
        }

        // Power summary from PD power sources
        let usb_pd = sources.iter().find(|s| s.name == "USB-PD");
        if let Some(usb_pd) = usb_pd {
            let max_w = (usb_pd.max_power_mw() as f64 / 1000.0).round() as i64;
            let has_options = !usb_pd.options.is_empty();
            if has_options && max_w > 0 {
                bullets.push(format!("Charger advertises up to {}W", max_w));
            }
            if let Some(win) = &usb_pd.winning {
                bullets.push(format!(
                    "Currently negotiated: {} @ {} ({})",
                    win.volts_label(),
                    win.amps_label(),
                    win.watts_label()
                ));
            }
        }

        if let Some(count) = port.overcurrent_count {
            if count > 0 {
                bullets.push(format!("⚠️  {} overcurrent event(s) detected", count));
            }
        }

        // Cable e-marker (SOP'): the cable's own capabilities
        let cable_emarker = identities.iter().find(|id| {
            matches!(
                id.endpoint,
                PDEndpoint::SopPrime | PDEndpoint::SopDoublePrime
            )
        });

        if let Some(cable) = cable_emarker {
            if let Some(cv) = cable.cable_vdo() {
                bullets.push(format!("Cable speed: {}", cv.speed.label()));
                bullets.push(format!(
                    "Cable rated for {} at up to {}V (~{}W)",
                    cv.current.label(),
                    cv.max_volts(),
                    cv.max_watts
                ));
                if matches!(cv.cable_type, crate::pd_vdo::CableType::Active) {
                    bullets.push(
                        "Active cable (contains signal-conditioning electronics)".to_string(),
                    );
                }
                if cv.vbus_through_cable {
                    bullets.push("Cable carries VBUS power".to_string());
                }
            }
        }

        // Partner identity (SOP): what's connected
        if let Some(partner) = identities.iter().find(|id| id.endpoint == PDEndpoint::Sop) {
            if let Some(header) = partner.id_header() {
                let kind = if header.ufp_product_type != crate::pd_vdo::ProductType::Undefined {
                    header.ufp_product_type.label()
                } else {
                    header.dfp_product_type.label()
                };
                bullets.push(format!(
                    "Connected device: {} — {}",
                    kind,
                    vendor_db::label(partner.vendor_id)
                ));
            }
        }

        // Cable e-marker vendor (SOP'): who made the cable
        if let Some(cable) = cable_emarker {
            if cable.vendor_id != 0 {
                bullets.push(format!(
                    "Cable made by {}",
                    vendor_db::label(cable.vendor_id)
                ));
            }
        }

        // Headline + status
        let charger_w: Option<i64> = usb_pd.and_then(|pd| {
            if pd.options.is_empty() {
                None
            } else {
                let w = (pd.max_power_mw() as f64 / 1000.0).round() as i64;
                if w > 0 {
                    Some(w)
                } else {
                    None
                }
            }
        });

        let charger_suffix = charger_w
            .map(|w| format!(" · {}W charger", w))
            .unwrap_or_default();

        let (headline, subtitle) = if has_tb {
            (
                format!("Thunderbolt / USB4{}", charger_suffix),
                subtitle_for_capabilities(has_usb3, has_dp, has_emarker),
            )
        } else if has_usb3 && has_dp {
            (
                format!("USB-C with video{}", charger_suffix),
                "Carrying both data and DisplayPort video.".to_string(),
            )
        } else if has_dp {
            (
                format!("Display connected{}", charger_suffix),
                "DisplayPort video over USB-C alt mode.".to_string(),
            )
        } else if has_usb3 {
            (
                format!("USB device{}", charger_suffix),
                "SuperSpeed data link is active.".to_string(),
            )
        } else if has_usb2 && !has_usb3 {
            (
                format!("Slow USB device or charge-only cable{}", charger_suffix),
                "Only USB 2.0 is active. If you expected high speed, the cable may not support it."
                    .to_string(),
            )
        } else if usb_pd.is_some() {
            (
                format!("Charging{}", charger_suffix),
                "Power is flowing. No data connection.".to_string(),
            )
        } else if active.is_empty() && port.transports_supported.contains(&"USB2".to_string()) {
            (
                "Charging only".to_string(),
                "Power is flowing but no data link is established.".to_string(),
            )
        } else {
            (
                "Connected".to_string(),
                "Couldn't determine cable type from this port.".to_string(),
            )
        };

        PortSummary {
            headline,
            subtitle,
            bullets,
        }
    }
}

fn subtitle_for_capabilities(usb3: bool, dp: bool, emarker: bool) -> String {
    let mut parts = Vec::new();
    if usb3 {
        parts.push("high-speed data");
    }
    if dp {
        parts.push("video");
    }
    if emarker {
        parts.push("smart cable");
    }
    if parts.is_empty() {
        "Connected.".to_string()
    } else {
        format!("Supports {}.", parts.join(", "))
    }
}

pub fn find_port_sources<'a>(port: &USBCPort, sources: &'a [PowerSource]) -> Vec<&'a PowerSource> {
    if let Some(key) = port.port_key() {
        sources.iter().filter(|s| s.port_key() == key).collect()
    } else {
        Vec::new()
    }
}

pub fn find_port_identities<'a>(
    port: &USBCPort,
    identities: &'a [PDIdentity],
) -> Vec<&'a PDIdentity> {
    if let Some(key) = port.port_key() {
        identities
            .iter()
            .filter(|id| id.port_key() == key)
            .collect()
    } else {
        Vec::new()
    }
}
