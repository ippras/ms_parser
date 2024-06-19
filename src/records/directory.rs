use super::{WordsToBytes, Parse, Spectral};
use crate::error::Result;
use nom::number::complete::be_i32;
use std::fmt::{self, Display, Formatter};
use uom::{
    fmt::DisplayStyle,
    si::{
        f32::Time,
        time::{millisecond, second},
    },
};

pub const DIRECTORY_SIZE: usize = 12;

/// Directory record
#[derive(Debug, Default)]
pub struct Directory {
    pub(crate) spectrum_offset: usize,
    pub(crate) retention_time: Time,
    // This Total Signal value is representative of TIC, but it has been scaled
    // down by some sort of polynomial transformation.
    total_signal: i32,
}

impl Directory {
    pub fn retention_time(&self) -> Time {
        self.retention_time
    }

    pub fn spectrum<'a>(&'a self, input: &'a [u8]) -> Result<(&[u8], Spectral)> {
        Spectral::parse(&input[self.spectrum_offset..])
    }

    pub fn total_signal(&self) -> i32 {
        self.total_signal
    }
}

impl Parse for Directory {
    fn parse(input: &[u8]) -> Result<(&[u8], Self)> {
        assert!(input.len() >= DIRECTORY_SIZE);
        // Spectrum offset in words.
        let (input, spectrum_offset) = be_i32(input)?;
        // Retention time in milliseconds.
        let (input, retention_time) = be_i32(input)?;
        let (input, total_signal) = be_i32(input)?;
        Ok((
            input,
            Self {
                spectrum_offset: spectrum_offset.words_to_bytes(),
                retention_time: Time::new::<millisecond>(retention_time as _),
                total_signal,
            },
        ))
    }
}

impl Display for Directory {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Directory")
            .field(
                "retention_time",
                &self
                    .retention_time
                    .into_format_args(second, DisplayStyle::Description),
            )
            .field("total_signal", &self.total_signal)
            .finish()
    }
}
