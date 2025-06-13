use std::{collections::HashMap, str::FromStr};

use log::debug;
use strum::IntoEnumIterator;
use tokio::task::JoinHandle;

use crate::{
    error::*,
    financial::*,
    master::{Master, MasterAnalysis},
    ticker::Ticker,
    utils,
};

pub struct EvaluateOptions {
    pub masters: Vec<String>,
}

pub async fn run(ticker: &str, options: &EvaluateOptions) -> InvmstResult<()> {
    let ticker = Ticker::from_str(ticker)?;
    debug!("{ticker:?}");

    let stock_info = get_stock_info(&ticker).await?;
    debug!("{stock_info:?}");

    let mut trailing_stock_metrics = vec![];
    let mut fiscal_quarter = utils::datetime::prev_fiscal_quarter(None);
    for _ in 0..4 {
        let stock_metrics = get_fiscal_stock_metrics(&ticker, Some(fiscal_quarter.clone())).await?;
        trailing_stock_metrics.push(stock_metrics);

        fiscal_quarter = fiscal_quarter.prev();
    }
    debug!("{trailing_stock_metrics:?}");

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

    let mut handles: HashMap<Master, JoinHandle<InvmstResult<MasterAnalysis>>> = HashMap::new();
    for master in masters {
        let stock_info = stock_info.clone();
        let trailing_stock_metrics = trailing_stock_metrics.clone();

        let handle =
            tokio::spawn(async move { master.analyze(&stock_info, &trailing_stock_metrics).await });
        handles.insert(master, handle);
    }

    for (master, handle) in handles {
        let result = handle.await?;
        println!("[{master:?}] {result:?}");
    }

    Ok(())
}
