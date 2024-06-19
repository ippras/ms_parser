use anyhow::Result;
use ms::Reader;

pub fn main() -> Result<()> {
    // let reader = Reader::new("input/7_FAMES_01.D/DATA.MS")?;
    let reader = Reader::new("input/12_FAMES_01.D/DATA.MS")?;
    println!("header: {:#?}", reader.header());

    // let parse = reader.parse()?;
    // println!("{parse}");
    let parse = reader.parse_data_frame()?;
    // println!("data_frame: {}", data_frame);
    // let grouped = data_frame
    //     .clone()
    //     .lazy()
    //     .group_by(["Retention time"])
    //     .agg([
    //         col("Mass to charge").sort(Default::default()),
    //         col("Signal"),
    //         col("Mass to charge").min().alias("MIN mass to charge"),
    //         col("Mass to charge").max().alias("MAX mass to charge"),
    //         col("Mass to charge").sum().alias("SUM mass to charge"),
    //     ])
    //     .sort(["Retention time"], Default::default())
    //     .collect()?;
    // println!("grouped: {}", grouped);
    Ok(())
}
