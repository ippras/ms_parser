//! Import data stored in Agilent (.D, .MS, .CH) files
//!
//! - [chemplexity](https://github.com/chemplexity/chromatography)
//! - [C# or VB.NET access to reading Agilent Chemstation .D dataset folders](https://github.com/PNNL-Comp-Mass-Spec/ChemstationMSFileReader)
//! - [reddit.com](https://www.reddit.com/r/chemistry/comments/35err3/agilent_file_format_ch_and_ms/)
//!
//! - [Global Natural Products Social Molecular Networking](https://github.com/CCMS-UCSD/GNPSDocumentation)

#![feature(default_free_fn)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_uninit_array)]
#![feature(associated_type_bounds)]

use crate::records::{Peak, PEAK_SIZE};

pub use self::records::{Directory, Header, Normalization, Spectral};

use self::{
    parse::Parse,
    records::{DIRECTORY_SIZE, HEADER_SIZE, NORMALIZATION_SIZE},
};
use anyhow::{ensure, Result};
use indexmap::{map::Entry, IndexMap, IndexSet};
use ndarray::{aview1, s, Array, Array2, ArrayView};
use nom::{
    bytes::complete::take,
    combinator::map_res,
    multi::{count, length_data},
    number::complete::{be_i16, be_i32, be_u16, u8},
    Err,
};
use ordered_float::{NotNan, OrderedFloat};
use std::{
    collections::{BTreeSet, HashMap},
    fs::read,
    io::Cursor,
    mem::MaybeUninit,
    path::Path,
    str,
};
use uom::{
    fmt::DisplayStyle,
    si::{
        f32::Time,
        ratio::ratio,
        time::{minute, second},
    },
};
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
        let start = self.header.directory_offset + DIRECTORY_SIZE * index;
        Ok(Directory::parse(&self.input[start..])?)
    }

    /// Spectrals
    pub fn spectrals(&self) -> Result<(Vec<Time>, Array2<f32>)> {
        let mut times = Vec::with_capacity(self.header.data_record_count);
        // let mut mass_to_charges = BTreeSet::new();
        let mut array = Array2::zeros((self.header.data_record_count, 0));
        for i in 0..self.header.data_record_count {
            let directory_offset = self.header.directory_offset + i * DIRECTORY_SIZE;
            let (_, directory) = Directory::parse(&self.input[directory_offset..])?;
            times.push(directory.retention_time);
            let (_, spectral) = Spectral::parse(&self.input[directory.spectrum_offset..])?;
            // for peak in spectral.peaks() {
            //     if mass_to_charges.insert(peak.mass_to_charge) {
            //         array.push_column(aview1(&vec![0.0; self.header.data_record_count]))?;
            //     }
            //     // array[[i, peak.mass_to_charge as _]] = peak.abundance();
            // }
            // for (j, abundance) in spectral.peaks() {
            //     array[[i, j]] = abundance;
            // }
        }
        Ok((times, array))
    }

    /// Spectral
    pub fn spectral(&self, index: usize) -> Result<Spectral> {
        let (_, directory) = self.directory(index)?;
        let (_, spectral) = Spectral::parse(&self.input[directory.spectrum_offset..])?;
        Ok(spectral)
    }

    pub fn peak(&self, index: (usize, usize)) -> Result<()> {
        // let (_, spectral) = Spectral::parse(&self.input[directory.spectrum_offset..])?;
        Ok(())
    }

    // pub fn mz(&self) -> Result<()> {
    // }

    pub fn test(&self) -> Result<()> {
        let mut times = Vec::with_capacity(self.header.data_record_count);
        let mut masses = IndexSet::new();
        let mut abundances = HashMap::new();
        let mut input = &self.input[self.header.directory_offset..];
        for i in 0..self.header.data_record_count {
            let directory;
            (input, directory) = Directory::parse(input)?;
            times.push(directory.retention_time);
            let (mut input, spectral) = Spectral::parse(&self.input[directory.spectrum_offset..])?;
            for mut j in 0..spectral.number_of_peaks() {
                let peak;
                (input, peak) = Peak::parse(input)?;
                (j, _) = masses.insert_full(OrderedFloat(peak.mass_to_charge()));
                abundances.insert((i, j), peak.abundance());
            }
        }
        let mut array = Array2::zeros((times.len(), masses.len()));
        for ((i, j), abundance) in abundances {
            array[(i, j)] = abundance;
        }

        for ((time, mass), abundance) in abundances {
            array[(times[time], masses[mass])] = abundance;
        }

        // let array = Array2::from_shape_fn((times.len(), mz.len()), |(i, j)| {
        //     abundances.get(&(i, j)).unwrap_or_default()
        // });
        // println!("{mz:?}");
        println!("{}", masses.len());
        // println!("{times:?}");
        println!("{}", times.len());
        println!("{}", array.len());
        println!(
            "time: {}",
            times[0].into_format_args(minute, DisplayStyle::Abbreviation)
        );
        println!("row: {:?}", [masses[0].0, masses[1].0, masses[2].0, masses[3].0, masses[4].0]);
        println!("row: {}", array.row(0));
        // println!("{mass_to_charges:?}");

        // let mut peak_numbers = Vec::new();
        // for spectrum_offset in &mut spectrum_offsets {
        //     let (mut input, spectral_header) =
        //         SpectralHeader::parse(&self.input[*spectrum_offset..])?;
        // }
        // times.sort_unstable_by(|a, b| a.value.total_cmp(&b.value));

        // for i in 0..self.header.data_record_count {
        //     let (_, directory) =
        //         Directory::parse(&self.input[self.header.directory_offset + i * DIRECTORY_SIZE..])?;
        //     times.push(directory.retention_time);
        //     println!(
        //         "{:x} {:x?}",
        //         directory.spectrum_offset,
        //         &self.input[directory.spectrum_offset..directory.spectrum_offset + 8]
        //     );
        //     let (mut input, spectral_header) =
        //         SpectralHeader::parse(&self.input[directory.spectrum_offset..])?;
        //     let mut peak;
        //     for j in 0..spectral_header.number_of_peaks() {
        //         (input, peak) = Peak::parse(input)?;
        //         if mz.insert(NotNan::new(peak.mass_to_charge()).unwrap()) {
        //             array.insert(0, 0.0);
        //             //     // array.push_column(aview1(&vec![0.0; self.header.data_record_count]))?;
        //         }
        //         // array.push(peak.abundance());
        //     }
        //     // for peak in spectral.peaks() {
        //     //     // println!("{:?}", peak.mass_to_charge());
        //     //     //     if mass_to_charges.insert(peak.mass_to_charge) {
        //     //     //         // array.push_column(aview1(&vec![0.0; self.header.data_record_count]))?;
        //     //     //     }
        //     // }
        // }
        Ok(())
    }
}

