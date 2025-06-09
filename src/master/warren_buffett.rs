use serde_json::json;

use crate::{
    error::InvmstError,
    master::{AnalysisDraft, FiscalStockMetrics, InvmstResult},
    ticker::Ticker,
};

pub async fn evaluate(
    _ticker: &Ticker,
    trailing_stock_metrics: &[FiscalStockMetrics],
) -> InvmstResult<()> {
    if trailing_stock_metrics.is_empty() {
        return Err(InvmstError::NoData(
            "NO_STOCK_METRICS",
            "No stock metrics data".to_string(),
        ));
    }

    let prompt_system = r#"
    我是沃伦·巴菲特（Warren Buffett），下面是我的投资分析方法论：

    **核心原则**
    1. 能力圈原则：只投资我彻底理解的生意。
    2. 经济护城河：寻找具有持久竞争优势的企业——定价权、品牌力、规模优势、转换成本。
    3. 管理层品质：选择诚实能干、像所有者一样思考、善于资本配置的管理层。
    4. 财务堡垒：偏好资产负债表强劲、盈利稳定、负债极少的企业。
    5. 内在价值与安全边际：支付远低于企业价值的对价。
    6. 长期视角：寻找能繁荣数十年的企业。
    7. 定价权：最好的企业可以提价而不流失客户。

    **能力圈偏好**
    - 拥有强势品牌的消费品（可口可乐、宝洁、沃尔玛、好市多）
    - 商业银行（美国银行、富国银行）
    - 保险业（GEICO、财产意外险）
    - 铁路和公用事业（BNSF铁路、简单基础设施）
    - 具有护城河的简单工业（UPS、联邦快递、卡特彼勒）
    - 拥有储备和管道的能源公司（雪佛龙，不投勘探类）

    **能力圈回避**
    - 复杂科技（半导体、软件，苹果例外因其消费生态）
    - 生物科技和制药（过于复杂，监管风险高）
    - 航空业（商品化生意，经济性差）
    - 加密货币和金融科技投机
    - 复杂衍生品或金融工具
    - 技术快速迭代的行业
    - 缺乏定价权的重资产生意
    - 投资银行

    **评估方法**
    1. 能力圈：如果不懂商业模式或行业逻辑，无论潜在回报多高都不投。
    2. 企业质量：是否有护城河？20年后是否依然兴旺？
    3. 管理层：是否维护股东利益？资本配置是否明智？
    4. 财务实力：盈利是否稳定？负债是否低？资本回报率是否强劲？
    5. 估值：是否为优秀企业支付了合理价格？

    **评分等级**
    - 80-100：卓越企业，价格诱人
    - 60-79：护城河良好的企业，估值合理
    - 40-59：信号混杂，需更多信息或更优价格
    - 20-39：超出能力圈或基本面存疑
    - 0-19：劣质企业或严重高估

    注意：我宁愿以合理价格买卓越企业，也不愿以超低价买平庸企业。当有疑虑时，答案通常是不投资，因为错过机会不会受惩罚，本金永久损失才会。
    "#;

    let latest_stock_metrics = trailing_stock_metrics.first().unwrap();

    let fundamentals = analyze_fundamentals(&latest_stock_metrics).await?;
    println!("{fundamentals:?}");

    let data_json = json!({
        "fundamentals": fundamentals
    })
    .to_string();

    let prompt_user = format!(
        r#"
    根据下面提供的数据，分析评估投资对象：

    ```
    {data_json}
    ```

    结果严格按照下面的 JSON 格式返回：
    
    ```
    {{
      "signal": "bullish" | "bearish" | "neutral",
      "confidence": 0到100之间的整数,
      "reasoning": "详细的分析过程"
    }}
    ```

    在分析中具体说明：
    1. 该投资是否属于我的能力圈范围及原因
    2. 对企业竞争护城河的评估
    3. 管理层质量与资本配置能力
    4. 财务健康状况与盈利稳定性
    5. 相对于内在价值的估值水平
    6. 长期前景及任何风险警示
    7. 与我现有投资组合机会的对比
    "#
    );

    Ok(())
}

async fn analyze_fundamentals(
    latest_stock_metrics: &FiscalStockMetrics,
) -> InvmstResult<AnalysisDraft> {
    let mut sum_scores: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;
    let mut assessments: Vec<String> = vec![];

    let (_, stock_metrics) = latest_stock_metrics;

    if let Some(return_on_equity) = stock_metrics.financial_summary.return_on_equity {
        let weight = 1.0;
        if return_on_equity > 0.15 {
            sum_scores += weight;
            assessments.push(format!("Strong return on equity ({return_on_equity})"));
        } else {
            assessments.push(format!("Weak return on equity ({return_on_equity})"));
        }
        sum_weights += weight;
    } else {
        assessments.push("No return on equity data".to_string());
    }

    if let Some(debt_to_equity) = stock_metrics.financial_summary.debt_to_equity {
        let weight = 1.0;
        if debt_to_equity < 0.5 {
            sum_scores += weight;
            assessments.push(format!("Low debt to equity ({debt_to_equity})"));
        } else {
            assessments.push(format!("High debt to equity ({debt_to_equity})"));
        }
        sum_weights += weight;
    } else {
        assessments.push("No debt to equity data".to_string());
    }

    if let Some(operating_margin) = stock_metrics.financial_summary.operating_margin {
        let weight = 1.0;
        if operating_margin > 0.15 {
            sum_scores += weight;
            assessments.push(format!("Strong operating margin ({operating_margin})"));
        } else {
            assessments.push(format!("Weak operating margin ({operating_margin})"));
        }
        sum_weights += weight;
    } else {
        assessments.push("No operating margin data".to_string());
    }

    if let Some(current_ratio) = stock_metrics.financial_summary.current_ratio {
        let weight = 0.5;
        if current_ratio > 0.15 {
            sum_scores += weight;
            assessments.push(format!(
                "Good liquidity with current ratio ({current_ratio})"
            ));
        } else {
            assessments.push(format!(
                "Weak liquidity with current ratio ({current_ratio})"
            ));
        }
        sum_weights += weight;
    } else {
        assessments.push("No current ratio data".to_string());
    }

    let score = if sum_weights > 0.0 {
        Some(sum_scores / sum_weights)
    } else {
        None
    };

    Ok(AnalysisDraft { score, assessments })
}
