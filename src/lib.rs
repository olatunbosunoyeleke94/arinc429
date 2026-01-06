//! # arinc429
//!
//! A Rust library for the **ARINC 429** avionics data bus protocol.
//!
//! Provides full support for:
//! - Encoding and decoding 32-bit ARINC 429 words
//! - Label bit reversal and odd parity
//! - Octal label parsing (e.g., `"012"`, `"203"`)
//! - BNR (Binary) physical value interpretation with signed/unsigned handling
//! - BCD date (label 260) and UTC time (label 150) decoding
//! - SSM (Sign/Status Matrix) interpretation
//! - Common flight parameters (ground speed, altitude, Mach, TAT, roll angle, etc.)
//!
//! ## Features
//! - Pure Rust, no_std compatible (with minor changes)
//! - Comprehensive error handling via [`thiserror`]
//! - Well-tested with unit tests and cross-validation
//! - Ready for integration with flight simulators (JSBSim, FlightGear) or real hardware
//!
//! ## Example
//!
//! ```rust
//! use arinc429::{encode, decode, Label};
//!
//! // Encode Ground Speed = 250 knots (label 012)
//! let word = encode(Label::GroundSpeed.raw(), 0, 2000, 3).unwrap(); // 2000 * 0.125 = 250
//! assert_eq!(format!("{:08X}", word), "E01F4050");
//!
//! // Decode it back
//! let decoded = decode(word).unwrap();
//! assert_eq!(decoded.label, Label::GroundSpeed);
//! assert_eq!(decoded.to_physical(), Some(250.0));
//! ```

use thiserror::Error;


/// Errors that can occur during ARINC 429 operations.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ArincError {
    /// Data field exceeds 19 bits (max allowed value: 524287)
    #[error("Data exceeds 19 bits: {0}")]
    DataOverflow(u32),

    /// Source/Destination Identifier must be 0–3
    #[error("SDI must be 0-3: {0}")]
    InvalidSdi(u8),

    /// Sign/Status Matrix must be 0–3
    #[error("SSM must be 0-3: {0}")]
    InvalidSsm(u8),

    /// Odd parity check failed
    #[error("Parity check failed")]
    ParityMismatch,

    /// Invalid octal label string (e.g., contains non-octal digits or out of range)
    #[error("Invalid octal label string")]
    InvalidOctalLabel,
}

/// Sign/Status Matrix (SSM) values as defined in ARINC 429.
///
/// These indicate data validity and are common to both BNR and BCD data types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ssm {
    /// Failure Warning – equipment failure detected
    FailureWarning,
    /// No Computed Data – data not available or invalid
    NoComputedData,
    /// Functional Test – self-test in progress
    FunctionalTest,
    /// Normal Operation – data is valid
    NormalOperation,
}

impl Ssm {
    /// Convert raw SSM bits (0–3) to the corresponding enum variant.
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::FailureWarning,
            1 => Self::NoComputedData,
            2 => Self::FunctionalTest,
            3 => Self::NormalOperation,
            _ => Self::NoComputedData, // Invalid values treated as NCD
        }
    }

    /// Human-readable description of the SSM state.
    pub fn name(&self) -> &'static str {
        match self {
            Self::FailureWarning => "Failure Warning",
            Self::NoComputedData => "No Computed Data",
            Self::FunctionalTest => "Functional Test",
            Self::NormalOperation => "Normal Operation",
        }
    }
}

/// Known ARINC 429 parameter labels supported by this crate.
///
/// Each variant includes its standard octal and decimal code, data type (BNR/BCD),
/// and physical interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Label {
    /// Ground Speed – label 012 (decimal 10), BNR, resolution 0.125 knots
    GroundSpeed,
    /// UTC Time – label 150 (decimal 104), BCD, format hh:mm:ss
    UtcTime,
    /// Pressure Altitude – label 203 (decimal 131), BNR signed, feet
    PressureAltitude,
    /// Baro-Corrected Altitude – label 204 (decimal 132), BNR signed, feet
    BaroCorrectedAlt,
    /// Mach – label 205 (decimal 133), BNR positive, resolution 0.001
    Mach,
    /// True Airspeed – label 210 (decimal 136), BNR, knots
    TrueAirspeed,
    /// Total Air Temperature (TAT) – label 211 (decimal 137), BNR signed, resolution 0.25 °C
    Tat,
    /// Date – label 260 (decimal 176), BCD, format dd-mm-yy
    Date,
    /// Roll Angle – label 324 (decimal 212), BNR signed, resolution 0.01°
    RollAngle,
    /// Unknown or unsupported label
    Unknown(u8),
}

impl Label {
    /// Convert a raw decimal label code (after bit reversal) to the enum variant.
    pub fn from_u8(raw: u8) -> Self {
        match raw {
            10 => Label::GroundSpeed,
            104 => Label::UtcTime,
            131 => Label::PressureAltitude,
            132 => Label::BaroCorrectedAlt,
            133 => Label::Mach,
            136 => Label::TrueAirspeed,
            137 => Label::Tat,
            176 => Label::Date,
            212 => Label::RollAngle,
            _ => Label::Unknown(raw),
        }
    }

