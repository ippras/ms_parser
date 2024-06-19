use anyhow::Result;
use ms::Reader;
use polars::prelude::*;
use std::{env, fs};

pub fn main() -> Result<()> {
    env::set_var("POLARS_FMT_MAX_COLS", "256");
    env::set_var("POLARS_FMT_MAX_ROWS", "256");
    env::set_var("POLARS_FMT_TABLE_CELL_LIST_LEN", "256");
    // env::set_var("POLARS_FMT_STR_LEN", "256");

    // let reader = Reader::new("input/7_FAMES_01.D/DATA.MS")?;
    // let reader = Reader::new("input/12_FAMES_01.D/DATA.MS")?;
    let reader = Reader::new("input/Amal/40/Flow/SUP37-40C-5C-MIN-FLOW-1.D/DATA.MS")?;
    println!("header: {:#?}", reader.header());

    // let parse = reader.parse()?;
    // println!("{parse}");
    let data_frame = reader
        .parse_data_frame()?
        .sort(["Retention time"], Default::default())?;
    let contents = ron::ser::to_string_pretty(&df, PrettyConfig::default())?;
    fs::write("df.ron", contents)?;
    println!("data_frame: {}", data_frame);
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
