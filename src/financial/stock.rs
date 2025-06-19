use std::collections::HashMap;

use chrono::NaiveDate;
use serde_json::json;

use crate::{
    data::{daily::*, stock::*},
    ds::aktools,
    error::*,
    ticker::Ticker,
    utils::datetime::*,
};

pub async fn fetch_stock_daily_valuations(ticker: &Ticker) -> InvmstResult<DailyDataset> {
    match ticker.exchange.as_str() {
        "SSE" | "SZSE" => {
            let json = aktools::call_public_api(
                "/stock_value_em",
                &json!({
                    "symbol": ticker.symbol,
                }),
            )
            .await?;

            let mut value_field_names: HashMap<String, String> = HashMap::new();
            value_field_names.insert("price".to_string(), "当日收盘价".to_string());
            value_field_names.insert("market_cap".to_string(), "总市值".to_string());

            DailyDataset::from_json(&json, "数据日期", &value_field_names)
        }
        _ => Err(InvmstError::Invalid(
            "EXCHANGE_NOT_SUPPORTED",
            format!("Not yet supported exchange '{}'", ticker.exchange),
        )),
    }
}

pub async fn fetch_stock_dividends(
    ticker: &Ticker,
    date_start: &NaiveDate,
    date_end: &NaiveDate,
) -> InvmstResult<Vec<StockDividend>> {
    match ticker.exchange.as_str() {
        "SSE" | "SZSE" => {
            let mut result = vec![];

            {
                let json = aktools::call_public_api(
                    "/stock_fhps_detail_em",
                    &json!({
                        "symbol": ticker.symbol,
                    }),
                )
                .await?;

                if let Some(array) = json.as_array() {
                    for item in array {
                        let date_announce =
                            date_from_str(item["预案公告日"].as_str().unwrap_or_default());
                        let date_record =
                            date_from_str(item["股权登记日"].as_str().unwrap_or_default());
                        let dividend_yield = item["现金分红-股息率"].as_f64();

                        if let (Some(date_announce), Some(date_record), Some(dividend_yield)) =
                            (date_announce, date_record, dividend_yield)
                        {
                            if date_announce >= *date_start && date_announce <= *date_end {
                                result.push(StockDividend {
                                    date_announce,
                                    date_record,
                                    dividend_yield,
                                });
                            }
                        }
                    }
                }
            }

            Ok(result)
        }
        _ => Err(InvmstError::Invalid(
            "EXCHANGE_NOT_SUPPORTED",
            format!("Not yet supported exchange '{}'", ticker.exchange),
        )),
    }
}

