use crate::query::ast::{Field, FieldsProjection, Predicate, Query};
use crate::query::evaluator::reflect::Reflectable;
use crate::query::evaluator::result_set::ResultSet;
use crate::query::EvaluationError;
use std::borrow::Cow;
use std::collections::{HashMap};

impl Query {
    /// Execute [`Query`] on given `items`.
    ///
    /// Method will filter items by predicate and then project them to [`ResultSet`]
    pub fn execute<'a, T: Reflectable + 'a>(
        &self,
        items: impl IntoIterator<Item = &'a T>,
    ) -> Result<ResultSet, EvaluationError> {
        if let Some(predicate) = &self.predicate {
            self.fields_projection.project(predicate.filter(items)?)
        } else {
            self.fields_projection.project(items)
        }
    }
}

impl FieldsProjection {
    /// Return an iterator over column names, that need to be projected in [`ResultSet`].
    pub fn columns<'a, T: Reflectable + 'a>(&self) -> impl Iterator<Item = Cow<str>> {
        let fields_names = T::field_names();
        let mut columns = self
            .0
            .iter()
            .fold(
                HashMap::with_capacity(fields_names.len()),
                |mut columns, field| {
                    match field {
                        Field::Asterisk => {
                            for field in &*fields_names {
                                if !columns.contains_key(field) {
                                    columns.insert(field.clone(), columns.len());
                                }
                            }
                        }
                        Field::Name(field) => {
                            if !columns.contains_key(&Cow::from(&field.0)) {
                                columns.insert((&field.0).into(), columns.len());
                            }
                        }
                    }

                    columns
                },
            )
            .into_iter()
            .collect::<Vec<_>>();
        columns.sort_by_key(|(_, idx)| *idx);

        columns.into_iter().map(|(name, _)| name)
    }
    /// Projects `items` to the [`ResultSet`].
    pub fn project<'a, T: Reflectable + 'a>(
        &self,
        items: impl IntoIterator<Item = &'a T>,
    ) -> Result<ResultSet, EvaluationError> {
        items.into_iter().try_fold(
            ResultSet::with_columns(self.columns::<T>()),
            |mut result_set, item| {
                let mut values = Vec::new();
                for field in &self.0 {
                    match field {
                        Field::Asterisk => {
                            values.extend(item.fields().map(|(name, value)| (name, value)))
                        }
                        Field::Name(name) => {
                            values.push(((&name.0).into(), item.get_field(&name.0)?))
                        }
                    }
                }

                result_set.add_row(values);

                Ok(result_set)
            },
        )
    }
}

impl Predicate {
    /// Test given `value` by predicate.
    pub fn test<T: Reflectable + ?Sized>(&self, value: &T) -> Result<bool, EvaluationError> {
        Ok(self.expr.eval(value)?.cast_to_bool()?)
    }

