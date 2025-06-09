use std::{collections::HashMap, str::FromStr};

use strum::IntoEnumIterator;
use tokio::task::JoinHandle;

use crate::{
    ds,
    error::{InvmstError, InvmstResult},
    master::Master,
    ticker::Ticker,
};

pub struct EvaluateOptions {
    pub masters: Vec<String>,
}

pub async fn run(ticker: &str, options: &EvaluateOptions) -> InvmstResult<()> {
    let ticker = Ticker::from_str(ticker)?;
    println!("{ticker:?}");

    let stock_metrics = ds::get_stock_metrics(&ticker, None).await?;
    println!("{stock_metrics:?}");

    let trailing_stock_metrics = vec![stock_metrics];

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

    let mut handles: HashMap<Master, JoinHandle<InvmstResult<()>>> = HashMap::new();
    for master in masters {
        let ticker = ticker.clone();
        let trailing_stock_metrics = trailing_stock_metrics.clone();

        let handle = tokio::spawn(async move {
            let _ = master.evaluate(&ticker, &trailing_stock_metrics).await;
            Ok(())
        });
        handles.insert(master, handle);
    }

    for (master, handle) in handles {
        let result = handle.await?;
        println!("[{master:?}] {result:?}");
    }

    Ok(())
}
