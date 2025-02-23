use anyhow::Result;
use chrono::NaiveDateTime;
use clap::{Parser, ValueEnum};
use metadata::{MetaDataFrame, Metadata};
use ms_parser::Reader;
use polars::prelude::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{env, fs::File, iter, path::PathBuf};
use tracing::{error, info, warn};

const DBYHMP: &str = "%d %b %y %I:%M %P";

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
    Ipc,
    Ron,
}

pub fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // let args = Args::parse();

    // let t = 60;
    // let dt = 1;
    for t in (60..=140).step_by(10) {
        for dt in 1..=10 {
            for index in 1..=4 {
                let path = PathBuf::from(format!("input/ms/constant-flow/{t}/{t}-{dt}.{index}.ms"));
                if !path.exists() {
                    error!(?path);
                    continue;
                } else {
                    println!("{}", path.display());
                }
                let reader = Reader::new(&path)?;
                let header = reader.header();
                // println!("header: {header:#?}");

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

                let mut meta = Metadata::default();
                meta.name = format!("constant flow {t} {dt}");
                meta.description = reader.header().to_string();
                meta.authors = vec!["Giorgi Kazakov".into(), "Roman Sidorov".into()];
                meta.version = Some(Version::new(0, 0, index));
                meta.date = Some(NaiveDateTime::parse_from_str(&header.acq_date, DBYHMP)?.date());
                let file = File::create(path.with_extension("ipc"))?;
                MetaDataFrame::new(meta, data_frame).write(file)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        // let s = "20 Aug 24 02:26 am";
        let s = "07 Aug 24  08:00 pm";
        let t = NaiveDateTime::parse_from_str(s, "%d %b %y %I:%M %P");
        println!("t: {t:?}");
    }
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
