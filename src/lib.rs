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
    cmp::min,
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

    pub fn parse(self) -> Result<Parsed> {
        let (_, spectrums) = count(
            map_res(Directory::parse, |directory| {
                let (_, spectral) = Spectral::parse(&self.input[directory.spectrum_offset..])?;
                let mut signal_range = f32::MAX..=f32::MIN;
                for peak in &spectral.peaks {
                    let signal = peak.signal();
                    signal_range =
                        signal_range.start().min(signal)..=signal_range.end().max(signal);
                }
                Ok(Spectrum {
                    retention_time: spectral.retention_time,
                    base_peak: spectral.base_peak,
                    peaks: spectral.peaks,
                    signal_range,
                })
            }),
            self.header.data_record_count,
        )(&self.input[self.header.directory_offset..])?;
        Ok(Parsed {
            retention_time_range: self.header.retention_time_range,
            signal_range: self.header.signal_range,
            spectrums,
        })
    }

    pub fn parse_data_frame(self) -> Result<DataFrame> {
        // let s1 = Series::new("Fruit", &["Apple", "Apple", "Pear"]);
        // let s2 = Series::new("Color", &["Red", "Yellow", "Green"]);
        let mut data_frame = DataFrame::empty();
        // let mut ecn: ListPrimitiveChunkedBuilder<Int32Type> =
        //     ListPrimitiveChunkedBuilder::new("PECN", 8, 8, DataType::Int32);
        // ecn.append_slice(&[12, 12, 12]);
        // let mut retention_time: PrimitiveChunkedBuilder<Float32Type> =
        //     PrimitiveChunkedBuilder::new("Retention time", self.header.data_record_count);
        let (_, series) = count(
            map_res(Directory::parse, |directory| {
                let (_, spectral) = Spectral::parse(&self.input[directory.spectrum_offset..])?;
                let length = spectral.peaks.len();
                let mut mass_to_charges: PrimitiveChunkedBuilder<Float32Type> =
                    PrimitiveChunkedBuilder::new("Mass to charge", length);
                let mut signals: PrimitiveChunkedBuilder<Float32Type> =
                    PrimitiveChunkedBuilder::new("Signal", length);
                for peak in &spectral.peaks {
                    mass_to_charges.append_value(peak.mass_to_charge());
                    signals.append_value(peak.signal());
                }
                assert_eq!(directory.retention_time, spectral.retention_time);
                data_frame.vstack_mut(&df! {
                    "Retention time" => vec![spectral.retention_time.get::<second>(); length],
                    "Mass to charge" => mass_to_charges.finish(),
                    "Signal" => signals.finish(),
                }?)?;
                Ok(())
            }),
            self.header.data_record_count,
        )(&self.input[self.header.directory_offset..])?;

        // 0.10
        // 0.05
        println!("data_frame: {}", data_frame);
        let grouped = data_frame
            .clone()
            .lazy()
            .group_by(["Retention time"])
            .agg([
                col("Mass to charge").sort(Default::default()),
                col("Signal"),
                col("Mass to charge").min().alias("MIN mass to charge"),
                col("Mass to charge").max().alias("MAX mass to charge"),
                col("Mass to charge").sum().alias("SUM mass to charge"),
            ])
            .sort(["Retention time"], Default::default())
            .collect()?;
        println!("grouped: {}", grouped);
        // assert_eq!(self.header.retention_time_range, );
        // assert_eq!(self.header.signal_range, );
        Ok(data_frame)
        // Ok(Parsed {
        //     retention_time_range: self.header.retention_time_range,
        //     signal_range: self.header.signal_range,
        //     spectrums,
        // })
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
            map(Directory::parse, |directory| directory.retention_time()),
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
