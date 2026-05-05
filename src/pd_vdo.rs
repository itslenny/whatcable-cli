/// USB Power Delivery 3.0 / 3.1 VDO decoders.
/// Only parses fields we surface to the user.
/// Refer to USB-PD spec (Universal Serial Bus Power Delivery Specification, Revision 3.1)
/// for full layout.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductType {
    Undefined,
    PdusbHub,
    PdusbPeripheral,
    PassiveCable,
    ActiveCable,
    Ama, // Alternate Mode Adapter
    Vpd, // VCONN-Powered Device
    Other,
}

impl ProductType {
    pub fn from_raw(value: u32) -> Self {
        match value {
            0 => ProductType::Undefined,
            1 => ProductType::PdusbHub,
            2 => ProductType::PdusbPeripheral,
            3 => ProductType::PassiveCable,
            4 => ProductType::ActiveCable,
            5 => ProductType::Ama,
            6 => ProductType::Vpd,
            7 => ProductType::Other,
            _ => ProductType::Undefined,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ProductType::Undefined => "Unspecified",
            ProductType::PdusbHub => "USB Hub",
            ProductType::PdusbPeripheral => "USB Peripheral",
            ProductType::PassiveCable => "Passive cable",
            ProductType::ActiveCable => "Active cable",
            ProductType::Ama => "Alternate Mode Adapter",
            ProductType::Vpd => "VCONN-powered device",
            ProductType::Other => "Other",
        }
    }
}

#[derive(Debug, Clone)]
pub struct IDHeader {
    pub usb_comm_host: bool,
    pub usb_comm_device: bool,
    pub modal_operation: bool,
    pub ufp_product_type: ProductType,
    pub dfp_product_type: ProductType,
}

pub fn decode_id_header(vdo: u32) -> IDHeader {
    IDHeader {
        usb_comm_host: (vdo >> 31) & 1 == 1,
        usb_comm_device: (vdo >> 30) & 1 == 1,
        modal_operation: (vdo >> 26) & 1 == 1,
        ufp_product_type: ProductType::from_raw((vdo >> 27) & 0b111),
        dfp_product_type: ProductType::from_raw((vdo >> 23) & 0b111),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CableSpeed {
    Usb20,
    Usb32Gen1, // 5 Gbps
    Usb32Gen2, // 10 Gbps
    Usb4Gen3,  // 20 / 40 Gbps
    Usb4Gen4,  // 80 Gbps
}

impl CableSpeed {
    pub fn from_raw(value: u32) -> Self {
        match value {
            0 => CableSpeed::Usb20,
            1 => CableSpeed::Usb32Gen1,
            2 => CableSpeed::Usb32Gen2,
            3 => CableSpeed::Usb4Gen3,
            4 => CableSpeed::Usb4Gen4,
            _ => CableSpeed::Usb20,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            CableSpeed::Usb20 => "USB 2.0 (480 Mbps)",
            CableSpeed::Usb32Gen1 => "USB 3.2 Gen 1 (5 Gbps)",
            CableSpeed::Usb32Gen2 => "USB 3.2 Gen 2 (10 Gbps)",
            CableSpeed::Usb4Gen3 => "USB4 Gen 3 (20 / 40 Gbps)",
            CableSpeed::Usb4Gen4 => "USB4 Gen 4 (80 Gbps)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CableCurrent {
    UsbDefault, // 900 mA / 1.5 A typical USB
    ThreeAmp,
    FiveAmp,
}

impl CableCurrent {
    pub fn from_raw(value: u32) -> Self {
        match value {
            0 => CableCurrent::UsbDefault,
            1 => CableCurrent::ThreeAmp,
            2 => CableCurrent::FiveAmp,
            _ => CableCurrent::UsbDefault,
        }
    }

    pub fn max_amps(&self) -> f64 {
        match self {
            CableCurrent::UsbDefault => 3.0, // be charitable; Type-C default current is 3A on cables
            CableCurrent::ThreeAmp => 3.0,
            CableCurrent::FiveAmp => 5.0,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            CableCurrent::UsbDefault => "USB default",
            CableCurrent::ThreeAmp => "3 A",
            CableCurrent::FiveAmp => "5 A",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CableType {
    Passive,
    Active,
}

#[derive(Debug, Clone)]
pub struct CableVDO {
    pub speed: CableSpeed,
    pub current: CableCurrent,
    pub max_watts: i64,
    pub cable_type: CableType,
    pub vbus_through_cable: bool,
    pub max_voltage_encoded: i64,
}

impl CableVDO {
    pub fn max_volts(&self) -> i64 {
        match self.max_voltage_encoded {
            0 => 20,
            1 => 30,
            2 => 40,
            3 => 50,
            _ => 20,
        }
    }
}

pub fn decode_cable_vdo(vdo: u32, is_active: bool) -> CableVDO {
    let speed_bits = vdo & 0b111;
    let speed = CableSpeed::from_raw(speed_bits);
    let vbus_through = (vdo >> 4) & 1 == 1;
    let current_bits = (vdo >> 5) & 0b11;
    let current = CableCurrent::from_raw(current_bits);
    let max_v = ((vdo >> 9) & 0b11) as i64;
    let cable_type = if is_active {
        CableType::Active
    } else {
        CableType::Passive
    };

    let volts = match max_v {
        1 => 30.0,
        2 => 40.0,
        3 => 50.0,
        _ => 20.0,
    };
    let amps = current.max_amps();
    let watts = (volts * amps).round() as i64;

    CableVDO {
        speed,
        current,
        max_watts: watts,
        cable_type,
        vbus_through_cable: vbus_through,
        max_voltage_encoded: max_v,
    }
}
