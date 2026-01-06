use arinc429::{decode, encode};

fn main() {
    let label = 10u8; // Octal 012, e.g., for ground speed
    let sdi = 0u8;
    let data = 25000u32; // Arbitrary data, e.g., knots * scale
    let ssm = 3u8; // Normal operation

    let word = encode(label, sdi, data, ssm).unwrap();
    println!("Encoded word (hex): {word:08X}");

    let decoded = decode(word).unwrap();
    println!("Decoded: {:?}", decoded);
}
