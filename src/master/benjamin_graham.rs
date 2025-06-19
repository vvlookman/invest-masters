use log::debug;
use serde_json::json;

use crate::{
    data::stock::StockInfo,
    error::InvmstError,
    llm,
    llm::{ChatCompletionOptions, ChatMessage, Role},
    master::{
        AnalysisDraft, InvmstResult, MASTER_ANALYSIS_JSON_PROMPT, MasterAnalysis,
        MasterAnalyzeOptions, StockDailyData, StockEvents, StockFiscalMetricset,
    },
    utils,
    utils::datetime::Quarter,
};

pub async fn analyze(
    stock_info: &StockInfo,
    stock_events: &StockEvents,
    stock_daily_data: &StockDailyData,
    stock_fiscal_metricsets: &[StockFiscalMetricset],
    options: &MasterAnalyzeOptions,
) -> InvmstResult<MasterAnalysis> {
    if stock_fiscal_metricsets.is_empty() {
        return Err(InvmstError::NoData(
            "NO_STOCK_METRICS",
            "No stock metrics data".to_string(),
        ));
    }

    let analysis_core_valuation =
        analyze_core_valuation(stock_daily_data, stock_fiscal_metricsets).await?;
    let analysis_financial_health = analyze_financial_health(stock_fiscal_metricsets).await?;
    let analysis_earnings_stability = analyze_earnings_stability(stock_fiscal_metricsets).await?;
    let analysis_dividend = analyze_dividend(stock_events, options.backward_days).await?;

    let data_json = json!({
        "basic_information": stock_info,
        "analysis_core_valuation": analysis_core_valuation,
        "analysis_financial_health": analysis_financial_health,
        "analysis_earnings_stability": analysis_earnings_stability,
        "analysis_dividend": analysis_dividend,
    });
    debug!("[Benjamin Graham Data] {data_json}");

    let prompt = format!(
        r#"
基于下面的数据，使用我的投资分析方法评估投资对象，结果以标准的 JSON 对象格式返回：
```
{data_json}
```

{MASTER_ANALYSIS_JSON_PROMPT}
"#
    );

    let messages: Vec<ChatMessage> = vec![
        ChatMessage {
            role: Role::System,
            content: LLM_SYSTEM.to_string(),
            reasoning: None,
        },
        ChatMessage {
            role: Role::User,
            content: prompt.to_string(),
            reasoning: None,
        },
    ];

    let bot_message = llm::chat_completion(&messages, &ChatCompletionOptions::default()).await?;
    debug!("{bot_message:?}");

    let json_str = utils::markdown::extract_code_block(&bot_message.content);
    let analysis = MasterAnalysis::from_json(&json_str)?;

    Ok(analysis)
}

