use chrono::{Duration, Local, NaiveDate};

use crate::{data::stock::*, error::*, financial::stock::*, ticker::Ticker, utils::datetime::*};

pub mod stock;

#[derive(Debug, PartialEq, strum::Display, strum::EnumIter, strum::EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Prospect {
    Bullish,
    Bearish,
    Neutral,
}

pub async fn get_stock_events(
    ticker: &Ticker,
    date: Option<&NaiveDate>,
    backward_days: i64,
) -> InvmstResult<StockEvents> {
    let date_end = date.map(|v| *v).unwrap_or(Local::now().date_naive());
    let date_start = date_end - Duration::days(backward_days);

    let dividends = fetch_stock_dividends(ticker, &date_start, &date_end).await?;

    Ok(StockEvents { dividends })
}

pub async fn get_stock_fiscal_metrics(
    ticker: &Ticker,
    quater: Option<FiscalQuarter>,
) -> InvmstResult<StockFiscalMetrics> {
    let fiscal_quater = quater.unwrap_or_else(|| prev_fiscal_quarter(None));
    let financial_summary = fetch_stock_financial_summary(ticker, &fiscal_quater).await?;

    Ok((fiscal_quater, StockMetrics { financial_summary }))
}

pub async fn get_stock_info(ticker: &Ticker) -> InvmstResult<StockInfo> {
    fetch_stock_info(ticker).await
}
