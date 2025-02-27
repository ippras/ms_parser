use anyhow::Result;
use clap::Parser;
use polars::prelude::*;
use std::{env, ffi::OsStr, fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

pub fn main() -> Result<()> {
    unsafe { env::set_var("POLARS_FMT_MAX_COLS", "256") };
    unsafe { env::set_var("POLARS_FMT_MAX_ROWS", "256") };
    unsafe { env::set_var("POLARS_FMT_TABLE_CELL_LIST_LEN", "256") };
    unsafe { env::set_var("POLARS_FMT_STR_LEN", "80") };

    let args = Args::parse();
    let data_frame: DataFrame = match args.path.extension().and_then(OsStr::to_str) {
        Some("bin") => bincode::deserialize(&fs::read(&args.path)?)?,
        Some("ron") => ron::de::from_str(&fs::read_to_string(&args.path)?)?,
        _ => panic!("unsupported input file extension"),
    };
    println!("{}", data_frame);
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
