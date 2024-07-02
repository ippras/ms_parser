//! Import data stored in Agilent (.D, .MS, .CH) files
//!
//! - [chemplexity](https://github.com/chemplexity/chromatography)
//! - [C# or VB.NET access to reading Agilent Chemstation .D dataset folders](https://github.com/PNNL-Comp-Mass-Spec/ChemstationMSFileReader)
//! - [reddit.com](https://www.reddit.com/r/chemistry/comments/35err3/agilent_file_format_ch_and_ms/)
//!
//! - [Global Natural Products Social Molecular Networking](https://github.com/CCMS-UCSD/GNPSDocumentation)

#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_uninit_array)]
#![feature(sort_floats)]

pub use self::records::{Directory, Header, Normalization, Spectral};

use self::{
    parse::Parse,
    records::{Peak, DIRECTORY_SIZE, HEADER_SIZE, NORMALIZATION_SIZE},
};
use anyhow::{ensure, Ok, Result};
use indexmap::{map::Entry, IndexMap, IndexSet};
use itertools::Itertools;
use nom::{
    bytes::complete::take,
    combinator::{map, map_res},
    multi::{count, length_data},
    number::complete::{be_i16, be_i32, be_u16, u8},
    Err,
};
use ordered_float::{NotNan, OrderedFloat};
use polars::prelude::*;
use std::{
    borrow::Borrow,
    collections::{BTreeSet, HashMap},
    fmt::{self, Display, Formatter},
    fs::read,
    io::Cursor,
    mem::MaybeUninit,
    ops::{Deref, Index, Range, RangeInclusive},
    path::Path,
    str,
};
use uom::{
    fmt::DisplayStyle,
    si::{
        f32::{Ratio, Time},
        ratio::ratio,
        time::{millisecond, minute, second},
    },
};
use utils::{nom::array, Preview};

pub const NORMALIZATIONS_COUNT: usize = 10;

// fn str<const SIZE: usize>(input: &[u8]) -> Result<&[u8], String> {
//     let (input, output) = map_res(length_data(u8), |bytes| str::from_utf8(bytes))(input)?;
//     let (input, _) = take(SIZE - output.len())(input)?;
//     Ok((input, output.trim().to_owned()))
// }

/// Reader
pub struct Reader {
    input: Vec<u8>,
    header: Header,
}

impl Reader {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let input = read(path)?;
        let (_, header) = Header::parse(&input[..HEADER_SIZE])?;
        Ok(Self { input, header })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Normalizations
    ///
    /// Reads the 10 normalization records from the data file.
    pub fn normalizations(&self) -> Result<[Normalization; NORMALIZATIONS_COUNT]> {
        ensure!(
            self.header.normalization_records_offset > 0,
            "normalization records offset is zero"
        );
        let (_, normalizations) =
            array(Normalization::parse)(&self.input[self.header.normalization_records_offset..])?;
        Ok(normalizations)
    }

    // pub fn d<T>(&self, f: impl Fn(Directory) -> T) -> Result<Vec<T>> {
    //     Ok(
    //         count(map(Directory::parse, f), self.header.data_record_count)(
    //             &self.input[self.header.directory_offset..],
    //         )?
    //         .1,
    //     )
    // }

    // pub fn s<T>(&self, f: impl Fn(Spectral) -> T) -> Result<Vec<T>> {
    //     self.d(|directory| {
    //         let (_, spectral) = Spectral::parse(&self.input[directory.spectrum_offset..])?;
    //         f(spectral)
    //     })
    // }

    /// Directories
    pub fn directories(&self) -> Result<Vec<Directory>> {
        let (_, directories) = count(Directory::parse, self.header.data_record_count)(
            &self.input[self.header.directory_offset..],
        )?;
        Ok(directories)
    }

    /// Directory
    pub fn directory(&self, index: usize) -> Result<(&[u8], Directory)> {
        assert!(index < self.header.data_record_count);
        Ok(Directory::parse(
            &self.input[self.header.directory_offset + DIRECTORY_SIZE * index..],
        )?)
    }

    // pub fn parse(self) -> Result<Parsed> {
    //     let (_, spectrums) = count(
    //         map_res(Directory::parse, |directory| {
    //             let (_, spectral) = Spectral::parse(&self.input[directory.spectrum_offset..])?;
    //             let mut signal_range = f32::MAX..=f32::MIN;
    //             for peak in &spectral.peaks {
    //                 let signal = peak.signal();
    //                 signal_range =
    //                     signal_range.start().min(signal)..=signal_range.end().max(signal);
    //             }
    //             Ok(Spectrum {
    //                 retention_time: Time::new::<millisecond>(spectral.retention_time as _),
    //                 base_peak: spectral.base_peak,
    //                 peaks: spectral.peaks,
    //                 signal_range,
    //             })
    //         }),
    //         self.header.data_record_count,
    //     )(&self.input[self.header.directory_offset..])?;
    //     Ok(Parsed {
    //         retention_time_range: Time::new::<millisecond>(
    //             *self.header.retention_time_range.start() as _,
    //         )
    //             ..=Time::new::<millisecond>(*self.header.retention_time_range.end() as _),
    //         signal_range: self.header.signal_range,
    //         spectrums,
    //     })
    // }

