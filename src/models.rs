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
    /// Bus index derived from hpm<N> ancestor in IOKit parent chain (M3+).
    /// Used to match devices to their physical port. None on M1/M2.
    pub bus_index: Option<i64>,
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
            Some(1) => "Normal",
            _ => "Flipped",
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
    pub fn from_string(s: &str) -> Self {
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
    /// Bus index from XHCI controller (upper byte of locationID).
    /// Used to match device to physical port. None if derivation failed.
    pub bus_index: Option<i64>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usbc_port_orientation_labels() {
        let mut port = create_test_port();

        port.plug_orientation = Some(1);
        assert_eq!(port.plug_orientation_label(), "Normal");

        port.plug_orientation = Some(2);
        assert_eq!(port.plug_orientation_label(), "Flipped");

        port.plug_orientation = Some(0);
        assert_eq!(port.plug_orientation_label(), "Flipped");

        port.plug_orientation = None;
        assert_eq!(port.plug_orientation_label(), "Flipped");
    }

    #[test]
    fn test_usbc_port_mode_type_labels() {
        let mut port = create_test_port();

        port.usb_mode_type = Some(0);
        assert_eq!(port.usb_mode_type_label(), "None");

        port.usb_mode_type = Some(1);
        assert_eq!(port.usb_mode_type_label(), "Host");

        port.usb_mode_type = Some(2);
        assert_eq!(port.usb_mode_type_label(), "Device");

        port.usb_mode_type = Some(3);
        assert_eq!(port.usb_mode_type_label(), "DRD (Dual-Role)");
    }

    #[test]
    fn test_usbc_port_key() {
        let mut port = create_test_port();

        port.port_type_description = Some("USB-C".to_string());
        port.port_number = Some(1);
        assert_eq!(port.port_key(), Some((2, 1)));

        port.port_type_description = Some("MagSafe 3".to_string());
        port.port_number = Some(2);
        assert_eq!(port.port_key(), Some((17, 2)));

        port.port_type_description = None;
        assert_eq!(port.port_key(), None);
    }

    #[test]
    fn test_usb_device_speed_labels() {
        let mut device = create_test_device();

        device.speed_raw = Some(0);
        assert_eq!(device.speed_label(), "Low Speed (1.5 Mbps)");

        device.speed_raw = Some(1);
        assert_eq!(device.speed_label(), "Full Speed (12 Mbps)");

        device.speed_raw = Some(2);
        assert_eq!(device.speed_label(), "High Speed (480 Mbps)");

        device.speed_raw = Some(3);
        assert_eq!(device.speed_label(), "SuperSpeed (5 Gbps)");

        device.speed_raw = Some(4);
        assert_eq!(device.speed_label(), "SuperSpeed+ (10 Gbps)");
    }

    #[test]
    fn test_usb_device_display_name() {
        let mut device = create_test_device();

        device.product_name = Some("Magic Mouse".to_string());
        device.vendor_name = Some("Apple Inc.".to_string());
        assert_eq!(device.display_name(), "Magic Mouse");

        device.product_name = None;
        assert_eq!(device.display_name(), "Apple Inc.");

        device.vendor_name = None;
        device.vendor_id = 0x05AC;
        assert_eq!(device.display_name(), "USB Device (VID: 0x05AC)");
    }

    #[test]
    fn test_power_option_labels() {
        let option = PowerOption {
            voltage_mv: 20000,
            max_current_ma: 3000,
            max_power_mw: 60000,
        };

        assert_eq!(option.volts_label(), "20.0V");
        assert_eq!(option.amps_label(), "3.00A");
        assert_eq!(option.watts_label(), "60W");
    }

    #[test]
    fn test_pd_endpoint_from_str() {
        assert_eq!(PDEndpoint::from_string("SOP"), PDEndpoint::Sop);
        assert_eq!(PDEndpoint::from_string("SOP'"), PDEndpoint::SopPrime);
        assert_eq!(PDEndpoint::from_string("SOP''"), PDEndpoint::SopDoublePrime);
        assert_eq!(PDEndpoint::from_string("invalid"), PDEndpoint::Unknown);
    }

    fn create_test_port() -> USBCPort {
        USBCPort {
            id: 0,
            service_name: "Port-USB-C".to_string(),
            class_name: "AppleHPMInterfaceType12".to_string(),
            port_description: None,
            port_type_description: Some("USB-C".to_string()),
            port_number: Some(1),
            connection_active: Some(false),
            active_cable: Some(false),
            optical_cable: Some(false),
            usb_active: Some(false),
            super_speed_active: Some(false),
            usb_mode_type: Some(0),
            usb_connect_string: None,
            transports_supported: vec![],
            transports_active: vec![],
            transports_provisioned: vec![],
            plug_orientation: Some(0),
            plug_event_count: Some(0),
            connection_count: Some(0),
            overcurrent_count: Some(0),
            bus_index: None,
        }
    }

    fn create_test_device() -> USBDevice {
        USBDevice {
            id: 0,
            location_id: 0,
            vendor_id: 0,
            product_id: 0,
            vendor_name: None,
            product_name: None,
            serial_number: None,
            usb_version: None,
            speed_raw: Some(1),
            bus_power_ma: None,
            current_ma: None,
            bus_index: None,
        }
    }
}
