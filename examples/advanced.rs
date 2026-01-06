use arinc429::{Label, decode, encode};

fn main() {
    // Ground Speed: 250 knots
    let label_gs = Label::from_octal_str("012").unwrap();
    let gs_raw = 2000u32;
    let encoded_gs = encode(label_gs.raw(), 0, gs_raw, 3).unwrap();
    println!("Encoded GS word (hex): {encoded_gs:08X}");

    let decoded_gs = decode(encoded_gs).unwrap();
    println!(
        "Decoded: label {:?} ({}) [{}] — SSM: {}",
        decoded_gs.label,
        decoded_gs.label.name(),
        decoded_gs.label.octal(),
        decoded_gs.ssm.name()
    );
    if let Some(value) = decoded_gs.to_physical() {
        let unit = decoded_gs.label.units();
        let unit_str = if unit.is_empty() { "" } else { unit };
        println!("Physical value: {:.1} {}", value, unit_str);
    }

    // Mach: 2.500
    let label_mach = Label::from_octal_str("205").unwrap();
    let mach_raw = 2500u32;
    let encoded_mach = encode(label_mach.raw(), 0, mach_raw, 3).unwrap();
    let decoded_mach = decode(encoded_mach).unwrap();
    if let Some(mach) = decoded_mach.to_physical() {
        println!("Mach: {:.3}", mach);
    }

    // TAT: -50.0 °C
    let tat_celsius = -50.0_f64;
    let tat_raw_signed = (tat_celsius / 0.25) as i32;
    let tat_raw = if tat_raw_signed < 0 {
        ((tat_raw_signed + 0x80000) as u32) & 0x7FFFF
    } else {
        tat_raw_signed as u32
    };
    let label_tat = Label::from_octal_str("211").unwrap();
    let encoded_tat = encode(label_tat.raw(), 0, tat_raw, 3).unwrap();
    let decoded_tat = decode(encoded_tat).unwrap();
    if let Some(tat) = decoded_tat.to_physical() {
        println!("TAT: {:.2} °C", tat);
    }

    // Roll Angle: +45.0 °
    let roll_degrees = 45.0_f64;
    let roll_raw = (roll_degrees / 0.01) as u32;
    let label_roll = Label::from_octal_str("324").unwrap();
    let encoded_roll = encode(label_roll.raw(), 0, roll_raw, 3).unwrap();
    let decoded_roll = decode(encoded_roll).unwrap();
    if let Some(roll) = decoded_roll.to_physical() {
        println!("Roll Angle: {:.2} °", roll);
    }

    // Date: 06-01-26
    let date_data = (0b00 << 17) |  // day tens
        (0b0110 << 13) | // day units = 6
        (0b0 << 12) |    // month tens = 0
        (0b0001 << 8) |  // month units = 1
        (0b0010 << 4) |  // year tens = 2
        0b0110; // year units = 6

    let label_date = Label::from_octal_str("260").unwrap();
    let encoded_date = encode(label_date.raw(), 0, date_data, 3).unwrap();
    let decoded_date = decode(encoded_date).unwrap();
    if let Some(date) = decoded_date.to_bcd_date() {
        println!("Date: {}", date);
    }

    // UTC Time: 12:34:56 — NOW FIXED (bits within 0..=18)
    // hour: 12 → tens=1 (bit 20-21 → but data field is bits 11-29 → index 0-18)
    // Correct bit positions (0 = LSB = bit 11 of word):
    // sec units:  bits 0-3
    // sec tens:   bits 4-6
    // min units:  bits 7-10
    // min tens:   bits 11-13
    // hour units: bits 14-17
    // hour tens:  bits 18-19 (max!)
    let time_data = (0b01 << 18) |   // hour tens = 1
        (0b0010 << 14) | // hour units = 2
        (0b011 << 11) |  // min tens = 3
        (0b0100 << 7) |  // min units = 4
        (0b101 << 4) |   // sec tens = 5
        0b0110; // sec units = 6

    let label_time = Label::from_octal_str("150").unwrap();
    let encoded_time = encode(label_time.raw(), 0, time_data, 3).unwrap();
    let decoded_time = decode(encoded_time).unwrap();
    if let Some(time) = decoded_time.to_bcd_time() {
        println!("UTC Time: {}", time);
    }

    // Failure SSM example
    let encoded_fail = encode(label_gs.raw(), 0, gs_raw, 0).unwrap(); // SSM=0
    let decoded_fail = decode(encoded_fail).unwrap();
    println!("SSM (Failure case): {}", decoded_fail.ssm.name());
    println!("Physical value: {:?}", decoded_fail.to_physical());
}
