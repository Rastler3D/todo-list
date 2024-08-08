use super::{Number, Value};
use chrono::{DateTime, NaiveDateTime, Utc};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// Represents possible types of [`Value`].
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum Type {
    DateTime = 0,
    Number = 1,
    Bool = 3,
    String = 4,
    Null = 5,
}

impl Type {
    /// Returns precedence of the type.
    ///
    /// When an operator combines expressions of different data types, the data type with the lower precedence is first converted to the data type with the higher precedence.
    pub fn precedence(self) -> u8 {
        self as u8
    }
}

impl Value {
    /// Returns the type of current [`Value`]
    pub fn r#type(&self) -> Type {
        match self {
            Value::Null => Type::Null,
            Value::Bool(_) => Type::Bool,
            Value::Number(_) => Type::Number,
            Value::String(_) => Type::String,
            Value::DateTime(_) => Type::DateTime,
        }
    }
    /// Unify types so they are now the same type and can be used in binary operations.
    ///
    /// When an operator combines expressions of different data types, the data type with the lower precedence is first converted to the data type with the higher precedence.
    pub fn unify_types<'a, 'b>(
        left: &'a Value,
        right: &'b Value,
    ) -> Result<(Cow<'a, Self>, Cow<'b, Self>), ConversionError> {
        let left_type = left.r#type();
        let right_type = right.r#type();

        match left_type.precedence().cmp(&right_type.precedence()) {
            Ordering::Equal => Ok((left.into(), right.into())),
            Ordering::Less => Ok((left.into(), right.cast_to(left_type)?.into())),
            Ordering::Greater => Ok((left.cast_to(right_type)?.into(), right.into())),
        }
    }
    /// Try to cast current [`Value`] to provided [`Type`].
    ///
    /// If conversion to the provided type fails or is not possible, an error will be returned.
    pub fn cast_to(&self, r#type: Type) -> Result<Self, ConversionError> {
        return match r#type {
            Type::DateTime => self.cast_to_datetime().map(Value::DateTime),
            Type::Number => self.cast_to_number().map(Value::Number),
            Type::Bool => self.cast_to_bool().map(Value::Bool),
            Type::String => self.cast_to_string().map(|x| Value::String(x.to_string())),
            Type::Null => Err(ConversionError::NotAllowed {
                from: self.r#type(),
                to: Type::Null,
            }),
        };
    }
    /// Try to cast current [`Value`] to [`DateTime`].
    ///
    /// If conversion to [`DateTime`] fails or is not possible, an error will be returned.
    pub fn cast_to_datetime(&self) -> Result<DateTime<Utc>, ConversionError> {
        let value = match self {
            Value::DateTime(datetime) => *datetime,
            Value::Number(number) => {
                DateTime::from_timestamp(number.as_i64(), 0).ok_or_else(|| {
                    ConversionError::Failed {
                        value: Value::Number(*number),
                        dest_type: Type::DateTime,
                        reason: "Number is out-of-range".to_string(),
                    }
                })?
            }
            Value::String(string) => NaiveDateTime::parse_from_str(string, "%Y-%m-%d %H:%M")
                .map_err(|err| ConversionError::Failed {
                    value: Value::String(string.to_string()),
                    dest_type: Type::DateTime,
                    reason: err.to_string(),
                })?
                .and_utc(),
            value => {
                return Err(ConversionError::NotAllowed {
                    from: value.r#type(),
                    to: Type::DateTime,
                })
            }
        };

        Ok(value)
    }
    /// Try to cast current [`Value`] to [`Number`].
    ///
    /// If conversion to [`Number`] fails or is not possible, an error will be returned.
    pub fn cast_to_number(&self) -> Result<Number, ConversionError> {
        let value = match self {
            Value::Number(number) => *number,
            Value::Bool(bool) => Number::Int(*bool as i64),
            Value::DateTime(datetime) => Number::Int(datetime.timestamp()),
            Value::String(string) => {
                string
                    .parse::<Number>()
                    .map_err(|err| ConversionError::Failed {
                        value: Value::String(string.to_string()),
                        dest_type: Type::Number,
                        reason: err.to_string(),
                    })?
            }
            value => {
                return Err(ConversionError::NotAllowed {
                    from: value.r#type(),
                    to: Type::Number,
                })
            }
        };

        Ok(value)
    }
    /// Try to cast current [`Value`] to [`String`].
    ///
    /// If conversion to [`String`] fails or is not possible, an error will be returned.
    pub fn cast_to_string(&self) -> Result<Cow<str>, ConversionError> {
        let value = match self {
            Value::String(string) => string.into(),
            Value::Bool(bool) => bool.to_string().into(),
            Value::Number(number) => number.to_string().into(),
            Value::DateTime(datetime) => datetime.format("%Y-%m-%d %H:%M").to_string().into(),
            value => {
                return Err(ConversionError::NotAllowed {
                    from: value.r#type(),
                    to: Type::String,
                })
            }
        };

        Ok(value)
    }
    /// Try to cast current [`Value`] to [`bool`].
    ///
    /// If conversion to [`bool`] fails or is not possible, an error will be returned.
    pub fn cast_to_bool(&self) -> Result<bool, ConversionError> {
        let value = match self {
            Value::Bool(bool) => *bool,
            Value::Number(number) => {
                if number.as_i64() == 0 {
                    false
                } else {
                    true
                }
            }
            Value::String(string) => {
                string
                    .parse::<bool>()
                    .map_err(|err| ConversionError::Failed {
                        value: Value::String(string.to_string()),
                        dest_type: Type::Bool,
                        reason: err.to_string(),
                    })?
            }
            value => {
                return Err(ConversionError::NotAllowed {
                    from: value.r#type(),
                    to: Type::Bool,
                })
            }
        };

        Ok(value)
    }
}

