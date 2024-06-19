use super::{Parse, WordsToBytes};
use crate::{error::Result, utils::nom::string};
use nom::{
    bytes::complete::take,
    number::complete::{be_i16, be_i32},
};
use std::{ops::RangeInclusive, str};

pub const HEADER_SIZE: usize = 512;

const FILE_NUMBER: usize = 3;
const FILE_STRING: usize = 19;
const DATA_NAME: usize = 61;
const MISC_INFO: usize = 61;
const OPERATOR_NAME: usize = 29;
const ACQ_DATE: usize = 29;
const INSTRUMENT_MODEL: usize = 9;
const INLET: usize = 9;
const METHOD_FILE: usize = 19;

/// Header record
#[derive(Debug)]
pub struct Header {
    file_number: String,
    file_string: String,
    data_name: String,
    misc_info: String,
    operator_name: String,
    acq_date: String,
    instrument_model: String,
    inlet: String,
    method_file: String,
    file_type: i32,
    seq_index: i16,
    als_bottle: i16,
    replicate: i16,
    directory_entry_type: i16,
    /// `Directory` records (TIC) offset.
    pub directory_offset: usize,
    /// `Spectral` records (MZ) offset.
    pub data_offset: usize,
    /// Unused.
    _run_table_offset: i32,
    /// `Nomalization` records offset.
    pub normalization_records_offset: usize,
    extra_records: i16,
    pub data_record_count: usize,
    pub retention_time_range: RangeInclusive<i32>,
    pub signal_range: RangeInclusive<usize>,
}

impl Header {
    pub fn file(&self) -> &str {
        &self.file_string
    }

    // TODO: std::result::Result -> Result
    pub fn file_number(&self) -> std::result::Result<usize, &str> {
        self.file_number.parse().map_err(|_| &*self.file_number)
    }
}

impl Parse for Header {
    fn parse(input: &[u8]) -> Result<(&[u8], Self)> {
        assert_eq!(input.len(), HEADER_SIZE);
        let (output, input) = take(HEADER_SIZE)(input)?;
        let (input, file_number) = string::<FILE_NUMBER>(input)?;
        let (input, file_string) = string::<FILE_STRING>(input)?;
        let (input, data_name) = string::<DATA_NAME>(input)?;
        let (input, misc_info) = string::<MISC_INFO>(input)?;
        let (input, operator_name) = string::<OPERATOR_NAME>(input)?;
        let (input, acq_date) = string::<ACQ_DATE>(input)?;
        let (input, instrument_model) = string::<INSTRUMENT_MODEL>(input)?;
        let (input, inlet) = string::<INLET>(input)?;
        let (input, method_file) = string::<METHOD_FILE>(input)?;
        let (input, file_type) = be_i32(input)?;
        let (input, seq_index) = be_i16(input)?;
        let (input, als_bottle) = be_i16(input)?;
        let (input, replicate) = be_i16(input)?;
        let (input, directory_entry_type) = be_i16(input)?;
        let (input, directory_offset) = be_i32(input)?;
        let (input, data_offset) = be_i32(input)?;
        let (input, run_table_offset) = be_i32(input)?;
        let (input, normalization_records_offset) = be_i32(input)?;
        let (input, extra_records) = be_i16(input)?;
        let (input, data_record_count) = be_i32(input)?;
        let (input, retention_time_msec_start) = be_i32(input)?;
        let (input, retention_time_msec_end) = be_i32(input)?;
        let (input, signal_maximum) = be_i32(input)?;
        let (input, signal_minimum) = be_i32(input)?;
        assert_eq!(input.len(), 214);
        Ok((
            output,
            Self {
                file_number,
                file_string,
                data_name,
                misc_info,
                operator_name,
                acq_date,
                instrument_model,
                inlet,
                method_file,
                file_type,
                seq_index,
                als_bottle,
                replicate,
                directory_entry_type,
                directory_offset: directory_offset.words_to_bytes(),
                data_offset: data_offset.words_to_bytes(),
                _run_table_offset: run_table_offset,
                normalization_records_offset: normalization_records_offset.words_to_bytes(),
                extra_records,
                data_record_count: data_record_count as _,
                retention_time_range: retention_time_msec_start..=retention_time_msec_end,
                signal_range: signal_minimum as _..=signal_maximum as _,
            },
        ))
    }
}
