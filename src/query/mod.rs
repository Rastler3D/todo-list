pub mod evaluator;
pub mod ast;

use thiserror::Error;
use crate::query::evaluator::value::operations::{BinaryOperationError, UnaryOperationError};
use crate::query::evaluator::value::conversion::ConversionError;
use crate::query::reflect::ReflectError;

pub use evaluator::reflect;
pub use evaluator::result_set::ResultSet;
pub use ast::{Query};

/// Represents possible errors of expression evaluation
#[derive(Debug, Error)]
pub enum EvaluationError{
    #[error(transparent)]
    Reflect(#[from] ReflectError),
    #[error(transparent)]
    Conversion(#[from] ConversionError),
    #[error(transparent)]
    BinaryOperation(#[from] BinaryOperationError),
    #[error(transparent)]
    UnaryOperation(#[from] UnaryOperationError)
}