/// Represents possible errors of type conversion
#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Conversion from type '{from}' to type '{to}' is not allowed")]
    NotAllowed { from: Type, to: Type },
    #[error("Failed to convert value '{value}' to type '{dest_type}'. \nReason: {reason}")]
    Failed {
        value: Value,
        dest_type: Type,
        reason: String,
    },
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Type::DateTime => "DateTime",
            Type::Number => "Number",
            Type::Bool => "Bool",
            Type::String => "String",
            Type::Null => "Null",
        };

        Display::fmt(val, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unify_types() {
        let left = Value::String("2020-12-12 20:20".to_string());
        let right = Value::DateTime(Utc::now());

        assert_ne!(left.r#type(), right.r#type());

        let (left, right) = Value::unify_types(&left, &right).unwrap();

        assert_eq!(left.r#type(), right.r#type());
        assert_eq!(left.r#type(), Type::DateTime);

        let null = Value::Null;

        assert!(matches!(Value::unify_types(&left, &null), Err(ConversionError::NotAllowed {..})));
    }

    #[test]
    fn cast_string_to_num() {
        let value = Value::String("3.14".to_string());

        assert_ne!(value.r#type(), Type::Number);

        assert!(matches!(value.cast_to_number(), Ok(Number::Float(3.14))));

        let incorrect = Value::String("IncorrectNumber".to_string());

        assert!(matches!(
            incorrect.cast_to_number(),
            Err(ConversionError::Failed { .. })
        ));
    }

    #[test]
    fn cast_string_to_datetime() {
        let value = Value::String("2020-12-12 20:20".to_string());

        assert_ne!(value.r#type(), Type::DateTime);

        assert!(matches!(value.cast_to_datetime(),
            Ok(datetime) if datetime == NaiveDateTime::parse_from_str("2020-12-12 20:20", "%Y-%m-%d %H:%M")
                .unwrap()
                .and_utc())
        );

        let incorrect = Value::String("IncorrectDate".to_string());

        assert!(matches!(
            incorrect.cast_to_datetime(),
            Err(ConversionError::Failed { .. })
        ));
    }

    #[test]
    fn not_allowed_cast() {
        let value = Value::Bool(true);

        assert_ne!(value.r#type(), Type::DateTime);

        assert!(matches!(value.cast_to_datetime(), Err(ConversionError::NotAllowed { .. })));
    }
}
