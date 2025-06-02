use std::path::Path;

use strum::IntoEnumIterator;

use crate::{error::InvmstResult, evaluate, masters::Master};

pub type EvaluateOptions = evaluate::EvaluateOptions;

pub async fn evaluate(data_dir: Option<&Path>, options: &EvaluateOptions) -> InvmstResult<()> {
    evaluate::run(data_dir, options).await
}

pub async fn masters() -> Vec<Master> {
    Master::iter().collect()
}
