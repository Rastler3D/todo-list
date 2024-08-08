use std::fmt::{Display, Formatter};

pub use crate::query::evaluator::value::Number;

/// Expression that can be evaluated to [`Value`]
#[derive(Clone,Debug, PartialEq)]
pub enum Expression{
    Identifier(Identifier),
    Literal(Literal),
    Operation(Box<Operation>)
}

/// Name of the identifier that can be read from the type that implement [`Reflectable`].
#[derive(Clone,Debug, PartialEq)]
pub struct Identifier(pub String);

/// Possible literals.
#[derive(Clone,Debug, PartialEq)]
pub enum Literal{
    Number(Number),
    String(String),
    Bool(bool),
    Null
}

/// Expression operations.
#[derive(Clone,Debug, PartialEq)]
pub enum Operation{
    Unary(UnaryOperation),
    Binary(BinaryOperation)
}
/// Unary operation that can be evaluated to [`Value`].
#[derive(Clone,Debug, PartialEq)]
pub struct UnaryOperation{
    pub expression: Expression,
    pub op: UnaryOp
}

/// Possible unary operators.
#[derive(Clone,Debug, PartialEq)]
pub enum UnaryOp{
    Not
}

/// Binary operation that can be evaluated to [`Value`].
#[derive(Clone,Debug, PartialEq)]
pub struct BinaryOperation{
    pub left_expression: Expression,
    pub op: BinaryOp,
    pub right_expression: Expression
}

/// Possible binary operators.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BinaryOp{
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    Like,
    And,
    Or
}

impl Display for BinaryOp{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            BinaryOp::Gt => ">",
            BinaryOp::Lt => "<",
            BinaryOp::Gte => ">=",
            BinaryOp::Lte => "<=",
            BinaryOp::Eq => "=",
            BinaryOp::Like => "LIKE",
            BinaryOp::And => "AND",
            BinaryOp::Or => "OR"
        };

        Display::fmt(value, f)
    }
}