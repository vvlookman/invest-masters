use std::str::FromStr;

use strum::IntoEnumIterator;

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