// fn parse(input: &[u8]) -> Result<(&[u8], ()), Err<Error>> {
//     // Header
//     let (_, header) = Header::parse(input)?;
//     println!("{:#?}", header);
//     println!(
//         "time: {}..={}",
//         header
//             .retention_time_range
//             .start()
//             .into_format_args(minute, DisplayStyle::Abbreviation),
//         header
//             .retention_time_range
//             .end()
//             .into_format_args(second, DisplayStyle::Abbreviation),
//     );
//     let mut times = Vec::with_capacity(header.data_record_count);
//     let mut array = Array2::zeros((header.data_record_count, 1000));
//     for i in 0..header.data_record_count {
//         let directory_offset = header.directory_offset + i * DIRECTORY_SIZE;
//         let (_, directory) = Directory::parse(&input[directory_offset..])?;
//         let (_, spectral) = Spectral::parse(&input[directory.spectrum_offset..])?;
//         times.push(directory.retention_time);
//         for (j, abundance) in spectral.peaks() {
//             array[[i, j as _]] = abundance;
//             // .push_row(ArrayView::from(&spectral.abundances()))
//             // .unwrap();
//         }
//         // array.fill(index);
//     }
//     println!("{array:?}");

//     // // Directories
//     // let (_, directories) =
//     // count(DirectoryRecord::parse, header.data_record_count)(&input[header.directory_offset..])?;
//     // let directory = &directories[i];
//     // let (_, spectral) = SpectralRecord::parse(&input[directory.spectrum_offset..]).unwrap();
//     // for directory in &directories {
//     //     times.push(directory.retention_time)
//     //     // println!("{directory}");
//     //     // println!(
//     //     //     "retention_time: {}, total_signal: {}",
//     //     //     directory
//     //     //         .retention_time
//     //     //         .into_format_args(minute, Abbreviation),
//     //     //     directory.total_signal,
//     //     // );
//     //     // let (_, spectral) = SpectralRecord::parse(&input[directory.spectrum_offset..])?;
//     //     // println!("{}", spectral);
//     // }
//     // println!("{:#?}", &directories[..4]);
//     // let len = directories.len();
//     // println!("{:#?}", &directories[len - 4..]);

//     // directory_offset: 7896524,
//     // data_offset: 5768,
//     // Spectral
//     let (_, spectral) = Normalization::parse(&input[header.normalization_records_offset..])?;
//     // Directory
//     let (_, directory) = Directory::parse(&input[header.directory_offset..])?;
//     println!("{directory}");
//     // Spectral
//     let (_, spectral) = Spectral::parse(&input[header.data_offset..])?;
//     // println!("{:#?}", spectral);
//     println!("{spectral}");
//     Ok((input, ()))
// }

pub mod records;

mod error;
mod parse;
mod utils;

#[test]
fn test() -> anyhow::Result<()> {
    let reader = Reader::new("input/7_FAMES_01.D/DATA.MS")?;
    // let normalizations = reader.normalizations()?;
    // println!("normalizations: {normalizations:?}");
    println!("header: {:#?}", reader.header());
    let spectral = reader.spectral(0)?;
    let peaks = reader.test();
    // println!("{}", peaks.first().unwrap());
    // println!("{}", peaks.last().unwrap());
    // let directories = reader.directories()?;
    // println!("directories: {}", directories.len());
    // for index in 0..10 {
    //     println!("directories: {:?}", reader.directory(index)?.1);
    //     let spectral = reader.spectral(index)?;
    //     println!("spectral: {}", spectral);
    // }
    // let (times, spectrals) = reader.spectrals()?;
    // // println!("times: {:?}", times);
    // println!("spectrals: {}", spectrals);
    Ok(())
}
