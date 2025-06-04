use std::{collections::HashMap, fs::create_dir_all, path::Path, str::FromStr};

use strum::IntoEnumIterator;
use tokio::task::JoinHandle;

use crate::{
    APP_DATA_DIR, ds,
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

    let mut metrics_map: HashMap<String, ds::StockMetrics> = HashMap::new();
    if options.tickers.is_empty() {
        return Err(InvmstError::Required(
            "TICKER_REQUIRED",
            "No ticker is specified".to_string(),
        ));
    } else {
        let mut handles: HashMap<String, JoinHandle<InvmstResult<ds::StockMetrics>>> =
            HashMap::new();

        for ticker in &options.tickers {
            let ticker = ticker.clone();

            handles.insert(
                ticker.clone(),
                tokio::spawn(async move { ds::stock_metrics(&ticker).await }),
            );
        }

        for (ticker, handle) in handles {
            let metrics = handle.await??;
            metrics_map.insert(ticker, metrics);
        }
    }
    println!("{metrics_map:?}");

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
