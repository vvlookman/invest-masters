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
};

pub async fn analyze(
    stock_info: &StockInfo,
    stock_events: &StockEvents,
    _stock_daily_data: &StockDailyData,
    stock_fiscal_metricsets: &[StockFiscalMetricset],
    options: &MasterAnalyzeOptions,
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
        "analysis_consistency": analyze_consistency(stock_fiscal_metricsets).await?,
        "analysis_moat": analyze_moat(stock_fiscal_metricsets).await?,
        "analysis_management": analyze_management(stock_events, options.backward_days).await?,
    });
    debug!("[Warren Buffett Data] {data_json}");

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
    debug!("[Warren Buffett LLM] {bot_message:?}");

    let json_str = utils::markdown::extract_code_block(&bot_message.content);
    let analysis = MasterAnalysis::from_json(&json_str)?;

    Ok(analysis)
}

async fn analyze_consistency(
    stock_fiscal_metricsets: &[StockFiscalMetricset],
) -> InvmstResult<AnalysisDraft> {
    if stock_fiscal_metricsets.len() < 4 {
        return Ok(AnalysisDraft {
            score: None,
            assessments: vec!["Insufficient historical data for consistency analysis".to_string()],
        });
    }

    let mut sum_scores: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;
    let mut assessments: Vec<String> = vec![];

    // 净利润持续增长
    {
        let mut growth_rates: Vec<f64> = vec![];
        for i in 0..stock_fiscal_metricsets.len() - 1 {
            if let (Some(net_profit_current), Some(net_profit_prev)) = (
                stock_fiscal_metricsets[i].1.financial_summary.net_profit,
                stock_fiscal_metricsets[i + 1]
                    .1
                    .financial_summary
                    .net_profit,
            ) {
                growth_rates.push((net_profit_current - net_profit_prev) / net_profit_prev);
            }
        }

        let weight = 1.0;
        let growth_rate_avg = growth_rates.iter().sum::<f64>() / growth_rates.len() as f64;
        if growth_rate_avg > 0.0 {
            sum_scores += weight;
            assessments.push(format!(
                "Average net profit growth rate is positive value: {growth_rate_avg}"
            ));
        } else {
            assessments.push(format!(
                "Average net profit growth rate is negative value: {growth_rate_avg}"
            ));
        }
        sum_weights += weight;
    }

    // 每股净资产持续增长
    {
        let mut growth_rates: Vec<f64> = vec![];
        for i in 0..stock_fiscal_metricsets.len() - 1 {
            if let (Some(book_value_per_share_current), Some(book_value_per_share_prev)) = (
                stock_fiscal_metricsets[i]
                    .1
                    .financial_summary
                    .book_value_per_share,
                stock_fiscal_metricsets[i + 1]
                    .1
                    .financial_summary
                    .book_value_per_share,
            ) {
                growth_rates.push(
                    (book_value_per_share_current - book_value_per_share_prev)
                        / book_value_per_share_prev,
                );
            }
        }

        let weight = 1.0;
        let growth_rate_avg = growth_rates.iter().sum::<f64>() / growth_rates.len() as f64;
        if growth_rate_avg > 0.0 {
            sum_scores += weight;
            assessments.push(format!(
                "Average book value per share growth rate is positive value: {growth_rate_avg}"
            ));
        } else {
            assessments.push(format!(
                "Average book value per share growth rate is negative value: {growth_rate_avg}"
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
            assessments.push("Consistent net profit growth".to_string());
        } else {
            assessments.push("Inconsistent net profit growth".to_string());
        }
    }

    Ok(AnalysisDraft { score, assessments })
}

async fn analyze_fundamentals(
    stock_fiscal_metricsets: &[StockFiscalMetricset],
) -> InvmstResult<AnalysisDraft> {
    if stock_fiscal_metricsets.is_empty() {
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

    // 资本回报率
    if let Some(return_on_equity) = stock_metrics.financial_summary.return_on_equity {
        let weight = 1.0;
        if return_on_equity > 0.15 {
            sum_scores += weight;
            assessments.push(format!("High return on equity ({return_on_equity})"));
        } else if return_on_equity > 0.07 {
            sum_scores += weight / 2.0;
            assessments.push(format!("Acceptable return on equity ({return_on_equity})"));
        } else {
            assessments.push(format!("Low return on equity ({return_on_equity})"));
        }
        sum_weights += weight;
    }

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

    // 短期偿债能力
    if let Some(current_ratio) = stock_metrics.financial_summary.current_ratio {
        let weight = 1.0;
        if current_ratio > 0.15 {
            sum_scores += weight;
            assessments.push(format!(
                "Good liquidity with current ratio ({current_ratio})"
            ));
        } else if current_ratio > 0.07 {
            sum_scores += weight / 2.0;
            assessments.push(format!(
                "Acceptable liquidity with current ratio ({current_ratio})"
            ));
        } else {
            assessments.push(format!(
                "Weak liquidity with current ratio ({current_ratio})"
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
            assessments.push("Have good fundamentals".to_string());
        } else {
            assessments.push("Not have good fundamentals".to_string());
        }
    }

    Ok(AnalysisDraft { score, assessments })
}

async fn analyze_management(
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
            assessments.push("High management quality".to_string());
        } else {
            assessments.push("Low management quality".to_string());
        }
    }

    Ok(AnalysisDraft { score, assessments })
}

async fn analyze_moat(
    stock_fiscal_metricsets: &[StockFiscalMetricset],
) -> InvmstResult<AnalysisDraft> {
    if stock_fiscal_metricsets.len() < 4 {
        return Ok(AnalysisDraft {
            score: None,
            assessments: vec!["Insufficient historical data for moat analysis".to_string()],
        });
    }

    let mut sum_scores: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;
    let mut assessments: Vec<String> = vec![];

    let roes: Vec<f64> = stock_fiscal_metricsets
        .iter()
        .filter_map(|(_, metrics)| metrics.financial_summary.return_on_equity)
        .collect();
    let operating_margins: Vec<f64> = stock_fiscal_metricsets
        .iter()
        .filter_map(|(_, metrics)| metrics.financial_summary.operating_margin)
        .collect();

    // 持续的高资本回报率
    {
        if roes.len() >= 4 {
            let high_roes_count = roes.iter().filter(|v| **v >= 0.15).count();
            let roe_consistency = high_roes_count as f64 / roes.len() as f64;

            let weight = 1.0;
            if roe_consistency >= 0.75 {
                sum_scores += weight;
                assessments.push("High ROE consistency".to_string());
            } else {
                assessments.push("Low ROE consistency".to_string());
            }
            sum_weights += weight;
        }
    }

    // 定价权（稳定的高利润率）
    {
        if operating_margins.len() >= 4 {
            let avg = operating_margins.iter().sum::<f64>() / operating_margins.len() as f64;

            let recent_operating_margins = operating_margins[..3].to_vec();
            let recent_avg = recent_operating_margins.iter().sum::<f64>()
                / recent_operating_margins.len() as f64;

            let early_operating_margins = operating_margins[operating_margins.len() - 3..].to_vec();
            let early_avg =
                early_operating_margins.iter().sum::<f64>() / early_operating_margins.len() as f64;

            let weight = 1.0;
            if avg >= 0.15 && recent_avg >= avg * 0.8 && early_avg >= 0.8 {
                sum_scores += weight;
                assessments.push("Strong pricing power".to_string());
            } else {
                assessments.push("Weak pricing power".to_string());
            }
            sum_weights += weight;
        }
    }

    // 规模优势（资产周转率）
    {
        let asset_turnovers: Vec<f64> = stock_fiscal_metricsets
            .iter()
            .filter_map(|(_, metrics)| metrics.financial_summary.asset_turnover)
            .collect();
        if asset_turnovers.len() >= 3 {
            let weight = 1.0;
            if asset_turnovers.iter().any(|v| *v >= 1.0) {
                sum_scores += weight;
                assessments
                    .push("Efficient asset utilization indicates scale advantages".to_string());
            } else {
                assessments.push("Not have efficient asset utilization".to_string());
            }
            sum_weights += weight;
        }
    }

    // 竞争优势（稳定的利润率和资本回报率）
    {
        if roes.len() >= 4 && operating_margins.len() >= 4 {
            let roe_avg = roes.iter().sum::<f64>() / roes.len() as f64;
            let roe_std = utils::stats::std(&roes);
            let roe_stability = if let Some(roe_std) = roe_std {
                1.0 - (roe_std) / roe_avg
            } else {
                0.0
            };

            let operating_margin_avg =
                operating_margins.iter().sum::<f64>() / operating_margins.len() as f64;
            let operating_margin_std = utils::stats::std(&operating_margins);
            let operating_margin_stability =
                if let Some(operating_margin_std) = operating_margin_std {
                    1.0 - (operating_margin_std) / operating_margin_avg
                } else {
                    0.0
                };

            let overall_stability = (roe_stability + operating_margin_stability) / 2.0;

            let weight = 1.0;
            if overall_stability >= 0.7 {
                sum_scores += weight;
                assessments
                    .push("High performance stability indicates strong competitive".to_string());
            } else {
                assessments.push("Not have high performance stability".to_string());
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
            assessments.push("Have a durable competitive advantage (moat)".to_string());
        } else {
            assessments.push("Not have a durable competitive advantage (moat)".to_string());
        }
    }

    Ok(AnalysisDraft { score, assessments })
}

static LLM_SYSTEM: &str = r#"
我是沃伦·巴菲特（Warren Buffett），下面是我的投资分析方法论：

**核心原则**
1. 能力圈原则：只投资我彻底理解的生意。
2. 经济护城河：寻找具有持久竞争优势的企业——定价权、品牌力、规模优势、转换成本。
3. 管理层品质：选择诚实能干、像所有者一样思考、善于资本配置的管理层。
4. 财务堡垒：偏好资产负债表强劲、盈利稳定、负债极少的企业。
5. 内在价值与安全边际：支付远低于企业价值的对价。
6. 长期视角：寻找能繁荣数十年的企业。
7. 定价权：最好的企业可以提价而不流失客户。

## 能力圈偏好
- 拥有强势品牌的消费品（可口可乐、宝洁、沃尔玛、好市多）
- 商业银行（美国银行、富国银行）
- 保险业（GEICO、财产意外险）
- 铁路和公用事业（BNSF铁路、简单基础设施）
- 具有护城河的简单工业（UPS、联邦快递、卡特彼勒）
- 拥有储备和管道的能源公司（雪佛龙，不包括勘探类）

## 能力圈回避
- 复杂科技（半导体、软件，苹果例外因其消费生态）
- 生物科技和制药（过于复杂，监管风险高）
- 航空业（商品化生意，经济性差）
- 加密货币和金融科技投机
- 复杂衍生品或金融工具
- 技术快速迭代的行业
- 缺乏定价权的重资产生意
- 投资银行

## 评估方法
1. 能力圈：如果不懂商业模式或行业逻辑，无论潜在回报多高都不投。
2. 企业质量：是否有护城河？20年后是否依然兴旺？
3. 管理层：是否维护股东利益？资本配置是否明智？
4. 财务实力：盈利是否稳定？负债是否低？资本回报率是否强劲？
5. 估值：是否为优秀企业支付了合理价格？

## 评分等级（百分制）
- 80-100：卓越企业，价格诱人
- 60-79：良好企业，估值合理
- 40-59：信号混杂，需更多信息或更优价格
- 20-39：超出能力圈或基本面存疑
- 0-19：劣质企业或严重高估

注意：我宁愿以合理价格买卓越企业，也不愿以超低价买平庸企业。当有疑虑时，答案通常是不投资，因为错过机会不会受惩罚，本金永久损失才会。
"#;