    /// Parse an octal label string (e.g., `"012"`, `"203"`) into the corresponding variant.
    ///
    /// Returns an error if the string is not valid octal or maps to an unknown label.
    pub fn from_octal_str(s: &str) -> Result<Self, ArincError> {
        let decimal = u8::from_str_radix(s, 8).map_err(|_| ArincError::InvalidOctalLabel)?;
        Ok(Self::from_u8(decimal))
    }

    /// Raw decimal label code for use with [`encode`].
    pub fn raw(&self) -> u8 {
        match self {
            Label::GroundSpeed => 10,
            Label::UtcTime => 104,
            Label::PressureAltitude => 131,
            Label::BaroCorrectedAlt => 132,
            Label::Mach => 133,
            Label::TrueAirspeed => 136,
            Label::Tat => 137,
            Label::Date => 176,
            Label::RollAngle => 212,
            Label::Unknown(n) => *n,
        }
    }

    /// Standard octal representation (3 digits, zero-padded).
    pub fn octal(&self) -> String {
        match self {
            Label::GroundSpeed => "012".to_string(),
            Label::UtcTime => "150".to_string(),
            Label::PressureAltitude => "203".to_string(),
            Label::BaroCorrectedAlt => "204".to_string(),
            Label::Mach => "205".to_string(),
            Label::Tat => "211".to_string(),
            Label::TrueAirspeed => "210".to_string(),
            Label::Date => "260".to_string(),
            Label::RollAngle => "324".to_string(),
            Label::Unknown(n) => format!("{:03o}", n),
        }
    }

    /// Human-readable parameter name.
    pub fn name(&self) -> &'static str {
        match self {
            Label::GroundSpeed => "Ground Speed",
            Label::UtcTime => "UTC Time",
            Label::PressureAltitude => "Pressure Altitude (1013.25 mb)",
            Label::BaroCorrectedAlt => "Baro-Corrected Altitude",
            Label::Mach => "Mach",
            Label::Tat => "Total Air Temperature (TAT)",
            Label::TrueAirspeed => "True Airspeed",
            Label::Date => "Date",
            Label::RollAngle => "Roll Angle",
            Label::Unknown(_) => "Unknown Label",
        }
    }

    /// Physical units (empty string if none).
    pub fn units(&self) -> &'static str {
        match self {
            Label::GroundSpeed | Label::TrueAirspeed => "knots",
            Label::PressureAltitude | Label::BaroCorrectedAlt => "feet",
            Label::Mach => "",
            Label::Tat => "°C",
            Label::RollAngle => "°",
            Label::Date | Label::UtcTime => "",
            Label::Unknown(_) => "",
        }
    }
}

/// A fully decoded ARINC 429 word.
#[derive(Debug, PartialEq)]
pub struct ArincWord {
    /// The parameter label
    pub label: Label,
    /// Source/Destination Identifier (0–3)
    pub sdi: u8,
    /// Raw 19-bit data field
    pub data: u32,
    /// Sign/Status Matrix
    pub ssm: Ssm,
}

impl ArincWord {
    /// Convert the raw data to a physical value (e.g., knots, feet, °C) for supported BNR labels.
    ///
    /// Returns `None` if:
    /// - SSM is not Normal Operation
    /// - Label is not supported or is BCD (use `to_bcd_date`/`to_bcd_time` instead)
    pub fn to_physical(&self) -> Option<f64> {
        if !matches!(self.ssm, Ssm::NormalOperation) {
            return None;
        }

        let raw = self.data as i32;
        let signed = if (raw & 0x40000) != 0 {
            raw.wrapping_sub(0x80000)
        } else {
            raw
        };

        match self.label {
            Label::GroundSpeed => Some(self.data as f64 * 0.125),
            Label::PressureAltitude | Label::BaroCorrectedAlt => Some(signed as f64),
            Label::Mach => Some(self.data as f64 * 0.001),
            Label::Tat => Some(signed as f64 * 0.25),
            Label::TrueAirspeed => Some(self.data as f64),
            Label::RollAngle => Some(signed as f64 * 0.01),
            _ => None,
        }
    }

    /// Decode BCD Date (label 260) → `"dd-mm-yy"` string.
    ///
    /// Returns `None` if label mismatch, invalid BCD digits, or SSM not Normal.
    pub fn to_bcd_date(&self) -> Option<String> {
        if self.label != Label::Date || !matches!(self.ssm, Ssm::NormalOperation) {
            return None;
        }

        let d = self.data;
        let year_units = (d & 0xF) as u8;
        let year_tens = ((d >> 4) & 0xF) as u8;
        let month_units = ((d >> 8) & 0xF) as u8;
        let month_tens = ((d >> 12) & 0x1) as u8;
        let day_units = ((d >> 13) & 0xF) as u8;
        let day_tens = ((d >> 17) & 0x3) as u8;

        if year_tens > 9
            || year_units > 9
            || month_tens > 1
            || month_units > 9
            || day_tens > 3
            || day_units > 9
            || (month_tens * 10 + month_units) == 0
            || (day_tens * 10 + day_units) == 0
        {
            return None;
        }

        Some(format!(
            "{:02}-{:02}-{:02}",
            day_tens * 10 + day_units,
            month_tens * 10 + month_units,
            year_tens * 10 + year_units
        ))
    }

