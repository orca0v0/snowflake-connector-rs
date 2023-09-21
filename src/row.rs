use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::{Error, Result};

#[derive(Debug)]
pub struct SnowflakeRow {
    pub(crate) row: Vec<Option<String>>,
    pub(crate) column_names: Arc<HashMap<String, usize>>,
}

impl SnowflakeRow {
    pub fn get<T: SnowflakeDecode>(&self, column_name: &str) -> Result<T> {
        let index = self
            .column_names
            .get(&column_name.to_ascii_uppercase())
            .ok_or_else(|| Error::Decode(format!("column not found: {}", column_name)))?;
        self.row[*index].try_get()
    }
}

pub trait SnowflakeDecode: Sized {
    fn try_decode(value: &Option<String>) -> Result<Self>;
}

impl SnowflakeDecode for u64 {
    fn try_decode(value: &Option<String>) -> Result<Self> {
        let value = unwrap(value)?;
        value
            .parse()
            .map_err(|_| Error::Decode(format!("'{value}' is not u64")))
    }
}

impl SnowflakeDecode for String {
    fn try_decode(value: &Option<String>) -> Result<Self> {
        let value = unwrap(value)?;
        Ok(value.to_string())
    }
}

impl SnowflakeDecode for bool {
    fn try_decode(value: &Option<String>) -> Result<Self> {
        let value = unwrap(value)?;
        if let Ok(v) = value.parse::<u16>() {
            return Ok(v > 0);
        }
        if let Ok(v) = value.parse::<bool>() {
            return Ok(v);
        }
        Err(Error::Decode(format!("'{value}' is not bool")))
    }
}

impl SnowflakeDecode for NaiveDateTime {
    fn try_decode(value: &Option<String>) -> Result<Self> {
        let value = unwrap(value)?;
        if let Ok(v) = value.parse::<f64>() {
            let secs = v.trunc() as i64;
            let nsec = (v.fract() * 1_000_000_000.0) as u32;
            let dt = DateTime::<Utc>::from_timestamp(secs, nsec)
                .ok_or_else(|| Error::Decode(format!("invalid datetime: {}", value)))?;
            return Ok(dt.naive_utc());
        }
        if let Ok(v) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
            return Ok(v);
        }
        Err(Error::Decode(format!("'{value}' is not datetime")))
    }
}

impl<T: SnowflakeDecode> SnowflakeDecode for Option<T> {
    fn try_decode(value: &Option<String>) -> Result<Self> {
        if value.is_none() {
            return Ok(None);
        }
        T::try_decode(value).map(|v| Some(v))
    }
}

trait TryGet {
    fn try_get<T: SnowflakeDecode>(&self) -> Result<T>;
}

impl TryGet for Option<String> {
    fn try_get<T: SnowflakeDecode>(&self) -> Result<T> {
        T::try_decode(self)
    }
}

fn unwrap(value: &Option<String>) -> Result<&String> {
    value
        .as_ref()
        .ok_or_else(|| Error::Decode("value is null".into()))
}
