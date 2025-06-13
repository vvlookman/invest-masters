use crate::{data::stock::*, error::*, financial::stock::*, ticker::Ticker, utils::datetime::*};

pub mod stock;

#[derive(Debug, PartialEq, strum::Display, strum::EnumIter, strum::EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Prospect {
    Bullish,
    Bearish,
    Neutral,
}

pub async fn get_stock_info(ticker: &Ticker) -> InvmstResult<StockInfo> {
    fetch_stock_info(ticker).await
}

pub async fn get_fiscal_stock_metrics(
    ticker: &Ticker,
    quater: Option<FiscalQuarter>,
) -> InvmstResult<FiscalStockMetrics> {
    let fiscal_quater = quater.unwrap_or_else(|| prev_fiscal_quarter(None));
    let financial_summary = fetch_stock_financial_summary(ticker, &fiscal_quater).await?;

    Ok((fiscal_quater, StockMetrics { financial_summary }))
}
