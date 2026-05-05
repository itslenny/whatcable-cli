use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USBCPort {
    pub id: u64,
    pub service_name: String,
    pub class_name: String,
    pub port_description: Option<String>,
    pub port_type_description: Option<String>,
    pub port_number: Option<i64>,
    pub connection_active: Option<bool>,
    pub active_cable: Option<bool>,
    pub optical_cable: Option<bool>,
    pub usb_active: Option<bool>,
    pub super_speed_active: Option<bool>,
    pub usb_mode_type: Option<i64>,
    pub usb_connect_string: Option<String>,
    pub transports_supported: Vec<String>,
    pub transports_active: Vec<String>,
    pub transports_provisioned: Vec<String>,
    pub plug_orientation: Option<i64>,
    pub plug_event_count: Option<i64>,
    pub connection_count: Option<i64>,
    pub overcurrent_count: Option<i64>,
}

impl USBCPort {
    pub fn port_key(&self) -> Option<(i64, i64)> {
        // Port type 2 = USB-C, 17 = MagSafe 3
        let port_type = match self.port_type_description.as_deref() {
            Some("USB-C") => Some(2),
            Some(desc) if desc.starts_with("MagSafe") => Some(17),
            _ => None,
        }?;

        let port_num = self.port_number?;
        Some((port_type, port_num))
    }

    pub fn plug_orientation_label(&self) -> &str {
        match self.plug_orientation {
            Some(0) => "Normal",
            Some(1) => "Flipped",
            _ => "Unknown",
        }
    }

    pub fn usb_mode_type_label(&self) -> &str {
        match self.usb_mode_type {
            Some(0) => "None",
            Some(1) => "Host",
            Some(2) => "Device",
            Some(3) => "DRD (Dual-Role)",
            _ => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerSource {
    pub id: u64,
    pub name: String,
    pub parent_port_type: i64,
    pub parent_port_number: i64,
    pub options: Vec<PowerOption>,
    pub winning: Option<PowerOption>,
}

impl PowerSource {
    pub fn port_key(&self) -> (i64, i64) {
        (self.parent_port_type, self.parent_port_number)
    }

    pub fn max_power_mw(&self) -> i64 {
        self.options
            .iter()
            .map(|o| o.max_power_mw)
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerOption {
    pub voltage_mv: i64,
    pub max_current_ma: i64,
    pub max_power_mw: i64,
}

impl PowerOption {
    pub fn volts_label(&self) -> String {
        format!("{:.1}V", self.voltage_mv as f64 / 1000.0)
    }

    pub fn amps_label(&self) -> String {
        format!("{:.2}A", self.max_current_ma as f64 / 1000.0)
    }

    pub fn watts_label(&self) -> String {
        format!("{}W", (self.max_power_mw as f64 / 1000.0).round() as i64)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PDIdentity {
    pub id: u64,
    pub endpoint: PDEndpoint,
    pub parent_port_type: i64,
    pub parent_port_number: i64,
    pub vendor_id: i64,
    pub product_id: i64,
    pub bcd_device: i64,
    pub vdos: Vec<u32>,
    pub spec_revision: i64,
}

impl PDIdentity {
    pub fn port_key(&self) -> (i64, i64) {
        (self.parent_port_type, self.parent_port_number)
    }

    pub fn id_header(&self) -> Option<crate::pd_vdo::IDHeader> {
        self.vdos
            .first()
            .map(|&vdo| crate::pd_vdo::decode_id_header(vdo))
    }

    pub fn cable_vdo(&self) -> Option<crate::pd_vdo::CableVDO> {
        if !matches!(
            self.endpoint,
            PDEndpoint::SopPrime | PDEndpoint::SopDoublePrime
        ) {
            return None;
        }

        // Cable VDO is typically VDO[3] in PD 3.0+
        let vdo = self.vdos.get(3)?;
        let is_active = self
            .id_header()
            .map(|h| matches!(h.ufp_product_type, crate::pd_vdo::ProductType::ActiveCable))
            .unwrap_or(false);

        Some(crate::pd_vdo::decode_cable_vdo(*vdo, is_active))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PDEndpoint {
    Sop,
    SopPrime,
    SopDoublePrime,
    Unknown,
}

impl PDEndpoint {
    pub fn from_str(s: &str) -> Self {
        match s {
            "SOP" => PDEndpoint::Sop,
            "SOP'" => PDEndpoint::SopPrime,
            "SOP''" => PDEndpoint::SopDoublePrime,
            _ => PDEndpoint::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USBDevice {
    pub id: u64,
    pub location_id: u32,
    pub vendor_id: u16,
    pub product_id: u16,
    pub vendor_name: Option<String>,
    pub product_name: Option<String>,
    pub serial_number: Option<String>,
    pub usb_version: Option<String>,
    pub speed_raw: Option<u8>,
    pub bus_power_ma: Option<i64>,
    pub current_ma: Option<i64>,
}

impl USBDevice {
    pub fn speed_label(&self) -> &str {
        match self.speed_raw {
            Some(0) => "Low Speed (1.5 Mbps)",
            Some(1) => "Full Speed (12 Mbps)",
            Some(2) => "High Speed (480 Mbps)",
            Some(3) => "SuperSpeed (5 Gbps)",
            Some(4) => "SuperSpeed+ (10 Gbps)",
            Some(5) => "SuperSpeed+ Gen 2x2 (20 Gbps)",
            _ => "Unknown speed",
        }
    }

    pub fn display_name(&self) -> String {
        self.product_name
            .clone()
            .or_else(|| self.vendor_name.clone())
            .unwrap_or_else(|| format!("USB Device (VID: 0x{:04X})", self.vendor_id))
    }
}
