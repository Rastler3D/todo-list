use super::value::conversion::Type;
use std::borrow::Cow;
use thiserror::Error;

pub use super::value::Value;

/// Iterator over [`Reflectable`] type fields.
pub type FieldsIterator = Box<dyn Iterator<Item = (Cow<'static, str>, Value)>>;

/// Trait for runtime reflection and observation of struct fields.
pub trait Reflectable {
    /// Returns value of `field`.
    ///
    /// If field is not exists or cannot be converted to [`Value`] type, an error will be returned.
    fn get_field(&self, field: &str) -> Result<Value, ReflectError>;
    /// Returns field names along with their values.
    ///
    /// If field cannot be converted to [`Value`] type, it will be skipped.
    fn fields(&self) -> FieldsIterator;
    /// Returns field names.
    fn field_names() -> Cow<'static, [Cow<'static, str>]>
    where
        Self: Sized;
}

/// Represents possible errors of type reflection.
#[derive(Error, Debug)]
pub enum ReflectError {
    #[error("Field '{field}' has type '{r#type}', which is not supported. Type must be convertable to one of the supported types: '[{}, {}, {}, {}, {}]'", Type::Null, Type::String, Type::Number, Type::DateTime, Type::Bool)]
    UnsupportedType {
        field: Cow<'static, str>,
        r#type: Cow<'static, str>,
    },
    #[error("Field not exists")]
    NoField(String),
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use chrono::{DateTime, NaiveDateTime, Utc};
    use std::iter::empty;
    use serde::{Deserialize, Serialize};

    #[test]
    fn no_field() {
        let field_value = EmptyContext.get_field("Any field");

        assert!(matches!(field_value, Err(ReflectError::NoField(_))));
    }
    #[test]
    fn has_field() {
        let test_reflect = TestReflect::default();
        let field_value = test_reflect.get_field("string");

        assert!(matches!(field_value, Ok(Value::String(str)) if str == "Default string"));
    }

    #[test]
    fn fields() {
        let test_reflect = TestReflect::default();
        let fields = test_reflect.fields();

        assert!(fields.eq([
            ("string".into(), Value::Number(125.into())),
            ("number".into(), Value::String("Default string".to_string())),
            ("date_time".into(), Value::DateTime(NaiveDateTime::parse_from_str("2020-12-12 20:20", "%Y-%m-%d %H:%M")
                    .unwrap()
                    .and_utc()))
        ]));
    }

    #[test]
    fn fields_name() {
        let fields = TestReflect::field_names();

        assert_eq!(*fields,[
                Cow::Borrowed("string"),
                Cow::Borrowed("number"),
                Cow::Borrowed("date_time"),
            ]);
    }

    pub struct EmptyContext;

    impl Reflectable for EmptyContext {
        fn get_field(&self, field: &str) -> Result<Value, ReflectError> {
            Err(ReflectError::NoField(field.to_string()))
        }

        fn fields(&self) -> FieldsIterator {
            Box::new(empty())
        }

        fn field_names() -> Cow<'static, [Cow<'static, str>]> {
            (&[]).into()
        }
    }
    #[derive(Deserialize, Serialize, PartialEq, Debug)]
    pub struct TestReflect {
        pub string: String,
        pub number: i64,
        pub date_time: DateTime<Utc>,
    }
    impl Reflectable for TestReflect {
        fn get_field(&self, field: &str) -> Result<Value, ReflectError> {
            let value = match field {
                "string" => Value::String(self.string.to_string()),
                "number" => Value::Number(self.number.into()),
                "date_time" => Value::DateTime(self.date_time),
                field => return Err(ReflectError::NoField(field.to_string())),
            };

            return Ok(value);
        }

        fn fields(&self) -> FieldsIterator {
            Box::new(
                [
                    ("string".into(), Value::Number(self.number.into())),
                    ("number".into(), Value::String(self.string.to_string())),
                    ("date_time".into(), Value::DateTime(self.date_time)),
                ]
                .into_iter(),
            )
        }

        fn field_names() -> Cow<'static, [Cow<'static, str>]> {
            (&[
                Cow::Borrowed("string"),
                Cow::Borrowed("number"),
                Cow::Borrowed("date_time"),
            ])
                .into()
        }
    }

    impl Default for TestReflect {
        fn default() -> Self {
            TestReflect {
                number: 125,
                string: "Default string".to_string(),
                date_time: NaiveDateTime::parse_from_str("2020-12-12 20:20", "%Y-%m-%d %H:%M")
                    .unwrap()
                    .and_utc(),
            }
        }
    }
}
