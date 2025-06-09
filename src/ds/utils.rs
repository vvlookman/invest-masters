use serde_json::json;

use crate::{
    data::daily::DailyData,
    ds::aktools,
    error::{InvmstError, InvmstResult},
    ticker::Ticker,
    utils::datetime::{FiscalQuarter, Quarter},
};

#[derive(Clone, Debug, Default)]
pub struct FinancialSummary {
    pub asset_turnover: Option<f64>,
    pub book_value_per_share: Option<f64>,
    pub cash_ratio: Option<f64>,
    pub cost_of_profit: Option<f64>,
    pub cost_of_revenue: Option<f64>,
    pub cost_to_revenue: Option<f64>,
    pub current_ratio: Option<f64>,
    pub days_asset_outstanding: Option<f64>,
    pub days_inventory_outstanding: Option<f64>,
    pub days_sales_outstanding: Option<f64>,
    pub debt_to_assets: Option<f64>,
    pub debt_to_equity: Option<f64>,
    pub earnings_per_share: Option<f64>,
    pub free_cash_flow_per_share: Option<f64>,
    pub goodwill: Option<f64>,
    pub gross_margin: Option<f64>,
    pub inventory_turnover: Option<f64>,
    pub net_assets: Option<f64>,
    pub net_margin: Option<f64>,
    pub net_profit: Option<f64>,
    pub operating_cash_flow: Option<f64>,
    pub operating_costs: Option<f64>,
    pub operating_margin: Option<f64>,
    pub operating_revenue: Option<f64>,
    pub quick_ratio: Option<f64>,
    pub receivables_turnover: Option<f64>,
    pub return_on_assets: Option<f64>,
    pub return_on_equity: Option<f64>,
    pub return_on_invested_capital: Option<f64>,
    pub revenue_growth: Option<f64>,
}

pub async fn fetch_financial_summary(
    ticker: &Ticker,
    fiscal_quater: &FiscalQuarter,
) -> InvmstResult<FinancialSummary> {
    match ticker.exchange.as_str() {
        "SSE" | "SZSE" => {
            let mut result = FinancialSummary::default();

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

pub async fn fetch_daily_price_data(ticker: &Ticker) -> InvmstResult<DailyData> {
    match ticker.exchange.as_str() {
        "SSE" | "SZSE" => {
            let json = aktools::call_public_api(
                "/stock_zh_a_hist",
                &json!({
                    "adjust": "hfq",
                    "period": "daily",
                    "symbol": ticker.symbol,
                }),
            )
            .await?;

            DailyData::from_json(&json, "日期")
        }
        _ => Err(InvmstError::Invalid(
            "EXCHANGE_NOT_SUPPORTED",
            format!("Not yet supported exchange '{}'", ticker.exchange),
        )),
    }
}
