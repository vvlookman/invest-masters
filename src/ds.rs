use std::collections::HashMap;

use serde_json::json;

use crate::{
    data::daily::DailyData,
    error::{InvmstError, InvmstResult},
    ticker::Ticker,
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

pub async fn get_stock_metrics(ticker: &Ticker) -> InvmstResult<StockMetrics> {
    let daily_price_data = fetch_daily_price_data(ticker).await?;

    Ok(StockMetrics { price: 0.0 })
}

async fn fetch_daily_price_data(ticker: &Ticker) -> InvmstResult<DailyData> {
    if let Some(exchange) = &ticker.exchange {
        match exchange.as_str() {
            "SSE" | "SZSE" => {
                let price_aktools_json = aktools::call_public_api(
                    "/stock_zh_a_hist",
                    &json!({
                        "adjust": "hfq",
                        "period": "daily",
                        "symbol": ticker.symbol,
                    }),
                )
                .await?;

                DailyData::from_json(&price_aktools_json, "日期")
            }
            _ => Err(InvmstError::Invalid(
                "EXCHANGE_NOT_SUPPORTED",
                format!("Not yet supported exchange '{exchange}'"),
            )),
        }
    } else {
        Err(InvmstError::Required(
            "EXCHANGE_REQUIRED",
            format!("Unable to determine exchange of '{}'", ticker.symbol),
        ))
    }
}

mod aktools;
