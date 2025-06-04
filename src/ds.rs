use std::collections::HashMap;

use crate::{data::daily::DailyData, error::InvmstResult};

#[derive(Debug)]
pub struct StockMetrics {
    pub price: f64,
}

pub async fn stock_metrics(ticker: &str) -> InvmstResult<StockMetrics> {
    let daily_price_data = fetch_daily_price_data(ticker).await?;

    Ok(StockMetrics { price: 0.0 })
}

async fn fetch_daily_price_data(ticker: &str) -> InvmstResult<DailyData> {
    let mut price_aktools_query: HashMap<String, String> = HashMap::new();
    price_aktools_query.insert("adjust".to_string(), "hfq".to_string());
    price_aktools_query.insert("period".to_string(), "daily".to_string());
    price_aktools_query.insert("symbol".to_string(), ticker.to_string());

    let price_aktools_json =
        aktools::call_public_api("/stock_zh_a_hist", &price_aktools_query).await?;

    DailyData::from_json(&price_aktools_json, "日期")
}

mod aktools;
