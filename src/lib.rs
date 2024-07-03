//! Import data stored in Agilent (.D, .MS, .CH) files
//!
//! - [chemplexity](https://github.com/chemplexity/chromatography)
//! - [C# or VB.NET access to reading Agilent Chemstation .D dataset folders](https://github.com/PNNL-Comp-Mass-Spec/ChemstationMSFileReader)
//! - [reddit.com](https://www.reddit.com/r/chemistry/comments/35err3/agilent_file_format_ch_and_ms/)
//!
//! - [Global Natural Products Social Molecular Networking](https://github.com/CCMS-UCSD/GNPSDocumentation)

#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_uninit_array)]

pub use self::records::{Directory, Header, Normalization, Spectral};

use self::{
    parse::Parse,
    records::{DIRECTORY_SIZE, HEADER_SIZE},
};
use anyhow::{ensure, Ok, Result};
use nom::{combinator::map_res, multi::count};
use std::{fs::read, path::Path};
use utils::nom::array;

pub const NORMALIZATIONS_COUNT: usize = 10;

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

    /// Spectrals
    pub fn spectrals(&self) -> Result<Vec<Spectral>> {
        let (_, spectrals) = count(
            map_res(Directory::parse, |directory| {
                let (_, spectral) = Spectral::parse(&self.input[directory.spectrum_offset..])?;
                assert_eq!(directory.retention_time, spectral.retention_time);
                Ok(spectral)
            }),
            self.header.data_record_count,
        )(&self.input[self.header.directory_offset..])?;
        Ok(spectrals)
    }

    /// Spectral
    pub fn spectral(&self, index: usize) -> Result<Spectral> {
        let (_, directory) = self.directory(index)?;
        let (_, spectral) = Spectral::parse(&self.input[directory.spectrum_offset..])?;
        Ok(spectral)
    }

    // pub fn parse(self) -> Result<DataFrame> {
    //     let mut retention_time = Vec::new();
    //     let mut mass_to_charge = Vec::new();
    //     let mut signal = Vec::new();
    //     for spectral in self.spectrals()? {
    //         retention_time.push(spectral.retention_time);
    //         mass_to_charge.push(Series::from_iter(
    //             spectral.peaks.iter().map(|peak| peak.mass_to_charge()),
    //         ));
    //         signal.push(Series::from_iter(
    //             spectral.peaks.iter().map(|peak| Some(peak.abundance)),
    //         ));
    //     }
    //     Ok(df! {
    //         "RetentionTime" => retention_time,
    //         "MassToCharge" => mass_to_charge,
    //         "Signal" => signal,
    //     }?)
    // }

    // pub fn parse(self) -> Result<DataFrame> {
    //     let mut retention_time = Vec::new();
    //     let mut mass_to_charge = Vec::new();
    //     let mut signal = Vec::new();

    //     let mut input = &self.input[self.header.directory_offset..];
    //     for _ in 0..self.header.data_record_count {
    //         let (output, directory) = Directory::parse(input)?;
    //         let (_, spectral) = Spectral::parse(&self.input[directory.spectrum_offset..])?;
    //         assert_eq!(directory.retention_time, spectral.retention_time);
    //         retention_time.push(spectral.retention_time);
    //         mass_to_charge.push(Series::from_iter(
    //             spectral.peaks.iter().map(|peak| peak.mass_to_charge()),
    //         ));
    //         signal.push(Series::from_iter(
    //             spectral.peaks.iter().map(|peak| Some(peak.abundance)),
    //         ));
    //         input = output;
    //     }
    //     // let mut retention_time = Vec::new();
    //     // let mut mass_to_charge = Vec::new();
    //     // let mut signal = Vec::new();
    //     Ok(df! {
    //         "RetentionTime" => retention_time,
    //         "MassToCharge" => mass_to_charge,
    //         "Signal" => signal,
    //     }?)

    //     // let (_, series) = count(
    //     //     map_res(Directory::parse, |directory| {
    //     //         let (_, spectral) = Spectral::parse(&self.input[directory.spectrum_offset..])?;
    //     //         assert_eq!(directory.retention_time, spectral.retention_time);
    //     //         retention_time.push(spectral.retention_time);
    //     //         mass_to_charge.push(Series::from_iter(
    //     //             spectral.peaks.iter().map(|peak| peak.mass_to_charge()),
    //     //         ));
    //     //         signal.push(Series::from_iter(
    //     //             spectral.peaks.iter().map(|peak| peak.abundance),
    //     //         ));
    //     //         Ok(())
    //     //     }),
    //     //     self.header.data_record_count,
    //     // )(&self.input[self.header.directory_offset..])?;

    //     // let retention_time_range = data_frame
    //     //     .clone()
    //     //     .lazy()
    //     //     .select([
    //     //         min("Retention time").alias("RT.MIN"),
    //     //         max("Retention time").alias("RT.MAX"),
    //     //         min("Signal").alias("S.MIN"),
    //     //         max("Signal").alias("S.MAX"),
    //     //     ])
    //     //     .collect()?;
    //     // matches!(
    //     //     (
    //     //         retention_time_range["RT.MIN"].get(0)?,
    //     //         retention_time_range["RT.MAX"].get(0)?
    //     //     ),
    //     //     (Ok)
    //     // );
    //     // assert_eq!(
    //     //     self.header.retention_time_range,
    //     //     retention_time_range["RT.MIN"].get(0)?.try_extract::<f32>()?
    //     //         ..=retention_time_range["RT.MAX"].get(0)?.try_extract::<f32>()?
    //     // );
    //     // assert_eq!(
    //     //     self.header.retention_time_range,
    //     //     retention_time_range["RT.MIN"].get(0)?.try_extract()?
    //     //         ..=retention_time_range["RT.MAX"].get(0)?.try_extract()?
    //     // );
    //     // assert_eq!(
    //     //     self.header.signal_range,
    //     //     retention_time_range["S.MIN"].get(0)?.try_extract()?
    //     //         ..=retention_time_range["S.MAX"].get(0)?.try_extract()?
    //     // );
    // }
}

