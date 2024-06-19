use anyhow::Result;
use ms::Reader;

pub fn main() -> Result<()> {
    let reader = Reader::new("input/7_FAMES_01.D/DATA.MS")?;
    println!("header: {:#?}", reader.header());

    // let parse = reader.parse()?;
    // println!("{parse}");
    let parse = reader.parse_data_frame()?;
    Ok(())
}