async fn analyze_core_valuation(
    stock_daily_data: &StockDailyData,
    stock_fiscal_metricsets: &[StockFiscalMetricset],
) -> InvmstResult<AnalysisDraft> {
    if stock_fiscal_metricsets.len() < 1 {
        return Ok(AnalysisDraft {
            score: None,
            assessments: vec![
                "Insufficient historical data for core valuation analysis".to_string(),
            ],
        });
    }

    let mut sum_scores: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;
    let mut assessments: Vec<String> = vec![];

    let latest_stock_fiscal_metricsets = stock_fiscal_metricsets.first().unwrap();
    let (fiscal_quater, stock_metrics) = latest_stock_fiscal_metricsets;

    let fiscal_date_str = format!(
        "{}{}",
        fiscal_quater.year,
        match fiscal_quater.quarter {
            Quarter::Q1 => "0331",
            Quarter::Q2 => "0630",
            Quarter::Q3 => "0930",
            Quarter::Q4 => "1231",
        }
    );
    if let Some(date) = utils::datetime::date_from_str(&fiscal_date_str) {
        let price = stock_daily_data
            .daily_valuations
            .get_latest_value::<f64>(&date, "price");
        let market_cap = stock_daily_data
            .daily_valuations
            .get_latest_value::<f64>(&date, "market_cap");

        // 如果净流动资产高于市值，这可能表明公司被低估，存在安全边际
        if let (Some(net_assets), Some(market_cap)) =
            (stock_metrics.financial_summary.net_assets, market_cap)
        {
            let weight = 1.0;
            if net_assets > market_cap * 1.3 {
                sum_scores += weight;
                assessments.push("Undervalued price".to_string());
            } else if net_assets > market_cap {
                sum_scores += weight / 2.0;
                assessments.push("Acceptable price".to_string());
            } else {
                assessments.push("Overvalued price".to_string());
            }
            sum_weights += weight;
        }

        // 格雷厄姆数字（合理股价）= sqrt( 22.5 × 每股收益 × 每股账面价值 )
        if let (Some(price), Some(earnings_per_share), Some(book_value_per_share)) = (
            price,
            stock_metrics.financial_summary.earnings_per_share,
            stock_metrics.financial_summary.book_value_per_share,
        ) {
            let graham_number = (22.5 * earnings_per_share * book_value_per_share).sqrt();
            let margin_of_safety = (graham_number - price) / price;

            let weight = 1.0;
            if margin_of_safety > 0.5 {
                sum_scores += weight;
                assessments.push("Hight margin of safety".to_string());
            } else if margin_of_safety > 0.2 {
                sum_scores += weight / 2.0;
                assessments.push("Acceptable margin of safety".to_string());
            } else {
                assessments.push("Low margin of safety".to_string());
            }
            sum_weights += weight;
        }
    }

    let score = if sum_weights > 0.0 {
        Some(sum_scores / sum_weights)
    } else {
        None
    };

    if let Some(score) = score {
        if score >= 0.75 {
            assessments.push("Have good financial health".to_string());
        } else {
            assessments.push("Not have good financial health".to_string());
        }
    }

    Ok(AnalysisDraft { score, assessments })
}

async fn analyze_dividend(
    stock_events: &StockEvents,
    backward_days: i64,
) -> InvmstResult<AnalysisDraft> {
    let mut sum_scores: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;
    let mut assessments: Vec<String> = vec![];

    // 股息分红
    {
        if backward_days > 180 {
            let weight = 1.0;
            if (backward_days as f64 / stock_events.dividends.len() as f64) < 365.0 {
                sum_scores += weight;
                assessments.push("Dividends used to be paid regularly".to_string());
            } else {
                assessments.push("Dividends have not been paid regularly".to_string());
            }
            sum_weights += weight;
        }
    }

    let score = if sum_weights > 0.0 {
        Some(sum_scores / sum_weights)
    } else {
        None
    };

    if let Some(score) = score {
        if score >= 0.75 {
            assessments.push("Have good dividend records".to_string());
        } else {
            assessments.push("Not have good dividend records".to_string());
        }
    }

    Ok(AnalysisDraft { score, assessments })
}

async fn analyze_earnings_stability(
    stock_fiscal_metricsets: &[StockFiscalMetricset],
) -> InvmstResult<AnalysisDraft> {
    if stock_fiscal_metricsets.len() < 8 {
        return Ok(AnalysisDraft {
            score: None,
            assessments: vec![
                "Insufficient historical data for earning stability analysis".to_string(),
            ],
        });
    }

    let mut sum_scores: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;
    let mut assessments: Vec<String> = vec![];

    // 每股收益持续增长
    {
        let mut growth_rates: Vec<f64> = vec![];
        for i in 0..stock_fiscal_metricsets.len() - 1 {
            if let (Some(earnings_per_share_current), Some(earnings_per_share_prev)) = (
                stock_fiscal_metricsets[i]
                    .1
                    .financial_summary
                    .earnings_per_share,
                stock_fiscal_metricsets[i + 1]
                    .1
                    .financial_summary
                    .earnings_per_share,
            ) {
                growth_rates.push(
                    (earnings_per_share_current - earnings_per_share_prev)
                        / earnings_per_share_prev,
                );
            }
        }

        let weight = 1.0;
        let growth_rate_avg = growth_rates.iter().sum::<f64>() / growth_rates.len() as f64;
        if growth_rate_avg > 0.0 {
            sum_scores += weight;
            assessments.push(format!(
                "Average earning per share growth rate is positive value: {growth_rate_avg}"
            ));
        } else {
            assessments.push(format!(
                "Average earning per share growth rate is negative value: {growth_rate_avg}"
            ));
        }
        sum_weights += weight;
    }

    let score = if sum_weights > 0.0 {
        Some(sum_scores / sum_weights)
    } else {
        None
    };

    if let Some(score) = score {
        if score >= 0.75 {
            assessments.push("Strong earning stability".to_string());
        } else {
            assessments.push("Weak earning stability".to_string());
        }
    }

    Ok(AnalysisDraft { score, assessments })
}

