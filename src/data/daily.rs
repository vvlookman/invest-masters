use chrono::NaiveDate;
use polars::prelude::*;
use serde_json::Value;

use crate::{
    error::{InvmstError, InvmstResult},
    utils,
};

#[derive(Debug)]
pub struct DailyData {
    df: DataFrame,

    date_field_name: String,
}

impl DailyData {
    pub fn from_json(json: &Value, date_field_name: &str) -> InvmstResult<Self> {
        if let Some(array) = json.as_array() {
            if let Some(first) = array.first() {
                if let Some(obj) = first.as_object() {
                    let column_names: Vec<String> = obj.keys().cloned().collect();

                    let mut series: Vec<Column> = Vec::with_capacity(column_names.len());
                    for column_name in column_names {
                        let is_date_column = column_name == date_field_name;
                        let mut values: Vec<AnyValue> = vec![];

                        for item in array {
                            if let Some(obj) = item.as_object() {
                                if let Some(val) = obj.get(&column_name) {
                                    match val {
                                        Value::Null => {
                                            values.push(AnyValue::Null);
                                            continue;
                                        }
                                        Value::Bool(b) => {
                                            values.push(AnyValue::Boolean(*b));
                                            continue;
                                        }
                                        Value::Number(n) => {
                                            if let Some(i) = n.as_i64() {
                                                values.push(AnyValue::Int64(i));
                                                continue;
                                            } else if let Some(f) = n.as_f64() {
                                                values.push(AnyValue::Float64(f));
                                                continue;
                                            } else if let Some(u) = n.as_u64() {
                                                values.push(AnyValue::UInt64(u));
                                                continue;
                                            }
                                        }
                                        Value::String(s) => {
                                            if is_date_column {
                                                let days_after_epoch: i32 = if let Ok((date, _)) =
                                                    NaiveDate::parse_and_remainder(s, "%Y-%m-%d")
                                                {
                                                    utils::datetime::days_after_epoch(&date)
                                                        .unwrap_or(0)
                                                } else {
                                                    0
                                                };
                                                values.push(AnyValue::Date(days_after_epoch));
                                            } else {
                                                values.push(AnyValue::String(s));
                                            }

                                            continue;
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            values.push(AnyValue::Null);
                        }

                        series.push(Column::new(column_name.into(), values));
                    }

                    let df = DataFrame::new(series)?;

                    Ok(Self {
                        df,
                        date_field_name: date_field_name.to_string(),
                    })
                } else {
                    Err(InvmstError::Invalid(
                        "JSON_ITEM_IS_NOT_OBJECT",
                        "Json item is not a valid object".to_string(),
                    ))
                }
            } else {
                Err(InvmstError::Invalid(
                    "JSON_IS_EMPTY",
                    "Json is empty".to_string(),
                ))
            }
        } else {
            Err(InvmstError::Invalid(
                "JSON_IS_NOT_ARRAY",
                "Json is not a valid array".to_string(),
            ))
        }
    }

    fn get_date_max(&self) -> Option<NaiveDate> {
        if let Ok(df) = self
            .df
            .clone()
            .lazy()
            .filter(col(&self.date_field_name).is_not_null())
            .sort(
                [&self.date_field_name],
                SortMultipleOptions::default().with_order_descending(true),
            )
            .first()
            .collect()
        {
            if let Ok(col) = df.column(&self.date_field_name) {
                if let Ok(val) = col.get(0) {
                    if let Some(days_after_epoch) = val.extract::<i32>() {
                        return utils::datetime::date_from_days_after_epoch(days_after_epoch);
                    }
                }
            }
        }

        None
    }

    fn get_date_min(&self) -> Option<NaiveDate> {
        if let Ok(df) = self
            .df
            .clone()
            .lazy()
            .filter(col(&self.date_field_name).is_not_null())
            .sort(
                [&self.date_field_name],
                SortMultipleOptions::default().with_order_descending(false),
            )
            .first()
            .collect()
        {
            if let Ok(col) = df.column(&self.date_field_name) {
                if let Ok(val) = col.get(0) {
                    if let Some(days_after_epoch) = val.extract::<i32>() {
                        return utils::datetime::date_from_days_after_epoch(days_after_epoch);
                    }
                }
            }
        }

        None
    }

    fn has_date(&self, date: &NaiveDate) -> bool {
        match self
            .df
            .clone()
            .lazy()
            .filter(col(&self.date_field_name).eq(lit(*date)))
            .first()
            .collect()
        {
            Ok(df) => df.height() > 0,
            Err(_) => false,
        }
    }
}
