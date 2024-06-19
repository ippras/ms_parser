pub use self::{
    directory::{Directory, DIRECTORY_SIZE},
    header::{Header, HEADER_SIZE},
    normalization::{Normalization, NORMALIZATION_SIZE},
    spectral::{Peak, Spectral},
};
use super::Parse;

/// WordsToBytes
trait WordsToBytes {
    fn words_to_bytes(&self) -> usize;
}

impl WordsToBytes for i32 {
    fn words_to_bytes(&self) -> usize {
        // (*self as usize - 1) * 2
        (*self as usize).saturating_sub(1) * 2
    }
}

mod directory;
mod header;
mod normalization;
mod spectral;
