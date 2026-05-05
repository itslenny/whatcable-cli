use once_cell::sync::Lazy;
use std::collections::HashMap;

static VENDOR_NAMES: Lazy<HashMap<i64, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(0x05AC, "Apple");
    m.insert(0x004C, "Apple (legacy)");
    m.insert(0x05E3, "Genesys Logic");
    m.insert(0x0BDA, "Realtek");
    m.insert(0x174C, "ASMedia");
    m.insert(0x2109, "VIA Labs");
    m.insert(0x152D, "JMicron");
    m.insert(0x067B, "Prolific");
    m.insert(0x0451, "Texas Instruments");
    m.insert(0x8087, "Intel");
    m.insert(0x046D, "Logitech");
    m.insert(0x0BB4, "HTC");
    m.insert(0x18D1, "Google");
    m.insert(0x12D1, "Huawei");
    m.insert(0x04E8, "Samsung");
    m.insert(0x2717, "Xiaomi");
    m.insert(0x22D9, "OPPO");
    m.insert(0x2A70, "OnePlus");
    m.insert(0x05C6, "Qualcomm");
    m.insert(0x0BC2, "Seagate");
    m.insert(0x1058, "Western Digital");
    m.insert(0x0781, "SanDisk");
    m.insert(0x0930, "Toshiba");
    m.insert(0x0951, "Kingston");
    m.insert(0x125F, "ADATA");
    m.insert(0x1B1C, "Corsair");
    m.insert(0x154B, "PNY");
    m.insert(0x0080, "Crucial");
    m.insert(0x174F, "Syntek");
    m.insert(0x046E, "Behavior Tech");
    m.insert(0x05DC, "Lexar");
    m.insert(0x0E8D, "MediaTek");
    m.insert(0x148F, "Ralink");
    m.insert(0x0B95, "ASIX");
    m.insert(0x0CF3, "Qualcomm Atheros");
    m.insert(0x06CB, "Synaptics");
    m.insert(0x056A, "Wacom");
    m.insert(0x040A, "Kodak");
    m.insert(0x056D, "EIZO");
    m.insert(0x0AF8, "Belkin");
    m.insert(0x050D, "Belkin (older)");
    m.insert(0x2BCF, "Anker");
    m.insert(0x291A, "Anker (older)");
    m.insert(0x0BB8, "Plantronics / Poly");
    m.insert(0x0763, "M-Audio");
    m.insert(0x0FCE, "Sony Mobile");
    m.insert(0x054C, "Sony");
    m.insert(0x04F2, "Chicony");
    m.insert(0x046A, "Cherry");
    m.insert(0x04D9, "Holtek");
    m.insert(0x1532, "Razer");
    m.insert(0x1B7E, "Holosonics");
    m.insert(0x07AA, "Corega");
    m.insert(0x2188, "SmartAction");
    m.insert(0x0E0F, "VMware");
    m.insert(0x0FFE, "OWC");
    m.insert(0x152E, "Lenovo");
    m.insert(0x17EF, "Lenovo (older)");
    m.insert(0x0BAF, "U.S. Robotics");
    m.insert(0x0DCD, "Diconix");
    m.insert(0x0FCA, "Research In Motion");
    m.insert(0x05E0, "Symbol");
    m.insert(0x05DD, "Delorme");
    m.insert(0x0764, "CyberPower");
    m.insert(0x051D, "American Power Conversion (APC)");
    m.insert(0x2C7C, "Quectel");
    m.insert(0x2341, "Arduino");
    m.insert(0x1A40, "Terminus (hub chips)");
    m.insert(0x32AC, "Apple (Thunderbolt 4)");
    m.insert(0x1D6B, "Linux Foundation");
    m.insert(0x0CF8, "Targus");
    m.insert(0x0B05, "ASUS");
    m.insert(0x103C, "HP");
    m.insert(0x413C, "Dell");
    m.insert(0x0CCD, "TerraTec");
    m.insert(0x0E58, "Aopen");
    m.insert(0x14AD, "Microvision");
    m
});

pub fn name(vendor_id: i64) -> Option<&'static str> {
    VENDOR_NAMES.get(&vendor_id).copied()
}

pub fn label(vendor_id: i64) -> String {
    if let Some(n) = name(vendor_id) {
        format!("{} (0x{:04X})", n, vendor_id)
    } else {
        format!("0x{:04X}", vendor_id)
    }
}
