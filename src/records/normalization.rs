use crate::{error::Result, parse::Parse};
use nom::number::complete::be_f32;
use std::fmt::{self, Display, Formatter};

pub const NORMALIZATION_SIZE: usize = 24;

/// Normalization record
#[derive(Debug, Default)]
pub struct Normalization {
    normalization_mass: f32,
    slope: f32,
    intercept: f32,
}

impl Parse for Normalization {
    fn parse(input: &[u8]) -> Result<(&[u8], Self)> {
        assert!(input.len() > NORMALIZATION_SIZE);
        // Normalization mass.
        let (input, normalization_mass) = be_f32(input)?;
        // Slope.
        let (input, slope) = be_f32(input)?;
        // Intercept.
        let (input, intercept) = be_f32(input)?;
        Ok((
            input,
            Self {
                normalization_mass,
                slope,
                intercept,
            },
        ))
    }
}

impl Display for Normalization {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Normalization")
            .field("normalization_mass", &self.normalization_mass)
            .field("slope", &self.slope)
            .field("intercept", &self.intercept)
            .finish()
    }
}
