use chrono::{Local, NaiveDate};

use crate::{
    ds::utils::{FinancialSummary, fetch_financial_summary},
    error::InvmstResult,
    ticker::Ticker,
    utils::datetime::{FiscalQuarter, prev_fiscal_quarter},
};

pub type FiscalStockMetrics = (FiscalQuarter, StockMetrics);

#[derive(Clone, Debug)]
pub struct MarketMetrics {
    pub price: f64,
}

#[derive(Clone, Debug)]
pub struct StockMetrics {
    pub financial_summary: FinancialSummary,
}

pub async fn get_market_metrics() -> InvmstResult<MarketMetrics> {
    Ok(MarketMetrics { price: 0.0 })
}

pub async fn get_stock_metrics(
    ticker: &Ticker,
    date: Option<NaiveDate>,
) -> InvmstResult<FiscalStockMetrics> {
    let date = date.unwrap_or(Local::now().date_naive());
    let fiscal_quater = prev_fiscal_quarter(&date);

    let financial_summary = fetch_financial_summary(ticker, &fiscal_quater).await?;
    println!("{financial_summary:?}");

    Ok((fiscal_quater, StockMetrics { financial_summary }))
}

mod aktools;
mod utils;
