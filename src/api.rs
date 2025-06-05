use strum::IntoEnumIterator;

use crate::{error::InvmstResult, evaluate, master::Master};

pub type EvaluateOptions = evaluate::EvaluateOptions;

pub async fn evaluate(ticker: &str, options: &EvaluateOptions) -> InvmstResult<()> {
    evaluate::run(ticker, options).await
}

pub async fn masters() -> Vec<Master> {
    Master::iter().collect()
}
