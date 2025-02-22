use anyhow::Result;
use clap::{Parser, ValueEnum};
use ms_parser::Reader;
use polars::prelude::*;
use ron::{extensions::Extensions, ser::PrettyConfig};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    path: PathBuf,
    #[arg(default_value_t, long, short, value_enum)]
    format: Format,
}

/// Format
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    #[default]
    Bin,
    Ron,
}

pub fn main() -> Result<()> {
    unsafe { env::set_var("POLARS_FMT_MAX_COLS", "256") };
    unsafe { env::set_var("POLARS_FMT_MAX_ROWS", "256") };
    unsafe { env::set_var("POLARS_FMT_TABLE_CELL_LIST_LEN", "256") };
    // unsafe { env::set_var("POLARS_FMT_STR_LEN", "256") };

    let args = Args::parse();

    let reader = Reader::new(&args.path)?;
    println!("header: {:#?}", reader.header());

    let mut retention_time = Vec::new();
    let mut mass_to_charge = Vec::new();
    let mut signal = Vec::new();
    for spectral in reader.spectrals()? {
        retention_time.push(spectral.retention_time());
        mass_to_charge.push(Series::from_iter(
            spectral.peaks().iter().map(|peak| peak.mass_to_charge()),
        ));
        signal.push(Series::from_iter(
            spectral.peaks().iter().map(|peak| peak.packed_signal()),
        ));
    }
    let data_frame = df! {
        "RetentionTime" => retention_time,
        "MassToCharge" => mass_to_charge,
        "Signal" => signal,
    }?
    .lazy()
    .explode(["MassToCharge", "Signal"])
    .sort_by_exprs(
        [col("RetentionTime"), col("MassToCharge")],
        Default::default(),
    )
    .collect()?;

    // lazy_data_frame = lazy_data_frame
    // .explode(["MassToCharge", "Signal"])
    // // .group_by([col("RetentionTime"), col("MassToCharge")])
    // // .agg([col("Signal")])
    // .sort_by_exprs(
    //     [col("RetentionTime"), col("MassToCharge")],
    //     Default::default(),
    // )
    // .with_columns([
    //     col("MassToCharge"),
    //     col("Signal"),
    //     as_struct(vec![col("MassToCharge"), col("Signal")]).alias("Peak"),
    // ])
    // .group_by([col("RetentionTime")])

    match args.format {
        Format::Bin => {
            let contents = bincode::serialize(&data_frame)?;
            fs::write("df.bin", contents)?;
        }
        Format::Ron => {
            let contents = ron::ser::to_string_pretty(
                &data_frame,
                PrettyConfig::default().extensions(Extensions::IMPLICIT_SOME),
            )?;
            fs::write("df.ron", contents)?;
        }
    }

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
