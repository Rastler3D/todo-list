use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::iter::once;
use std::ops::Deref;
use tabled::builder::Builder;
use tabled::settings::Style;
use crate::query::evaluator::value::Value;

/// A table of data representing a [`Query`] result set.
///
/// Provides methods for manipulating table data.
/// # Examples
///
/// ```
/// use todo_list::query::ResultSet;
/// use todo_list::query::reflect::Value;
///
/// let mut result_set = ResultSet::new();
///
/// result_set.add_row([("second", Value::Bool(true)), ("third", Value::Null), ("first", Value::Number(1.into()))]);
/// result_set.add_row([("third", Value::Null), ("first", Value::Number(1.into())), ("second", Value::Bool(true))]);
/// result_set.add_row([("first", Value::Number(1.into())), ("second", Value::Bool(true)), ("third", Value::Null)]);
///
/// println!("{}", result_set);
/// ```
pub struct ResultSet{
    columns: HashMap<String, usize>,
    rows: Vec<Vec<Value>>
}
impl ResultSet{
    /// Create new empty [`ResultSet`].
    pub fn new() -> ResultSet{
        ResultSet{
            columns: HashMap::new(),
            rows: Vec::new()
        }
    }
    /// Create [`ResultSet`] with predefined `columns`.
    pub fn with_columns<'a, T: Into<Cow<'a, str>>>(columns: impl IntoIterator<Item = T>) -> ResultSet{
        let mut result_set = Self::new();
        result_set.add_columns(columns);
        result_set
    }

    /// Add new column with name `column_name` to [`ResultSet`] .
    ///
    /// The column is filled with `Value::Null` on existing rows
    pub fn add_column<'a>(&mut self, column_name: impl Into<Cow<'a, str>>){
        let column_name = column_name.into();
        if !self.columns.contains_key(&*column_name){
            self.columns.insert(column_name.into_owned(), self.columns.len());
            for row in &mut self.rows{
                row.push(Value::Null);
            }
        }
    }

    /// Add multiple columns to [`ResultSet`] .
    ///
    /// Columns are filled with `Value::Null` on existing rows
    pub fn add_columns<'a, T: Into<Cow<'a, str>>>(&mut self, columns: impl IntoIterator<Item = T>){
        for column_name in columns{
            self.add_column(column_name)
        }
    }

    /// Add new row with `values` to [`ResultSet`] .
    ///
    /// New columns will be added if required
    pub fn add_row<'a, T: Into<Cow<'a, str>>>(&mut self, values: impl IntoIterator<Item = (T, Value)>){
        let mut row = vec![Value::Null; self.columns.len()];

        for (column_name, value) in values{
            let column_name = column_name.into();
            if let Some(&id) = self.columns.get(&*column_name){
                row[id] = value;
            } else {
                self.add_column(column_name);
                row.push(value);
            }
        }

        self.rows.push(row);
    }

    /// Add multiple `rows` to [`ResultSet`] .
    ///
    /// New columns will be added if required
    pub fn add_rows<'a, R: IntoIterator<Item = (T, Value)>, T: Into<Cow<'a, str>>>(&mut self, rows: impl IntoIterator<Item = R>){
        for row in rows{
            self.add_row(row);
        }

    }

    /// Returns the iterator over the column names.
    ///
    /// The columns will be returned in the order in which they were added.
    pub fn columns(&self) -> impl Iterator<Item=&str>{
        let mut columns =self.columns.iter().collect::<Vec<_>>();
        columns.sort_by_key(|&(_, idx)| idx);

        columns.into_iter().map(|(name,_)| name.deref())
    }

    /// Returns the iterator over references to the [`Value`].
    ///
    /// The rows will be returned in the order in which they were added.
    /// The values in the row are ordered according to the order of the columns in the current [`ResultSet`]
    pub fn rows(&self) -> impl Iterator<Item=&[Value]>{
        self.rows
            .iter()
            .map(|x| x.deref())
    }

    /// Returns the iterator over references to the all [`Value`] of column with name `column_name`.
    ///
    /// If there is no such column in [`ResultSet`], an empty iterator will be returned.
    pub fn get_column(&self, column_name: &str) -> impl Iterator<Item=&Value>{
        let idx = self.columns.get(column_name).copied();

        self.rows
            .iter()
            .map(move |x| idx.and_then(|idx| x.get(idx)))
            .flatten()

    }
    /// Returns the iterator over references to the [`Value`] in to the row at index `idx`.
    ///
    /// If there is no row in [`ResultSet`] at the specified index, an empty iterator will be returned.
    pub fn get_row(&self, idx: usize) -> impl Iterator<Item=&Value>{
        self.rows
            .get(idx)
            .into_iter()
            .flatten()
    }

}

impl Display for ResultSet{

    /// Print [`ResultSet`] in the table format.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut table = Builder::new();
        let mut columns = self.columns.iter().collect::<Vec<_>>();
        columns.sort_by_key(|&(_,idx)| idx);
        for (column,_) in columns{
            table.push_column(once(column));
        }
        for row in &self.rows{
            table.push_record(row);
        }

        let mut table = table.build();

        Display::fmt(table.with(Style::modern_rounded()), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_column() {
        let mut result_set = test_result_set();
        result_set.add_column("fourth");

        assert!(result_set.get_column("fourth").eq(&[Value::Null, Value::Null, Value::Null]))
    }

    #[test]
    fn add_row_with_new_column() {
        let mut result_set = test_result_set();
        result_set.add_row([("first", Value::Number(1.into())), ("second", Value::Bool(true)), ("third", Value::Null),("fourth", Value::Bool(true))]);
        assert!(result_set.get_column("fourth").eq(&[Value::Null, Value::Null, Value::Null, Value::Bool(true)]))
    }

    #[test]
    fn print_table() {
        let result_set = test_result_set();
        println!("{}", result_set);

        assert_eq!(result_set.to_string(), [
            "╭───────┬────────┬───────╮" ,
            "│ first │ second │ third │" ,
            "├───────┼────────┼───────┤" ,
            "│ 1     │ true   │ NULL  │" ,
            "├───────┼────────┼───────┤" ,
            "│ 1     │ true   │ NULL  │" ,
            "├───────┼────────┼───────┤" ,
            "│ 1     │ true   │ NULL  │" ,
            "╰───────┴────────┴───────╯"
        ].join("\n"));
    }

    pub fn test_result_set() -> ResultSet{
        let mut result_set = ResultSet::with_columns(["first", "second", "third"]);
        result_set.add_rows([
            [("second", Value::Bool(true)), ("third", Value::Null), ("first", Value::Number(1.into()))],
            [("third", Value::Null), ("first", Value::Number(1.into())), ("second", Value::Bool(true)) ],
            [("first", Value::Number(1.into())), ("second", Value::Bool(true)), ("third", Value::Null)],
        ]);

        result_set

    }
}