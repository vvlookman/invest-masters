use serde::Serialize;

use crate::{ds::FiscalStockMetrics, error::InvmstResult, ticker::Ticker};

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    PartialEq,
    strum::Display,
    strum::EnumIter,
    strum::EnumMessage,
    strum::EnumString,
)]
#[strum(ascii_case_insensitive)]
pub enum Master {
    #[strum(
        message = "Benjamin Graham",
        serialize = "graham",
        serialize = "ben-graham"
    )]
    BenjaminGraham,

    #[strum(
        message = "Warren Buffett",
        serialize = "buffett",
        serialize = "warren-buffett"
    )]
    WarrenBuffett,
}

impl Master {
    pub async fn evaluate(
        &self,
        ticker: &Ticker,
        trailing_stock_metrics: &[FiscalStockMetrics],
    ) -> InvmstResult<()> {
        match self {
            Master::BenjaminGraham => todo!(),
            Master::WarrenBuffett => warren_buffett::evaluate(ticker, trailing_stock_metrics).await,
        }
    }
}

#[derive(Debug, Serialize)]
struct AnalysisDraft {
    score: Option<f64>,
    assessments: Vec<String>,
}

mod warren_buffett;
