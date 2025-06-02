use std::{fs::read_dir, path::Path, str::FromStr};

use dashmap::DashMap;
use strum::IntoEnumIterator;

use crate::{
    DEFAULT_DATA_DIR,
    data::daily::{Daily, DailyData},
    error::{InvmstError, InvmstResult},
    masters::Master,
    utils,
};

pub struct EvaluateOptions {
    pub masters: Vec<String>,
    pub tickers: Vec<String>,
}

pub async fn run(data_dir: Option<&Path>, options: &EvaluateOptions) -> InvmstResult<()> {
    let data_dir = data_dir.unwrap_or(&DEFAULT_DATA_DIR);
    if !data_dir.is_dir() {
        return Err(InvmstError::Invalid(
            "DATA_DIR_INVALID",
            format!("{data_dir:?} is not a directory"),
        ));
    }

    let ticker_prices: DashMap<String, DailyData> = DashMap::new();
    if options.tickers.is_empty() {
        return Err(InvmstError::Required(
            "TICKER_REQUIRED",
            "No ticker is specified".to_string(),
        ));
    } else {
        for ticker in &options.tickers {
            if let Ok(entries) = read_dir(data_dir) {
                for entry in entries {
                    let entry_path = entry?.path();
                    let filename = utils::fs::extract_filename_from_path(&entry_path);
                    if filename.starts_with(ticker) {
                        let price_data = DailyData::from_csv(&entry_path, "日期")?;
                        println!("{:?}", price_data.get_date_max());
                        println!("{:?}", price_data.get_date_min());
                        ticker_prices.insert(ticker.to_string(), price_data);
                        break;
                    }
                }
            }
        }
    }
    println!("{ticker_prices:?}");

    let mut masters: Vec<Master> = vec![];
    if options.masters.is_empty() {
        // Use all masters if no master is specified in options
        masters = Master::iter().collect();
    } else {
        for master_str in &options.masters {
            match Master::from_str(master_str) {
                Ok(master) => {
                    masters.push(master);
                }
                Err(_) => {
                    return Err(InvmstError::NotExists(
                        "MASTER_NOT_EXISTS",
                        format!("Master '{master_str}' not exists"),
                    ));
                }
            }
        }
    }

    println!("{masters:?}");

    Ok(())
}
