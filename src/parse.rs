use crate::error::Result;

/// Parse
pub trait Parse: Sized {
    fn parse(input: &[u8]) -> Result<(&[u8], Self)>;
}
