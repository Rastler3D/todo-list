pub mod conversion;
pub mod operations;

use std::borrow::Cow;
use crate::query::ast::expression::Literal;
use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use std::fmt::Display;
use std::num::ParseFloatError;
use std::str::FromStr;

/// Represents possible values of [`Query`] expression execution.
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    DateTime(DateTime<Utc>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => Display::fmt("NULL", f),
            Value::Bool(bool) => Display::fmt(bool, f),
            Value::String(string) => Display::fmt(string, f),
            Value::Number(number) => Display::fmt(number, f),
            Value::DateTime(date_time) => Display::fmt(&date_time.format("%Y-%m-%d %H:%M"), f),
        }
    }
}

impl From<&Literal> for Value {
    fn from(val: &Literal) -> Value {
        match val {
            Literal::Null => Value::Null,
            Literal::Bool(bool) => Value::Bool(*bool),
            Literal::Number(number) => Value::Number(*number),
            Literal::String(string) => Value::String(string.to_string()),
        }
    }
}

impl From<Value> for Cow<'static, Value> {
    fn from(value: Value) -> Self {
        Cow::Owned(value)
    }
}

impl<'a> From<&'a Value> for Cow<'a, Value> {
    fn from(value: &'a Value) -> Self {
        Cow::Borrowed(value)
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Number {
    Int(i64),
    Float(f64),
}

impl Number {
    pub fn as_i64(self) -> i64 {
        match self {
            Number::Int(i64) => i64,
            Number::Float(f64) => f64 as i64,
        }
    }

    pub fn as_f64(self) -> f64 {
        match self {
            Number::Int(i64) => i64 as f64,
            Number::Float(f64) => f64,
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(int) => Display::fmt(int, f),
            Number::Float(float) => Display::fmt(float, f),
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Number::Float(first), Number::Int(second))
            | (Number::Int(second), Number::Float(first)) => first.eq(&(*second as f64)),
            (Number::Int(first), Number::Int(second)) => first.eq(second),
            (Number::Float(first), Number::Float(second)) => first.eq(second),
        }
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Number::Float(first), Number::Int(second)) => first.partial_cmp(&(*second as f64)),
            (Number::Int(first), Number::Float(second)) => (*first as f64).partial_cmp(second),
            (Number::Int(first), Number::Int(second)) => first.partial_cmp(second),
            (Number::Float(first), Number::Float(second)) => first.partial_cmp(second),
        }
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Number::Float(value)
    }
}

impl From<i64> for Number {
    fn from(value: i64) -> Self {
        Number::Int(value)
    }
}

impl FromStr for Number {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<i64>()
            .map(Number::Int)
            .or_else(|_| s.parse::<f64>().map(Number::Float))
    }
}

impl Into<String> for &Value {
    fn into(self) -> String {
        self.to_string()
    }
}


#[cfg(test)]
mod tests {
    use chrono::{Datelike, Timelike};
    use super::*;

    #[test]
    fn date_format() {
        let date = DateTime::<Utc>::default()
            .with_year(2020).unwrap()
            .with_month(12).unwrap()
            .with_day(12).unwrap()
            .with_hour(20).unwrap()
            .with_minute(20).unwrap();

        let date = Value::DateTime(date);

        assert_eq!(date.to_string(), "2020-12-12 20:20");
    }

    #[test]
    fn value_cmp() {
        let left = Value::Number(1.into());
        let right = Value::Number((10.).into());

        assert!(left < right)
    }
}
