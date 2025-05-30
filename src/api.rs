use strum::IntoEnumIterator;

use crate::{error::InvmstResult, evaluate, masters::Master};

pub type EvaluateOptions = evaluate::EvaluateOptions;

pub async fn evaluate(options: &EvaluateOptions) -> InvmstResult<()> {
    evaluate::run(options).await
}

pub async fn masters() -> Vec<Master> {
    Master::iter().collect()
}
