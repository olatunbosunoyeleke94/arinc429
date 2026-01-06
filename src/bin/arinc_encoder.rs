// src/bin/arinc_encoder.rs

use std::collections::HashMap;
use std::io::{self, Read};
use serde::{Deserialize, Serialize};
use arinc429::{encode, Label};

#[derive(Deserialize)]
struct Input {
    #[serde(flatten)]
    labels: HashMap<String, i64>,
}

#[derive(Serialize)]
struct Output {
    #[serde(flatten)]
    words: HashMap<String, String>,
}

fn main() -> io::Result<()> {
    let mut input_str = String::new();
    io::stdin().read_to_string(&mut input_str)?;

    let input: Input = serde_json::from_str(&input_str).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, e)
    })?;

    let mut words = HashMap::new();

    for (name, raw_signed) in input.labels {
        let (label_u8, raw_u32) = match name.as_str() {
            "GroundSpeed" => (Label::GroundSpeed.raw(), raw_signed as u32),
            "PressureAltitude" | "BaroCorrectedAlt" => (Label::PressureAltitude.raw(), raw_signed as u32), // positive for now
            "Mach" => (Label::Mach.raw(), raw_signed as u32),
            "TrueAirspeed" => (Label::TrueAirspeed.raw(), raw_signed as u32),
            "Tat" | "RollAngle" => {
                let signed = raw_signed as i32;
                let u32_val = if signed < 0 {
                    ((signed as i64 + 0x80000) as u32) & 0x7FFFF
                } else {
                    signed as u32
                };
                let label = if name == "Tat" { Label::Tat.raw() } else { Label::RollAngle.raw() };
                (label, u32_val)
            }
            _ => continue,
        };

        match encode(label_u8, 0, raw_u32, 3) {  // SDI=0, SSM=3 Normal
            Ok(word) => {
                words.insert(name, format!("{:08X}", word));
            }
            Err(e) => {
                eprintln!("Encode error for {}: {}", name, e);
            }
        }
    }

    let output = Output { words };
    println!("{}", serde_json::to_string(&output).unwrap());

    Ok(())
}
