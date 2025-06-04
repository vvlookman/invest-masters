use std::{fs::create_dir_all, path::Path, str::FromStr};

use dashmap::DashMap;
use strum::IntoEnumIterator;

use crate::{
    APP_DATA_DIR,
    data::daily::{Daily, DailyData},
    ds,
    error::{InvmstError, InvmstResult},
    masters::Master,
};

pub struct EvaluateOptions {
    pub masters: Vec<String>,
    pub tickers: Vec<String>,
}

pub async fn run(data_dir: Option<&Path>, options: &EvaluateOptions) -> InvmstResult<()> {
    let data_dir = data_dir.unwrap_or(&APP_DATA_DIR);
    if data_dir.exists() {
        if !data_dir.is_dir() {
            return Err(InvmstError::Invalid(
                "DATA_DIR_IS_NOT_DIR",
                format!("'{data_dir:?}' is not a directory"),
            ));
        }
    } else {
        create_dir_all(data_dir)
            .unwrap_or_else(|_| panic!("Unable to create directory '{data_dir:?}'"));
    }

    let ticker_prices: DashMap<String, DailyData> = DashMap::new();
    if options.tickers.is_empty() {
        return Err(InvmstError::Required(
            "TICKER_REQUIRED",
            "No ticker is specified".to_string(),
        ));
    } else {
        for ticker in &options.tickers {
            let price_data = ds::fetch_daily_price_data(ticker).await?;
            println!("{:?}", price_data.get_date_max());
            println!("{:?}", price_data.get_date_min());
            ticker_prices.insert(ticker.to_string(), price_data);
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
