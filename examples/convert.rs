use anyhow::Result;
use clap::Parser;
use ms_parser::Reader;
use ron::ser::PrettyConfig;
use std::{env, fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

// cargo run --bin=convert -- --path="input/12_FAMES_01.D/DATA.MS"
// cargo run --bin=convert -- --path="input/Amal/60/Flow 1 ml-min/SUP37-60C-3C-MIN_30 PSI-1.D/DATA.MS"
pub fn main() -> Result<()> {
    env::set_var("POLARS_FMT_MAX_COLS", "256");
    env::set_var("POLARS_FMT_MAX_ROWS", "256");
    env::set_var("POLARS_FMT_TABLE_CELL_LIST_LEN", "256");
    // env::set_var("POLARS_FMT_STR_LEN", "256");

    let args = Args::parse();

    // let reader = Reader::new("input/7_FAMES_01.D/DATA.MS")?;
    // let reader = Reader::new("input/12_FAMES_01.D/DATA.MS")?;
    // let reader = Reader::new("input/Amal/40/Flow/SUP37-40C-5C-MIN-FLOW-1.D/DATA.MS")?;
    // let reader = Reader::new("input/Amal/60/Flow 1 ml-min/SUP37-60C-3C-MIN_30 PSI-1.D/DATA.MS")?;
    let reader = Reader::new(&args.path)?;
    println!("header: {:#?}", reader.header());

    // let parse = reader.parse()?;
    // println!("{parse}");
    let data_frame = reader
        .parse()?
        .sort(["RetentionTime"], Default::default())?;
    // let contents = ron::ser::to_string_pretty(&data_frame, PrettyConfig::default())?;
    // fs::write("df.ron", contents)?;
    // let contents = ron::ser::to_string_pretty(&data_frame, PrettyConfig::default())?;
    let contents = bincode::serialize(&data_frame)?;
    fs::write("df.bin", contents)?;

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
