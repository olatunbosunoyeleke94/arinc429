// src/bin/test_direct.rs

use arinc429::{encode, Label};

fn main() {
    println!("Testing arinc429 crate directly!\n");

    let tests = [
        ("Ground Speed 250 kts", Label::GroundSpeed.raw(), 2000u32, "012"),
        ("Pressure Altitude 25000 ft", Label::PressureAltitude.raw(), 25000u32, "203"),
        ("Mach 0.80", Label::Mach.raw(), 800u32, "205"),
        ("TAT -50°C", Label::Tat.raw(), {
            let signed = (-50.0 / 0.25) as i32;
            if signed < 0 { (signed + 0x80000) as u32 & 0x7FFFF } else { signed as u32 }
        }, "211"),
        ("Roll Angle +45°", Label::RollAngle.raw(), 4500u32, "324"),
    ];

    for (desc, label, data, octal) in tests {
        match encode(label, 0, data, 3) {  // SDI=0, SSM=Normal
            Ok(word) => {
                println!("{} [{}] → 0x{word:08X}", desc, octal, word = word);
            }
            Err(e) => println!("Error encoding {}: {}", desc, e),
        }
    }
}
