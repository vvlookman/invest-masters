use std::path::Path;

use chrono::NaiveDate;
use polars::prelude::*;

use crate::{error::InvmstResult, utils};

pub trait Daily {
    fn get_date_max(&self) -> Option<NaiveDate>;
    fn get_date_min(&self) -> Option<NaiveDate>;
    fn has_date(&self, date: &NaiveDate) -> bool;
}

#[derive(Debug)]
pub struct DailyData {
    df: DataFrame,

    date_field_name: String,
}

impl DailyData {
    pub fn from_csv(csv_path: &Path, date_field_name: &str) -> InvmstResult<Self> {
        let df = CsvReadOptions::default()
            // .with_columns(Some(Arc::new([
            //     date_field_name.into(),
            //     price_field_name.into(),
            // ])))
            .map_parse_options(|parse_options| parse_options.with_try_parse_dates(true))
            .try_into_reader_with_file_path(Some(csv_path.to_path_buf()))?
            .finish()?;

        Ok(Self {
            df,
            date_field_name: date_field_name.to_string(),
        })
    }
}

impl Daily for DailyData {
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