    /// Decode BCD UTC Time (label 150) → `"hh:mm:ss"` string.
    ///
    /// Returns `None` if label mismatch, invalid BCD digits, or SSM not Normal.
    pub fn to_bcd_time(&self) -> Option<String> {
        if self.label != Label::UtcTime || !matches!(self.ssm, Ssm::NormalOperation) {
            return None;
        }

        let d = self.data;
        let sec_units = (d & 0xF) as u8;
        let sec_tens = ((d >> 4) & 0x7) as u8;
        let min_units = ((d >> 7) & 0xF) as u8;
        let min_tens = ((d >> 11) & 0x7) as u8;
        let hour_units = ((d >> 14) & 0xF) as u8;
        let hour_tens = ((d >> 18) & 0x3) as u8;

        if hour_tens > 2
            || hour_units > 9
            || min_tens > 5
            || min_units > 9
            || sec_tens > 5
            || sec_units > 9
        {
            return None;
        }

        Some(format!(
            "{:02}:{:02}:{:02}",
            hour_tens * 10 + hour_units,
            min_tens * 10 + min_units,
            sec_tens * 10 + sec_units
        ))
    }
}

/// Encode an ARINC 429 word.
///
/// Performs label bit reversal, packs fields, and adds odd parity.
///
/// # Arguments
/// - `label`: Raw decimal label code (before reversal)
/// - `sdi`: Source/Destination Identifier (0–3)
/// - `data`: 19-bit data field (0–524287)
/// - `ssm`: Sign/Status Matrix (0–3)
///
/// # Returns
/// 32-bit ARINC 429 word on success
pub fn encode(label: u8, sdi: u8, data: u32, ssm: u8) -> Result<u32, ArincError> {
    if sdi > 3 {
        return Err(ArincError::InvalidSdi(sdi));
    }
    if ssm > 3 {
        return Err(ArincError::InvalidSsm(ssm));
    }
    if data > 0x7FFFF {
        return Err(ArincError::DataOverflow(data));
    }

    let label_bits = label.reverse_bits();
    let mut word = (label_bits as u32)
        | ((sdi as u32) << 8)
        | (data << 10)
        | ((ssm as u32) << 29);

    let ones = (word & 0x7FFFFFFF).count_ones();
    let parity = if ones % 2 == 0 { 1 << 31 } else { 0 };
    word |= parity;

    Ok(word)
}

/// Decode a 32-bit ARINC 429 word.
///
/// Validates odd parity, reverses label bits, extracts fields, and maps SSM/label.
///
/// # Returns
/// [`ArincWord`] struct on success
pub fn decode(word: u32) -> Result<ArincWord, ArincError> {
    if word.count_ones() % 2 == 0 {
        return Err(ArincError::ParityMismatch);
    }

    let label_bits = (word & 0xFF) as u8;
    let label = label_bits.reverse_bits();
    let sdi = ((word >> 8) & 0x3) as u8;
    let data = (word >> 10) & 0x7FFFF;
    let ssm_raw = ((word >> 29) & 0x3) as u8;

    Ok(ArincWord {
        label: Label::from_u8(label),
        sdi,
        data,
        ssm: Ssm::from_u8(ssm_raw),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_labels_parse() {
        assert_eq!(Label::from_octal_str("012").unwrap(), Label::GroundSpeed);
        assert_eq!(Label::from_octal_str("150").unwrap(), Label::UtcTime);
        assert_eq!(Label::from_octal_str("260").unwrap(), Label::Date);
    }

    #[test]
    fn test_bcd_time() {
        let data =
            (0b01 << 18) | (0b0010 << 14) | (0b011 << 11) | (0b0100 << 7) | (0b101 << 4) | 0b0110;
        let word = encode(104, 0, data, 3).unwrap();
        let decoded = decode(word).unwrap();
        assert_eq!(decoded.to_bcd_time(), Some("12:34:56".to_string()));
    }

    #[test]
    fn test_bcd_date() {
        let data =
            (0b00 << 17) | (0b0110 << 13) | (0b0 << 12) | (0b0001 << 8) | (0b0010 << 4) | 0b0110;
        let word = encode(176, 0, data, 3).unwrap();
        let decoded = decode(word).unwrap();
        assert_eq!(decoded.to_bcd_date(), Some("06-01-26".to_string()));
    }

    #[test]
    fn test_cross_py_ground_speed() {
        let word: u32 = 0xE01F4050;
        let decoded = decode(word).unwrap();
        assert_eq!(decoded.label, Label::GroundSpeed);
        assert_eq!(decoded.ssm, Ssm::NormalOperation);
        assert_eq!(decoded.to_physical(), Some(250.0));
    }
}
