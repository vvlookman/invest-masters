use log::debug;
use serde_json::json;

use crate::{
    data::stock::StockInfo,
    error::InvmstError,
    financial::stock::StockValuationFieldName,
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
    _stock_events: &StockEvents,
    stock_daily_data: &StockDailyData,
    stock_fiscal_metricsets: &[StockFiscalMetricset],
    _options: &MasterAnalyzeOptions,
) -> InvmstResult<MasterAnalysis> {
    if stock_fiscal_metricsets.is_empty() {
        return Err(InvmstError::NoData(
            "NO_STOCK_METRICS",
            "No stock metrics data".to_string(),
        ));
    }

    let data_json = json!({
        "basic_information": stock_info,
        "analysis_fundamentals": analyze_fundamentals(stock_fiscal_metricsets).await?,
        "analysis_growth": analyze_growth(stock_fiscal_metricsets).await?,
        "analysis_valuation": analyze_valuation(stock_daily_data, stock_fiscal_metricsets).await?,
    });
    debug!("[Peter Lynch Data] {data_json}");

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
    debug!("[Peter Lynch LLM] {bot_message:?}");

    let json_str = utils::markdown::extract_code_block(&bot_message.content);
    let analysis = MasterAnalysis::from_json(&json_str)?;

    Ok(analysis)
}

async fn analyze_fundamentals(
    stock_fiscal_metricsets: &[StockFiscalMetricset],
) -> InvmstResult<AnalysisDraft> {
    if stock_fiscal_metricsets.len() < 1 {
        return Ok(AnalysisDraft {
            score: None,
            assessments: vec!["Insufficient historical data for fundamentals analysis".to_string()],
        });
    }

    let mut sum_scores: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;
    let mut assessments: Vec<String> = vec![];

    let latest_stock_fiscal_metricsets = stock_fiscal_metricsets.first().unwrap();
    let (_, stock_metrics) = latest_stock_fiscal_metricsets;

    // 利润率
    if let Some(operating_margin) = stock_metrics.financial_summary.operating_margin {
        let weight = 1.0;
        if operating_margin > 0.15 {
            sum_scores += weight;
            assessments.push(format!("Strong operating margin ({operating_margin})"));
        } else if operating_margin > 0.07 {
            sum_scores += weight / 2.0;
            assessments.push(format!("Acceptable operating margin ({operating_margin})"));
        } else {
            assessments.push(format!("Weak operating margin ({operating_margin})"));
        }
        sum_weights += weight;
    }

    // 长期偿债能力
    if let Some(debt_to_equity) = stock_metrics.financial_summary.debt_to_equity {
        let weight = 1.0;
        if debt_to_equity < 0.5 {
            sum_scores += weight;
            assessments.push(format!("Low debt to equity ({debt_to_equity})"));
        } else if debt_to_equity < 1.0 {
            sum_scores += weight / 2.0;
            assessments.push(format!("Acceptable debt to equity ({debt_to_equity})"));
        } else {
            assessments.push(format!("High debt to equity ({debt_to_equity})"));
        }
        sum_weights += weight;
    }

    // 现金流
    if let Some(free_cash_flow_per_share) = stock_metrics.financial_summary.free_cash_flow_per_share
    {
        let weight = 1.0;
        if free_cash_flow_per_share > 0.0 {
            sum_scores += weight;
            assessments.push(format!("Positive free cash flow"));
        } else {
            assessments.push(format!("No positive free cash flow"));
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
            assessments.push("Have good fundamentals".to_string());
        } else {
            assessments.push("Not have good fundamentals".to_string());
        }
    }

    Ok(AnalysisDraft { score, assessments })
}

async fn analyze_growth(
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

    // 收入持续增长
    {
        let mut growth_rates: Vec<f64> = vec![];
        for i in 0..stock_fiscal_metricsets.len() - 1 {
            if let (Some(operating_revenue_current), Some(operating_revenue_prev)) = (
                stock_fiscal_metricsets[i]
                    .1
                    .financial_summary
                    .operating_revenue,
                stock_fiscal_metricsets[i + 1]
                    .1
                    .financial_summary
                    .operating_revenue,
            ) {
                growth_rates.push(
                    (operating_revenue_current - operating_revenue_prev) / operating_revenue_prev,
                );
            }
        }

        let weight = 1.0;
        let growth_rate_avg = growth_rates.iter().sum::<f64>() / growth_rates.len() as f64;
        if growth_rate_avg > 0.0 {
            sum_scores += weight;
            assessments.push(format!(
                "Revenue growth rate is positive value: {growth_rate_avg}"
            ));
        } else {
            assessments.push(format!(
                "Revenue growth rate is negative value: {growth_rate_avg}"
            ));
        }
        sum_weights += weight;
    }

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

async fn analyze_valuation(
    stock_daily_data: &StockDailyData,
    stock_fiscal_metricsets: &[StockFiscalMetricset],
) -> InvmstResult<AnalysisDraft> {
    if stock_fiscal_metricsets.len() < 1 {
        return Ok(AnalysisDraft {
            score: None,
            assessments: vec!["Insufficient historical data for valuation analysis".to_string()],
        });
    }

    let mut sum_scores: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;
    let mut assessments: Vec<String> = vec![];

    let latest_stock_fiscal_metricsets = stock_fiscal_metricsets.first().unwrap();
    let (fiscal_quater, _) = latest_stock_fiscal_metricsets;

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
        let pe = stock_daily_data
            .daily_valuations
            .get_latest_value::<f64>(&date, &StockValuationFieldName::Pe.to_string());
        let peg = stock_daily_data
            .daily_valuations
            .get_latest_value::<f64>(&date, &StockValuationFieldName::Peg.to_string());

        if let Some(pe) = pe {
            let weight = 1.0;
            if pe < 15.0 {
                sum_scores += weight;
                assessments.push("Good P/E".to_string());
            } else if pe < 25.0 {
                sum_scores += weight / 2.0;
                assessments.push("Acceptable P/E".to_string());
            } else {
                assessments.push("Unacceptable P/E".to_string());
            }
            sum_weights += weight;
        }

        if let Some(peg) = peg {
            let weight = 1.0;
            if peg < 1.0 {
                sum_scores += weight;
                assessments.push("Good PEG".to_string());
            } else if peg < 2.0 {
                sum_scores += weight / 2.0;
                assessments.push("Acceptable PEG".to_string());
            } else {
                assessments.push("Unacceptable PEG".to_string());
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

static LLM_SYSTEM: &str = r#"
我是彼得·林奇（Peter Lynch），下面是我的投资分析方法论：

## 核心原则
1. 强调投资于易于理解的业务，这些业务可能是在日常生活中发现的
2. 关注合理价格下的增长（GARP），以市盈率与增长比率（PEG）作为主要指标
3. 寻找那些能够显著增长盈利和股价的公司（十倍股）
4. 更倾向于稳定的收入/盈利增长，不太关心短期波动
5. 警惕危险的杠杆，避免高负债

## 评估方法
1. 判断业务是否是易于理解的
2. 检视主要指标，如市盈率与增长比率（PEG）
3. 在一段较长的时间上检视盈利的稳定性
4. 是否有可控的负债水平

## 评分等级（百分制）
- 80-100：卓越企业，价格诱人
- 60-79：良好企业，估值合理
- 40-59：信号混杂，需更多信息或更优价格
- 20-39：数据不足，无法做出评估
- 0-19：劣质企业或严重高估
"#;
