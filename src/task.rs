use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::iter::once;
use std::str::FromStr;
use crate::query::reflect::{FieldsIterator, ReflectError, Reflectable, Value};
use chrono::{DateTime, NaiveDateTime, Utc};
use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};
use tabled::settings::Style;

/// Represents task.
#[derive(Debug, Serialize, Deserialize, Args, Tabled, PartialEq)]
pub struct Task {
    pub name: String,
    pub description: String,
    #[arg(value_parser = parse_date_time)]
    pub date: DateTime<Utc>,
    pub category: String,
    pub status: Status
}

/// Represents task status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialOrd, PartialEq)]
pub enum Status{
    On,
    Off
}

fn parse_date_time(date: &str) -> Result<DateTime<Utc>, chrono::ParseError>{
    NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M")
        .map(|date| date.and_utc())
}

/// Reflectable implementation to be able to use task in select queries.
impl Reflectable for Task {
    fn get_field(&self, field: &str) -> Result<Value, ReflectError> {
        let value = match field {
            "name" => Value::String(self.name.to_string()),
            "description" => Value::String(self.description.to_string()),
            "date" => Value::DateTime(self.date),
            "category" => Value::String(self.category.to_string()),
            "status" => Value::String(self.status.to_string()),
            field => return Err(ReflectError::NoField(field.to_string())),
        };

        return Ok(value);
    }

    fn fields(&self) -> FieldsIterator {
        Box::new([
            ("name".into(), Value::String(self.name.to_string())),
            ("description".into(), Value::String(self.description.to_string())),
            ("date".into(), Value::DateTime(self.date)),
            ("category".into(), Value::String(self.category.to_string())),
            ("status".into(), Value::String(self.status.to_string())),
        ].into_iter())
    }

    fn field_names() -> Cow<'static, [Cow<'static, str>]> {
        (&[Cow::Borrowed("name"), Cow::Borrowed("description"), Cow::Borrowed("date"), Cow::Borrowed("category"), Cow::Borrowed("status")]).into()
    }
}

impl Display for Task{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new(once(self));

        Display::fmt(table.with(Style::modern_rounded()), f)

    }
}

impl Display for Status{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::On => Display::fmt("on", f),
            Status::Off => Display::fmt("off", f)
        }
    }
}

impl FromStr for Status{
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" | "ON" | "On" => Ok(Status::On),
            "off" | "OFF" | "Off" => Ok(Status::Off),
            _ => Err("String must be one of the possible value: ['on', 'On', 'ON', 'off', 'Off', 'OFF']")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_task() -> Task{
        Task{
            name: "RandomName".to_string(),
            description: "RandomDescription".to_string(),
            date: NaiveDateTime::parse_from_str("2020-12-12 20:20", "%Y-%m-%d %H:%M")
                .unwrap()
                .and_utc(),
            category: "RandomCategory".to_string(),
            status: Status::On
        }
    }
    #[test]
    fn get_field_reflectable() {
        let task = test_task();

        let name = task.get_field("name").unwrap();
        assert_eq!(name, Value::String(task.name.to_string()));

        let date = task.get_field("date").unwrap();
        assert_eq!(date, Value::DateTime(task.date));

        let status = task.get_field("status").unwrap();
        assert_eq!(status, Value::String(task.status.to_string()));

    }

    #[test]
    fn fields_reflectable() {
        let task = test_task();

        let fields = <Task as Reflectable>::fields(&task);

        assert!(fields.eq([
            ("name".into(), Value::String(task.name.to_string())),
            ("description".into(), Value::String(task.description.to_string())),
            ("date".into(), Value::DateTime(task.date)),
            ("category".into(), Value::String(task.category.to_string())),
            ("status".into(), Value::String(task.status.to_string()))
        ]));

    }
}