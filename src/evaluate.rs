use std::{collections::HashMap, str::FromStr};

use chrono::NaiveDate;
use log::debug;
use strum::IntoEnumIterator;
use tokio::task::JoinHandle;

use crate::{
    error::*,
    financial::*,
    master::{Master, MasterAnalysis, MasterAnalyzeOptions},
    ticker::Ticker,
    utils,
};

pub struct EvaluateOptions {
    pub backward_days: i64,
    pub date: Option<NaiveDate>,
    pub masters: Vec<String>,
}

pub struct Evaluation {
    pub master_analyses: HashMap<Master, MasterAnalysis>,
}

pub async fn run(ticker: &str, options: &EvaluateOptions) -> InvmstResult<Evaluation> {
    let ticker = Ticker::from_str(ticker)?;
    debug!("{ticker:?}");

    let stock_info = get_stock_info(&ticker).await?;
    debug!("{stock_info:?}");

    let stock_events =
        get_stock_events(&ticker, options.date.as_ref(), options.backward_days).await?;
    debug!("{stock_events:?}");

    let mut stock_metrics = vec![];
    let fiscal_count = options.backward_days / 91;
    let mut fiscal_quarter = utils::datetime::prev_fiscal_quarter(options.date.as_ref());
    for _ in 0..fiscal_count {
        let stock_fiscal_metrics =
            get_stock_fiscal_metrics(&ticker, Some(fiscal_quarter.clone())).await?;
        stock_metrics.push(stock_fiscal_metrics);

        fiscal_quarter = fiscal_quarter.prev();
    }
    debug!("{stock_metrics:?}");

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
        let options = MasterAnalyzeOptions {
            backward_days: options.backward_days,
            date: options.date.clone(),
        };

        let stock_info = stock_info.clone();
        let stock_events = stock_events.clone();
        let stock_metrics = stock_metrics.clone();

        let handle = tokio::spawn(async move {
            master
                .analyze(&stock_info, &stock_events, &stock_metrics, &options)
                .await
        });
        handles.insert(master, handle);
    }

    let mut master_analyses: HashMap<Master, MasterAnalysis> = HashMap::new();
    for (master, handle) in handles {
        let result = handle.await??;
        master_analyses.insert(master, result);
    }

    Ok(Evaluation { master_analyses })
}