async fn analyze_financial_health(
    stock_fiscal_metricsets: &[StockFiscalMetricset],
) -> InvmstResult<AnalysisDraft> {
    if stock_fiscal_metricsets.len() < 1 {
        return Ok(AnalysisDraft {
            score: None,
            assessments: vec![
                "Insufficient historical data for financial health analysis".to_string(),
            ],
        });
    }

    let mut sum_scores: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;
    let mut assessments: Vec<String> = vec![];

    let latest_stock_fiscal_metricsets = stock_fiscal_metricsets.first().unwrap();
    let (_, stock_metrics) = latest_stock_fiscal_metricsets;

    // 流动比率
    if let Some(current_ratio) = stock_metrics.financial_summary.current_ratio {
        let weight = 1.0;
        if current_ratio >= 2.0 {
            sum_scores += weight;
            assessments.push("High current ratio indicates strong liquidity".to_string());
        } else if current_ratio >= 1.5 {
            sum_scores += weight / 2.0;
            assessments.push("Acceptable liquidity".to_string());
        } else {
            assessments.push("Low current ratio indicates weak liquidity".to_string());
        }
        sum_weights += weight;
    }

    // 资产负债率
    if let Some(debt_to_assets) = stock_metrics.financial_summary.debt_to_assets {
        let weight = 1.0;
        if debt_to_assets <= 0.5 {
            sum_scores += weight;
            assessments.push("Hight debt ratio".to_string());
        } else if debt_to_assets <= 0.8 {
            sum_scores += weight / 2.0;
            assessments.push("Acceptable debt ratio".to_string());
        } else {
            assessments.push("Low debt ratio".to_string());
        }
        sum_weights += weight;
    }

    let score = if sum_weights > 0.0 {
        Some(sum_scores / sum_weights)
    } else {
        None
    };

    if let Some(score) = score {
        if score >= 0.75 {
            assessments.push("Have good financial health".to_string());
        } else {
            assessments.push("Not have good financial health".to_string());
        }
    }

    Ok(AnalysisDraft { score, assessments })
}

static LLM_SYSTEM: &str = r#"
我是本杰明·格雷厄姆（Benjamin Graham），下面是我的投资分析方法论：

## 核心原则
1. 坚持安全边际原则，以低于内在价值的价格购买（例如：使用格雷厄姆数字、净流动资产价值）
2. 强调公司的财务健康（低杠杆率、充足的流动资产）
3. 倾向于多年稳定的盈利表现
4. 考虑股息记录以增加安全性
5. 避免投机性或高增长假设，专注于经过验证的指标

## 评估方法
1. 关注对决策影响最大的关键估值指标（格雷厄姆数字、净流动资产价值、市盈率等）
2. 关注反应财务健康的指标（流动比率、债务水平等）
3. 在一段较长的时间上检视盈利的稳定性
4. 查看股息记录
5. 将各项指标与格雷厄姆的具体阈值进行比较

## 评分等级（百分制）
- 80-100：卓越企业，价格诱人
- 60-79：良好企业，估值合理
- 40-59：信号混杂，需更多信息或更优价格
- 20-39：数据不足，无法做出评估
- 0-19：劣质企业或严重高估
"#;
