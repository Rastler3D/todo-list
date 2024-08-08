use std::str::FromStr;
use nom::combinator::all_consuming;
use nom::error::convert_error;
use nom::Finish;
use nom::Parser;
use thiserror::Error;
use crate::query::ast::expression::{Expression, Identifier};
use crate::query::ast::parser::query;

mod parser;
pub mod expression;

/// Represents a query, that will filter items by predicate and then project them to [`ResultSet`].
#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    pub fields_projection: FieldsProjection,
    pub predicate: Option<Predicate>
}

/// Fields that will be projected to [`ResultSet`].
#[derive(Clone, Debug, PartialEq)]
pub struct FieldsProjection(pub Vec<Field>);


/// One of the possible field projection type.
///
///  * `Field::Asterisk` - all fields of projectable types will be included in [`ResultSet`];
///  * `Field::Name` - specified field will be included in [`ResultSet`];
#[derive(Clone, Debug, PartialEq)]
pub enum Field{
    Asterisk,
    Name(Identifier)
}

/// Predicate that will filter values.
#[derive(Clone,Debug, PartialEq)]
pub struct Predicate{
    pub expr: Expression
}


impl FromStr for Query{
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        all_consuming(query)
            .parse(s)
            .finish()
            .map_err(|x| ParseError(convert_error(s, x)))
            .map(|(_, x)| x)
    }
}

/// Represents possible errors of query parsing.
#[derive(Error, Debug)]
#[error("Query parsing failed. Error: {0}")]
pub struct ParseError(String);