use chrono::{Local, NaiveDate};

use crate::{
    ds::utils::fetch_financial_summary, error::InvmstResult, ticker::Ticker,
    utils::datetime::prev_fiscal_quarter,
};

#[derive(Debug)]
pub struct MarketMetrics {
    pub price: f64,
}

#[derive(Debug)]
pub struct StockMetrics {
    pub price: f64,
}

pub async fn get_market_metrics() -> InvmstResult<MarketMetrics> {
    Ok(MarketMetrics { price: 0.0 })
}

pub async fn get_stock_metrics(
    ticker: &Ticker,
    date: Option<NaiveDate>,
) -> InvmstResult<StockMetrics> {
    let date = date.unwrap_or(Local::now().date_naive());
    let fiscal_quater = prev_fiscal_quarter(&date);

    let financial_summary = fetch_financial_summary(ticker, &fiscal_quater).await?;

    println!("{financial_summary:?}");

    Ok(StockMetrics { price: 0.0 })
}

mod aktools;
mod utils;
