# arinc429

A full-featured Rust library for the ARINC 429 avionics data bus protocol.

## Features

- Full encode/decode with parity validation and label bit reversal
- BNR physical interpretation (altitude, speed, Mach, TAT, roll angle)
- BCD decoding for Date (label 260) and UTC Time (label 150)
- SSM (Sign/Status Matrix) interpretation
- Octal label support (e.g., "012", "203")
- `no_std` ready

## Quick Example

```rust
use arinc429::{encode, decode, Label};

fn main() {
    // Encode Ground Speed = 250 knots (label 012)
    let word = encode(Label::GroundSpeed.raw(), 0, 2000, 3).unwrap(); // SSM = Normal
    println!("Encoded word: 0x{:08X}", word); // → 0xE01F4050

    // Decode it back
    let decoded = decode(word).unwrap();
    println!("Label: {} [{}]", decoded.label.name(), decoded.label.octal());
    println!("Physical: {:.1} {}", decoded.to_physical().unwrap(), decoded.label.units());
}

Output

Encoded word: 0xE01F4050
Label: Ground Speed [012]
Physical: 250.0 knots

## CLI Usage

### Run the advanced example

```bash
cargo run --example advanced     # Full demo with BCD date/time and SSM
cargo run --bin test_direct      # Direct encoding tests
cargo test                       # Run unit tests

## JSBSim Integration (Realistic Flight Simulation)

Generate live ARINC 429 words from real flight physics.

### Setup

```bash
python -m venv venv
source venv/bin/activate          # Windows: venv\Scripts\activate
pip install jsbsim
cargo build --release --bin arinc_encoder

### Run

```bash
python test_jsbsim.py

Prints live ARINC 429 words to console

Automatically creates .csv with full simulation log