    /// Filter given values by predicate.
    pub fn filter<'a, T: Reflectable + ?Sized>(
        &self,
        items: impl IntoIterator<Item = &'a T>,
    ) -> Result<Vec<&'a T>, EvaluationError> {
        items
            .into_iter()
            .filter_map(|value| match self.test(value) {
                Ok(true) => Some(Ok(value)),
                Ok(false) => None,
                Err(err) => Some(Err(err)),
            })
            .collect()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::query::reflect::tests::TestReflect;
    use chrono::{NaiveDateTime};
    use std::str::FromStr;
    use crate::query::evaluator::value::conversion::ConversionError;
    use crate::query::reflect::{ReflectError, Value};

    #[test]
    fn predicate_filter() {
        let query = Query::from_str(r"
            SELECT *
            WHERE (date_time >= '2024-12-12 20:20' AND date_time < '2028-12-01 20:20')
            OR ((number = 10 OR number = 1) AND string LIKE 'Hello')"
        ).unwrap();
        let predicate = query.predicate.unwrap();
        let test_dataset = test_dataset();

        let result = predicate.filter(&test_dataset);
        assert!(matches!(result, Ok(vec) if vec.len() == 4))

    }

    #[test]
    fn field_projection_asterisk() {
        let query = Query::from_str(r"SELECT *").unwrap();
        let projection = query.fields_projection;
        let test_dataset = test_dataset();

        let result = projection.project(&test_dataset);

        assert!(matches!(result, Ok(vec) if vec.columns().eq(["string", "number", "date_time"])))
    }

    #[test]
    fn field_projection_selected() {
        let query = Query::from_str(r"SELECT string, date_time").unwrap();
        let projection = query.fields_projection;
        let test_dataset = test_dataset();

        let result = projection.project(&test_dataset);

        assert!(matches!(result, Ok(vec) if vec.columns().eq(["string", "date_time"])))
    }

    #[test]
    fn field_projection_combined() {
        let query = Query::from_str(r"SELECT date_time, *").unwrap();
        let projection = query.fields_projection;
        let test_dataset = test_dataset();

        let result = projection.project(&test_dataset);

        assert!(matches!(result, Ok(vec) if vec.columns().eq(["date_time","string", "number"])))
    }

    #[test]
    fn query() {
        let query = Query::from_str(r"
            SELECT number
            WHERE (date_time >= '2024-12-12 20:20' AND date_time < '2028-12-01 20:20')
            OR ((number = 10 OR number = 1) AND string LIKE 'Hello')"
        ).unwrap();
        let test_dataset = test_dataset();

        let result = query.execute(&test_dataset);

        assert!(matches!(result, Ok(vec) if vec.rows().eq([
            [Value::Number(1.into())],
            [Value::Number(10.into())],
            [Value::Number((-10).into())],
            [Value::Number(15.into())]
        ])))
    }

    #[test]
    fn incorrect_field_query() {
        let query = Query::from_str(r"
            SELECT field
            WHERE (date_time >= '2024-12-12 20:20' AND date_time < '2028-12-01 20:20')
            OR ((number = 10 OR number = 1) AND string LIKE 'Hello')"
        ).unwrap();
        let test_dataset = test_dataset();

        let result = query.execute(&test_dataset);

        assert!(matches!(result, Err(EvaluationError::Reflect(ReflectError::NoField(_)))));
    }

    #[test]
    fn incorrect_predicate_query() {
        let query = Query::from_str(r"
            SELECT *
            WHERE string > 0"
        ).unwrap();
        let test_dataset = test_dataset();

        let result = query.execute(&test_dataset);

        assert!(matches!(result, Err(EvaluationError::Conversion(ConversionError::Failed { .. }))));
    }

    pub fn test_dataset() -> Vec<TestReflect> {
        Vec::from([
            TestReflect {
                string: "Hello".to_string(),
                number: 1,
                date_time: NaiveDateTime::parse_from_str("2007-12-12 17:25", "%Y-%m-%d %H:%M")
                    .unwrap()
                    .and_utc(),
            },
            TestReflect {
                string: "Hello World".to_string(),
                number: 10,
                date_time: NaiveDateTime::parse_from_str("2002-12-12 20:20", "%Y-%m-%d %H:%M")
                    .unwrap()
                    .and_utc(),
            },
            TestReflect {
                string: "World".to_string(),
                number: -10,
                date_time: NaiveDateTime::parse_from_str("2024-12-12 20:20", "%Y-%m-%d %H:%M")
                    .unwrap()
                    .and_utc(),
            },
            TestReflect {
                string: "Hi".to_string(),
                number: 15,
                date_time: NaiveDateTime::parse_from_str("2027-12-12 15:20", "%Y-%m-%d %H:%M")
                    .unwrap()
                    .and_utc(),
            },
            TestReflect {
                string: "Welcome".to_string(),
                number: 13,
                date_time: NaiveDateTime::parse_from_str("2017-12-12 20:20", "%Y-%m-%d %H:%M")
                    .unwrap()
                    .and_utc(),
            },
            TestReflect {
                string: "Hi World".to_string(),
                number: -20,
                date_time: NaiveDateTime::parse_from_str("2020-12-01 20:20", "%Y-%m-%d %H:%M")
                    .unwrap()
                    .and_utc(),
            },
        ])
    }
}
