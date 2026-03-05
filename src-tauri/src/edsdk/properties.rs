// EDSDK property value → human-readable label lookup tables

/// Aperture (Av) value → f-stop label
pub fn av_label(value: u32) -> String {
    match value {
        0x00 => "Auto".into(),
        0x08 => "f/1".into(),
        0x0B => "f/1.1".into(),
        0x0C => "f/1.2".into(),
        0x10 => "f/1.4".into(),
        0x13 => "f/1.6".into(),
        0x14 => "f/1.8".into(),
        0x18 => "f/2".into(),
        0x1B => "f/2.2".into(),
        0x1C => "f/2.5".into(),
        0x20 => "f/2.8".into(),
        0x23 => "f/3.2".into(),
        0x24 => "f/3.5".into(),
        0x28 => "f/4".into(),
        0x2B => "f/4.5".into(),
        0x2D => "f/5".into(),
        0x30 => "f/5.6".into(),
        0x33 => "f/6.3".into(),
        0x35 => "f/7.1".into(),
        0x38 => "f/8".into(),
        0x3B => "f/9".into(),
        0x3D => "f/10".into(),
        0x40 => "f/11".into(),
        0x43 => "f/13".into(),
        0x45 => "f/14".into(),
        0x48 => "f/16".into(),
        0x4B => "f/18".into(),
        0x4D => "f/20".into(),
        0x50 => "f/22".into(),
        0x53 => "f/25".into(),
        0x55 => "f/27".into(),
        0x58 => "f/32".into(),
        0xFFFFFFFF => "Unknown".into(),
        v => format!("0x{v:02X}"),
    }
}

/// All valid Av values (for building a selector list on the frontend)
pub fn av_options() -> Vec<(u32, String)> {
    [
        0x00u32, 0x08, 0x0B, 0x0C, 0x10, 0x13, 0x14, 0x18, 0x1B, 0x1C, 0x20, 0x23, 0x24, 0x28,
        0x2B, 0x2D, 0x30, 0x33, 0x35, 0x38, 0x3B, 0x3D, 0x40, 0x43, 0x45, 0x48, 0x4B, 0x4D,
        0x50, 0x53, 0x55, 0x58,
    ]
    .iter()
    .map(|&v| (v, av_label(v)))
    .collect()
}

/// Shutter speed (Tv) value → label
pub fn tv_label(value: u32) -> String {
    match value {
        0x0C => "Bulb".into(),
        0x10 => "30\"".into(),
        0x13 => "25\"".into(),
        0x14 => "20\"".into(),
        0x15 => "20\" (1/3)".into(),
        0x18 => "15\"".into(),
        0x1B => "13\"".into(),
        0x1C => "10\"".into(),
        0x1D => "10\" (1/3)".into(),
        0x20 => "8\"".into(),
        0x23 => "6\" (1/3)".into(),
        0x24 => "6\"".into(),
        0x25 => "5\"".into(),
        0x28 => "4\"".into(),
        0x2B => "3.2\"".into(),
        0x2C => "3\"".into(),
        0x2D => "2.5\"".into(),
        0x30 => "2\"".into(),
        0x33 => "1.6\"".into(),
        0x34 => "1.5\"".into(),
        0x35 => "1.3\"".into(),
        0x38 => "1\"".into(),
        0x3B => "0.8\"".into(),
        0x3C => "0.7\"".into(),
        0x3D => "0.6\"".into(),
        0x40 => "1/2".into(),
        0x43 => "1/2.5".into(),
        0x44 => "1/3".into(),
        0x45 => "1/3.2".into(),
        0x48 => "1/4".into(),
        0x4B => "1/5".into(),
        0x4C => "1/6".into(),
        0x4D => "1/6.3".into(),
        0x50 => "1/8".into(),
        0x53 => "1/10 (1/3)".into(),
        0x54 => "1/10".into(),
        0x55 => "1/12.5".into(),
        0x58 => "1/15".into(),
        0x5B => "1/20 (1/3)".into(),
        0x5C => "1/20".into(),
        0x5D => "1/25".into(),
        0x60 => "1/30".into(),
        0x63 => "1/40".into(),
        0x64 => "1/45".into(),
        0x65 => "1/50".into(),
        0x68 => "1/60".into(),
        0x6B => "1/80".into(),
        0x6C => "1/90".into(),
        0x6D => "1/100".into(),
        0x70 => "1/125".into(),
        0x73 => "1/160".into(),
        0x74 => "1/180".into(),
        0x75 => "1/200".into(),
        0x78 => "1/250".into(),
        0x7B => "1/320".into(),
        0x7C => "1/350".into(),
        0x7D => "1/400".into(),
        0x80 => "1/500".into(),
        0x83 => "1/640".into(),
        0x84 => "1/750".into(),
        0x85 => "1/800".into(),
        0x88 => "1/1000".into(),
        0x8B => "1/1250".into(),
        0x8C => "1/1500".into(),
        0x8D => "1/1600".into(),
        0x90 => "1/2000".into(),
        0x93 => "1/2500".into(),
        0x94 => "1/3000".into(),
        0x95 => "1/3200".into(),
        0x98 => "1/4000".into(),
        0x9B => "1/5000".into(),
        0x9C => "1/6000".into(),
        0x9D => "1/6400".into(),
        0xA0 => "1/8000".into(),
        0xFFFFFFFF => "Unknown".into(),
        v => format!("0x{v:02X}"),
    }
}

