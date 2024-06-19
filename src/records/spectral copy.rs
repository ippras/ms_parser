use super::Parse;
use crate::{
    error::Result,
    utils::{format_slice, Preview},
};
use itertools::Itertools;
use nom::{
    multi::count,
    number::complete::{be_i16, be_i32, be_u16},
    sequence::pair,
};
use std::fmt::{self, Display, Formatter};
use uom::si::{f32::Time, time::millisecond};

const MASS_TO_CHARGE_FACTOR: f32 = 20.0;

/// Abundance is in special packed format.  
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
    pub(crate) retention_time: Time,
    pub(crate) base_peak: Peak,
    pub(crate) peaks: Vec<Peak>,
}

impl Spectral {
    pub fn retention_time(&self) -> Time {
        self.retention_time
    }

    pub fn base_peak(&self) -> Peak {
        self.base_peak
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
        let (input, base_peak) = Peak::parse(input)?;
        assert!(input.len() > 4 * number_of_peaks as usize);
        let (input, mut peaks) = count(Peak::parse, number_of_peaks as _)(input)?;
        peaks.reverse();
        Ok((
            input,
            Self {
                retention_time: Time::new::<millisecond>(retention_time as _),
                base_peak,
                peaks,
            },
        ))
    }
}

impl Display for Spectral {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Spectral")
            .field("base", &self.base_peak)
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
    mass_to_charge: f32,
    abundance: f32,
}

impl Peak {
    pub fn mass_to_charge(&self) -> f32 {
        self.mass_to_charge
    }

    pub fn signal(&self) -> f32 {
        self.abundance
    }
}

impl Parse for Peak {
    fn parse(input: &[u8]) -> Result<(&[u8], Self)> {
        let (input, (mass_to_charge, abundance)) = pair(be_u16, be_i16)(input)?;
        Ok((
            input,
            Self {
                mass_to_charge: mass_to_charge as f32 / MASS_TO_CHARGE_FACTOR,
                abundance: unpack(abundance as _) as _,
            },
        ))
    }
}

impl Display for Peak {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Peak")
            .field("mass_to_charge", &self.mass_to_charge())
            .field("abundance", &self.signal())
            .finish()
    }
}
