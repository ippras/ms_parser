use super::Parse;
use crate::{error::Result, utils::Preview};
use nom::{
    multi::count,
    number::complete::{be_i16, be_i32, be_u16},
    sequence::pair,
};
use std::fmt::{self, Display, Formatter};

const MASS_TO_CHARGE_FACTOR: f32 = 20.0;

/// Abundance is in special packed format.
///
/// Packed abundance stored as 2-bit scale (0..=3; power of 8; x1, x8, x64,
/// x512) with 14 bit mantissa (0..=16383).
fn unpack(packed: u16) -> u32 {
    // The abundance scale is stored in the first 2 bits.
    let scale = packed >> 14;
    // Mask off the first 2 bits.
    let mantissa = packed & 0b0011_1111_1111_1111;
    // Scale the abundance by powers of 8.
    mantissa as u32 * 8u32.pow(scale as _)
}

/// Spectral record  
/// Each spectral record is composed of the following entries, followed by a
/// list of mass and abundance values.
#[derive(Debug, Default)]
pub struct Spectral {
    /// Retention time ms
    pub(crate) retention_time: i32,
    /// Peak with max abundance from the peaks
    pub(crate) _base_peak: Peak,
    /// Peaks for the retention time
    pub(crate) peaks: Vec<Peak>,
}

impl Spectral {
    pub fn retention_time(&self) -> i32 {
        self.retention_time
    }

    pub fn peaks(&self) -> &[Peak] {
        &self.peaks
    }
}

impl Spectral {
    pub fn parse(input: &[u8]) -> Result<(&[u8], Self)> {
        assert!(input.len() > 18);
        let (input, _number_of_words) = be_i16(input)?;
        let (input, retention_time) = be_i32(input)?;
        let (input, _number_of_words_less3) = be_i16(input)?;
        let (input, _data_type) = be_i16(input)?;
        let (input, _status_word) = be_i16(input)?;
        // Peaks
        let (input, number_of_peaks) = be_i16(input)?;
        let (input, _base_peak) = Peak::parse(input)?;
        assert!(input.len() > 4 * number_of_peaks as usize);
        let (input, mut peaks) = count(Peak::parse, number_of_peaks as _)(input)?;
        peaks.reverse();
        Ok((
            input,
            Self {
                retention_time,
                _base_peak,
                peaks,
            },
        ))
    }
}

impl Display for Spectral {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Spectral")
            .field("retention_time", &self.retention_time)
            .field("peaks", &format_args!("{}", Preview(&self.peaks)))
            .finish()
    }
}

// &self
// .peaks
// .first()
// .into_iter()
// .chain(self.peaks.last())
// .map(|peak| format!("{peak:?}"))
// .join(", ..., "),

/// `Mass to charge` `abundance` pair  
/// The data is stored in the Data.ms file as `mass` and `abundance` pairs.
/// `Mass` is represented by u16 and stores the mass value times 20. `Intensity`
/// is represented by a packed i16 value.
#[derive(Clone, Copy, Debug, Default)]
pub struct Peak {
    pub(crate) mass_to_charge: u16,
    pub(crate) abundance: u16,
}

impl Peak {
    pub fn mass_to_charge(&self) -> f32 {
        self.mass_to_charge as f32 / MASS_TO_CHARGE_FACTOR
    }

    /// Packed signal
    pub fn packed_signal(&self) -> u16 {
        self.abundance
    }

    /// Unpacked signal
    pub fn unpacked_signal(&self) -> u32 {
        unpack(self.abundance)
    }
}

impl Parse for Peak {
    fn parse(input: &[u8]) -> Result<(&[u8], Self)> {
        let (input, (mass_to_charge, abundance)) = pair(be_u16, be_u16)(input)?;
        Ok((
            input,
            Self {
                mass_to_charge,
                abundance,
                // mass_to_charge: mass_to_charge as f32 / MASS_TO_CHARGE_FACTOR,
                // abundance: abundance as _,
                // TODO: OpenChrome does not unpack abundance
                // abundance: unpack(abundance as _) as _,
            },
        ))
    }
}

impl Display for Peak {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Peak")
            .field("mass_to_charge", &self.mass_to_charge())
            .field("abundance", &self.abundance)
            .finish()
    }
}