pub async fn fetch_stock_financial_summary(
    ticker: &Ticker,
    fiscal_quater: &FiscalQuarter,
) -> InvmstResult<StockFinancialSummary> {
    match ticker.exchange.as_str() {
        "SSE" | "SZSE" => {
            let mut result = StockFinancialSummary::default();

            {
                let json = aktools::call_public_api(
                    "/stock_financial_abstract",
                    &json!({
                        "symbol": ticker.symbol,
                    }),
                )
                .await?;

                let quarter_key = format!(
                    "{}{}",
                    fiscal_quater.year,
                    match fiscal_quater.quarter {
                        Quarter::Q1 => "0331",
                        Quarter::Q2 => "0630",
                        Quarter::Q3 => "0930",
                        Quarter::Q4 => "1231",
                    }
                );

                if let Some(array) = json.as_array() {
                    for item in array {
                        match item["指标"].as_str().unwrap_or_default() {
                            "总资产周转率" => {
                                result.asset_turnover = item[&quarter_key].as_f64();
                            }
                            "每股净资产" => {
                                result.book_value_per_share = item[&quarter_key].as_f64();
                            }
                            "现金比率" => {
                                result.cash_ratio = item[&quarter_key].as_f64();
                            }
                            "成本费用利润率" => {
                                result.cost_of_profit =
                                    item[&quarter_key].as_f64().map(|v| v / 100.0);
                            }
                            "销售成本率" => {
                                result.cost_of_revenue =
                                    item[&quarter_key].as_f64().map(|v| v / 100.0);
                            }
                            "成本费用率" => {
                                result.cost_to_revenue =
                                    item[&quarter_key].as_f64().map(|v| v / 100.0);
                            }
                            "流动比率" => {
                                result.current_ratio = item[&quarter_key].as_f64();
                            }
                            "总资产周转天数" => {
                                result.days_asset_outstanding = item[&quarter_key].as_f64();
                            }
                            "存货周转天数" => {
                                result.days_inventory_outstanding = item[&quarter_key].as_f64();
                            }
                            "应收账款周转天数" => {
                                result.days_sales_outstanding = item[&quarter_key].as_f64();
                            }
                            "资产负债率" => {
                                result.debt_to_assets =
                                    item[&quarter_key].as_f64().map(|v| v / 100.0);
                            }
                            "产权比率" => {
                                result.debt_to_equity =
                                    item[&quarter_key].as_f64().map(|v| v / 100.0);
                            }
                            "基本每股收益" => {
                                result.earnings_per_share = item[&quarter_key].as_f64();
                            }
                            "每股现金流" => {
                                result.free_cash_flow_per_share = item[&quarter_key].as_f64();
                            }
                            "商誉" => {
                                result.goodwill = item[&quarter_key].as_f64();
                            }
                            "毛利率" => {
                                result.gross_margin =
                                    item[&quarter_key].as_f64().map(|v| v / 100.0);
                            }
                            "存货周转率" => {
                                result.inventory_turnover = item[&quarter_key].as_f64();
                            }
                            "股东权益合计(净资产)" => {
                                result.net_assets = item[&quarter_key].as_f64();
                            }
                            "销售净利率" => {
                                result.net_margin = item[&quarter_key].as_f64().map(|v| v / 100.0);
                            }
                            "净利润" => {
                                result.net_profit = item[&quarter_key].as_f64();
                            }
                            "经营现金流量净额" => {
                                result.operating_cash_flow = item[&quarter_key].as_f64();
                            }
                            "营业成本" => {
                                result.operating_costs = item[&quarter_key].as_f64();
                            }
                            "营业利润率" => {
                                result.operating_margin =
                                    item[&quarter_key].as_f64().map(|v| v / 100.0);
                            }
                            "营业总收入" => {
                                result.operating_revenue = item[&quarter_key].as_f64();
                            }
                            "速动比率" => {
                                result.quick_ratio = item[&quarter_key].as_f64();
                            }
                            "应收账款周转率" => {
                                result.receivables_turnover = item[&quarter_key].as_f64();
                            }
                            "总资产报酬率(ROA)" => {
                                result.return_on_assets =
                                    item[&quarter_key].as_f64().map(|v| v / 100.0);
                            }
                            "净资产收益率(ROE)" => {
                                result.return_on_equity =
                                    item[&quarter_key].as_f64().map(|v| v / 100.0);
                            }
                            "投入资本回报率" => {
                                result.return_on_invested_capital =
                                    item[&quarter_key].as_f64().map(|v| v / 100.0);
                            }
                            "营业总收入增长率" => {
                                result.revenue_growth =
                                    item[&quarter_key].as_f64().map(|v| v / 100.0);
                            }
                            _ => {}
                        }
                    }
                }
            }

            Ok(result)
        }
        _ => Err(InvmstError::Invalid(
            "EXCHANGE_NOT_SUPPORTED",
            format!("Not yet supported exchange '{}'", ticker.exchange),
        )),
    }
}

pub async fn fetch_stock_info(ticker: &Ticker) -> InvmstResult<StockInfo> {
    match ticker.exchange.as_str() {
        "SSE" | "SZSE" => {
            let mut result = StockInfo::default();

            {
                let json = aktools::call_public_api(
                    "/stock_individual_info_em",
                    &json!({
                        "symbol": ticker.symbol,
                    }),
                )
                .await?;

                if let Some(array) = json.as_array() {
                    for item in array {
                        match item["item"].as_str().unwrap_or_default() {
                            "股票简称" => {
                                result.name = item["value"].as_str().map(|v| v.to_string());
                            }
                            "行业" => {
                                result.industry = item["value"].as_str().map(|v| v.to_string());
                            }
                            _ => {}
                        }
                    }
                }
            }

            Ok(result)
        }
        _ => Err(InvmstError::Invalid(
            "EXCHANGE_NOT_SUPPORTED",
            format!("Not yet supported exchange '{}'", ticker.exchange),
        )),
    }
}
