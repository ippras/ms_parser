use crate::error::Result;
use nom::{
    bytes::complete::take,
    combinator::{map, map_res},
    multi::length_data,
    number::complete::u8,
};
use std::{mem::MaybeUninit, str};

pub fn array<const LENGTH: usize, O>(
    mut inner: impl FnMut(&[u8]) -> Result<(&[u8], O)>,
) -> impl FnMut(&[u8]) -> Result<(&[u8], [O; LENGTH])> {
    move |mut input: &[u8]| {
        let mut array = MaybeUninit::uninit_array();
        let mut item;
        for index in 0..LENGTH {
            (input, item) = inner(input)?;
            array[index].write(item);
        }
        Ok((input, unsafe { MaybeUninit::array_assume_init(array) }))
    }
}

fn str<const SIZE: usize>(input: &[u8]) -> Result<(&[u8], &str)> {
    let (input, output) = map_res(length_data(u8), |bytes| str::from_utf8(bytes))(input)?;
    let (input, _) = take(SIZE - output.len())(input)?;
    Ok((input, output.trim()))
}

pub fn string<const SIZE: usize>(input: &[u8]) -> Result<(&[u8], String)> {
    map(str::<SIZE>, ToOwned::to_owned)(input)
}
