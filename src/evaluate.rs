use std::str::FromStr;

use crate::{
    error::{InvmstError, InvmstResult},
    masters::Master,
};

pub struct EvaluateOptions {
    pub masters: Vec<String>,
    pub tickers: Vec<String>,
}

pub async fn run(options: &EvaluateOptions) -> InvmstResult<()> {
    let mut masters: Vec<Master> = vec![];
    for master_str in &options.masters {
        match Master::from_str(master_str) {
            Ok(master) => {
                masters.push(master);
            }
            Err(_) => {
                return Err(InvmstError::NotExists(format!(
                    "Master '{master_str}' not exists"
                )));
            }
        }
    }
    println!("{:?}", masters);

    Ok(())
}