pub fn tv_options() -> Vec<(u32, String)> {
    [
        0x10u32, 0x13, 0x14, 0x18, 0x1C, 0x20, 0x24, 0x28, 0x2C, 0x30, 0x34, 0x38, 0x3C, 0x40,
        0x44, 0x48, 0x4C, 0x50, 0x54, 0x58, 0x5C, 0x60, 0x64, 0x68, 0x6C, 0x70, 0x74, 0x78,
        0x7C, 0x80, 0x84, 0x88, 0x8C, 0x90, 0x94, 0x98, 0x9C, 0xA0,
    ]
    .iter()
    .map(|&v| (v, tv_label(v)))
    .collect()
}

/// ISO value → label
pub fn iso_label(value: u32) -> String {
    match value {
        0x00000000 => "Auto".into(),
        0x00000028 => "ISO 6".into(),
        0x00000030 => "ISO 12".into(),
        0x00000038 => "ISO 25".into(),
        0x00000040 => "ISO 50".into(),
        0x00000048 => "ISO 100".into(),
        0x0000004B => "ISO 125".into(),
        0x0000004D => "ISO 160".into(),
        0x00000050 => "ISO 200".into(),
        0x00000053 => "ISO 250".into(),
        0x00000055 => "ISO 320".into(),
        0x00000058 => "ISO 400".into(),
        0x0000005B => "ISO 500".into(),
        0x0000005D => "ISO 640".into(),
        0x00000060 => "ISO 800".into(),
        0x00000063 => "ISO 1000".into(),
        0x00000065 => "ISO 1250".into(),
        0x00000068 => "ISO 1600".into(),
        0x0000006B => "ISO 2000".into(),
        0x0000006D => "ISO 2500".into(),
        0x00000070 => "ISO 3200".into(),
        0x00000073 => "ISO 4000".into(),
        0x00000075 => "ISO 5000".into(),
        0x00000078 => "ISO 6400".into(),
        0x0000007B => "ISO 8000".into(),
        0x0000007D => "ISO 10000".into(),
        0x00000080 => "ISO 12800".into(),
        0x00000083 => "ISO 16000".into(),
        0x00000085 => "ISO 20000".into(),
        0x00000088 => "ISO 25600".into(),
        0x00000090 => "ISO 51200".into(),
        0x00000098 => "ISO 102400".into(),
        0xFFFFFFFF => "Unknown".into(),
        v => format!("0x{v:04X}"),
    }
}

pub fn iso_options() -> Vec<(u32, String)> {
    [
        0x00000000u32,
        0x00000048,
        0x0000004B,
        0x0000004D,
        0x00000050,
        0x00000053,
        0x00000055,
        0x00000058,
        0x0000005B,
        0x0000005D,
        0x00000060,
        0x00000063,
        0x00000065,
        0x00000068,
        0x0000006B,
        0x0000006D,
        0x00000070,
        0x00000073,
        0x00000075,
        0x00000078,
        0x0000007B,
        0x0000007D,
        0x00000080,
        0x00000088,
        0x00000090,
        0x00000098,
    ]
    .iter()
    .map(|&v| (v, iso_label(v)))
    .collect()
}

/// White Balance value → label
pub fn wb_label(value: i32) -> String {
    match value {
        -1 => "Auto (ambience)".into(),
        0 => "Auto".into(),
        1 => "Daylight".into(),
        2 => "Cloudy".into(),
        3 => "Tungsten".into(),
        4 => "Fluorescent".into(),
        5 => "Flash".into(),
        6 => "Manual (Custom)".into(),
        8 => "Shade".into(),
        9 => "Color Temp.".into(),
        10 => "Custom WB 2".into(),
        11 => "Custom WB 3".into(),
        12 => "Custom WB 4".into(),
        23 => "Auto (white priority)".into(),
        v => format!("{v}"),
    }
}

pub fn wb_options() -> Vec<(i32, String)> {
    [0i32, 1, 2, 3, 4, 5, 6, 8, 9, 23]
        .iter()
        .map(|&v| (v, wb_label(v)))
        .collect()
}