pub mod records;

mod error;
mod parse;
mod utils;

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

// #[derive(Clone, Debug)]
// pub struct Parsed {
//     spectrums: Vec<Spectrum>,
//     retention_time_range: RangeInclusive<Time>,
//     signal_range: RangeInclusive<usize>,
// }

// impl Display for Parsed {
//     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//         f.debug_struct("Parsed")
//             .field(
//                 "spectrums",
//                 &format_args!(
//                     "{}",
//                     self.spectrums
//                         .first()
//                         .into_iter()
//                         .chain(self.spectrums.last())
//                         .map(|spectrum| format!("{spectrum}"))
//                         .join(", ..., ")
//                 ),
//             )
//             .field("retention_time_range", &self.retention_time_range)
//             .field("signal_range", &self.signal_range)
//             .finish()
//     }
// }

// // time_range: RangeInclusive<Time>,
// #[derive(Clone, Debug)]
// pub struct Spectrum {
//     retention_time: Time,
//     base_peak: Peak,
//     peaks: Vec<Peak>,
//     signal_range: RangeInclusive<f32>,
// }

// impl Spectrum {
//     pub fn peaks(&self) -> &[Peak] {
//         &self.peaks
//     }

//     pub fn retention_time(&self) -> Time {
//         self.retention_time
//     }
// }

// impl Display for Spectrum {
//     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//         f.debug_struct("Spectrum")
//             .field("retention_time", &self.retention_time)
//             .field("base_peak", &self.base_peak)
//             // .field(
//             //     "peaks",
//             //     &self
//             //         .peaks
//             //         .first()
//             //         .into_iter()
//             //         .chain(self.peaks.last())
//             //         .map(|peak| format!("{peak}"))
//             //         .join(", ..., "),
//             // )
//             .field("peaks", &format_args!("{}", Preview(&self.peaks)))
//             .field("signal_range", &self.signal_range)
//             .finish()
//     }
// }