    pub fn parse(self) -> Result<DataFrame> {
        let mut retention_time = Vec::new();
        let mut mass_to_charge = Vec::new();
        let mut signal = Vec::new();
        let (_, series) = count(
            map_res(Directory::parse, |directory| {
                let (_, spectral) = Spectral::parse(&self.input[directory.spectrum_offset..])?;
                assert_eq!(directory.retention_time, spectral.retention_time);
                retention_time.push(spectral.retention_time);
                mass_to_charge.push(Series::from_iter(
                    spectral.peaks.iter().map(|peak| peak.mass_to_charge()),
                ));
                signal.push(Series::from_iter(
                    spectral.peaks.iter().map(|peak| peak.abundance),
                ));
                Ok(())
            }),
            self.header.data_record_count,
        )(&self.input[self.header.directory_offset..])?;
        // let retention_time_range = data_frame
        //     .clone()
        //     .lazy()
        //     .select([
        //         min("Retention time").alias("RT.MIN"),
        //         max("Retention time").alias("RT.MAX"),
        //         min("Signal").alias("S.MIN"),
        //         max("Signal").alias("S.MAX"),
        //     ])
        //     .collect()?;
        // matches!(
        //     (
        //         retention_time_range["RT.MIN"].get(0)?,
        //         retention_time_range["RT.MAX"].get(0)?
        //     ),
        //     (Ok)
        // );
        // assert_eq!(
        //     self.header.retention_time_range,
        //     retention_time_range["RT.MIN"].get(0)?.try_extract::<f32>()?
        //         ..=retention_time_range["RT.MAX"].get(0)?.try_extract::<f32>()?
        // );
        // assert_eq!(
        //     self.header.retention_time_range,
        //     retention_time_range["RT.MIN"].get(0)?.try_extract()?
        //         ..=retention_time_range["RT.MAX"].get(0)?.try_extract()?
        // );
        // assert_eq!(
        //     self.header.signal_range,
        //     retention_time_range["S.MIN"].get(0)?.try_extract()?
        //         ..=retention_time_range["S.MAX"].get(0)?.try_extract()?
        // );
        Ok(df! {
            "RetentionTime" => retention_time,
            "MassToCharge" => mass_to_charge,
            "Signal" => signal,
        }?)
    }

    /// Spectral
    pub fn spectral(&self, index: usize) -> Result<Spectral> {
        let (_, directory) = self.directory(index)?;
        let (_, spectral) = Spectral::parse(&self.input[directory.spectrum_offset..])?;
        Ok(spectral)
    }

    /// Times
    pub fn times(&self) -> Result<Vec<Time>> {
        let (_, times) = count(
            map(Directory::parse, |directory| {
                Time::new::<millisecond>(directory.retention_time() as _)
            }),
            self.header.data_record_count,
        )(&self.input[self.header.directory_offset..])?;
        Ok(times)
    }
}

#[derive(Clone, Debug)]
pub struct Parsed {
    spectrums: Vec<Spectrum>,
    retention_time_range: RangeInclusive<Time>,
    signal_range: RangeInclusive<usize>,
}

impl Display for Parsed {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Parsed")
            .field(
                "spectrums",
                &format_args!(
                    "{}",
                    self.spectrums
                        .first()
                        .into_iter()
                        .chain(self.spectrums.last())
                        .map(|spectrum| format!("{spectrum}"))
                        .join(", ..., ")
                ),
            )
            .field("retention_time_range", &self.retention_time_range)
            .field("signal_range", &self.signal_range)
            .finish()
    }
}

// time_range: RangeInclusive<Time>,
#[derive(Clone, Debug)]
pub struct Spectrum {
    retention_time: Time,
    base_peak: Peak,
    peaks: Vec<Peak>,
    signal_range: RangeInclusive<f32>,
}

impl Spectrum {
    pub fn peaks(&self) -> &[Peak] {
        &self.peaks
    }

    pub fn retention_time(&self) -> Time {
        self.retention_time
    }
}

impl Display for Spectrum {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Spectrum")
            .field("retention_time", &self.retention_time)
            .field("base_peak", &self.base_peak)
            // .field(
            //     "peaks",
            //     &self
            //         .peaks
            //         .first()
            //         .into_iter()
            //         .chain(self.peaks.last())
            //         .map(|peak| format!("{peak}"))
            //         .join(", ..., "),
            // )
            .field("peaks", &format_args!("{}", Preview(&self.peaks)))
            .field("signal_range", &self.signal_range)
            .finish()
    }
}

pub mod records;

mod error;
mod parse;
mod utils;

#[test]
fn test() -> anyhow::Result<()> {
    let reader = Reader::new("input/7_FAMES_01.D/DATA.MS")?;
    println!("header: {:#?}", reader.header());

    // let parse = reader.parse()?;
    // println!("{parse}");
    let parse = reader.parse_data_frame()?;
    Ok(())
}
