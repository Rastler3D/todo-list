use thiserror::Error;
use crate::query::EvaluationError;
use crate::query::ast::expression::{BinaryOp};
use super::Value;
use super::conversion::Type;


impl Value{

    /// Tests that `left` and `right` are equal.
    ///
    /// if `left` and `right` are of different types, they will be unified.
    pub fn eq(left: &Value, right: &Value) -> Result<Value, EvaluationError> {
        if let (Value::Null, value ) | (value, Value::Null) = (left, right){
            return Ok(Value::Bool(value.r#type() == Type::Null))
        };
        let (left, right) = Value::unify_types(left, right)?;

        Ok(Value::Bool(left == right))
    }
    /// Tests that `left` is less than or equals to `right`.
    ///
    /// if `left` and `right` are of different types, they will be unified.
    pub fn lte(left: &Value, right: &Value) -> Result<Value, EvaluationError> {
        Value::unsupported_null(left,right, BinaryOp::Lte)?;
        let (left, right) = Value::unify_types(left, right)?;

        Ok(Value::Bool(left <= right))
    }

    /// Tests that `left` is less than `right`.
    ///
    /// if `left` and `right` are of different types, they will be unified.
    pub fn lt(left: &Value, right: &Value) -> Result<Value, EvaluationError> {
        Value::unsupported_null(left,right, BinaryOp::Lt)?;
        let (left, right) = Value::unify_types(left, right)?;

        Ok(Value::Bool(left < right))
    }
    /// Tests that `left` is greater than or equals to `right`.
    ///
    /// if `left` and `right` are of different types, they will be unified.
    pub fn gte(left: &Value, right: &Value) -> Result<Value, EvaluationError> {
        Value::unsupported_null(left,right, BinaryOp::Gte)?;
        let (left, right) = Value::unify_types(left, right)?;

        Ok(Value::Bool(left >= right))
    }
    /// Tests that `left` is greater than `right`.
    ///
    /// if `left` and `right` are of different types, they will be unified.
    pub fn gt(left: &Value, right: &Value) -> Result<Value, EvaluationError> {
        Value::unsupported_null(left,right, BinaryOp::Gt)?;
        let (left, right) = Value::unify_types(left, right)?;

        Ok(Value::Bool(left > right))
    }
    /// Performs a logical "and" operation between `left` and `right`.
    ///
    /// One of the values must be a boolean. Another will be converted to bool.
    pub fn and(left: &Value, right: &Value) -> Result<Value, EvaluationError> {
        if let (Value::Bool(left), right ) | ( right , Value::Bool(left)) = (left, right){
            Ok(Value::Bool(*left && right.cast_to_bool()?))
        } else {
            return Err(BinaryOperationError::Unsupported {
                left: left.r#type(),
                right: right.r#type(),
                operator: BinaryOp::And
            }.into())
        }
    }
    /// Performs a logical "or" operation between `left` and `right`.
    ///
    /// One of the values must be a boolean. Another will be converted to bool.
    pub fn or(left: &Value, right: &Value) -> Result<Value, EvaluationError> {
        if let (Value::Bool(left), right ) | ( right , Value::Bool(left)) = (left, right){
            Ok(Value::Bool(*left || right.cast_to_bool()?))
        } else {
            return Err(BinaryOperationError::Unsupported {
                left: left.r#type(),
                right: right.r#type(),
                operator: BinaryOp::Or
            }.into())
        }
    }

    /// Performs a pattern matching between `left` and `pattern`.
    ///
    /// `pattern` must be a string. `left` value will be converted to string.
    pub fn like(left: &Value, pattern: &Value) -> Result<Value, EvaluationError> {
        if let Value::String(pattern) = pattern {
            Ok(Value::Bool(left.cast_to_string()?.contains(&*pattern)))
        } else {
            return Err(BinaryOperationError::Unsupported {
                left: left.r#type(),
                right: pattern.r#type(),
                operator: BinaryOp::Like
            }.into())
        }
    }
    /// Performs a logical "not" operation on `value`.
    ///
    /// Value will be converted to bool.
    pub fn not(value: &Value) -> Result<Value, EvaluationError> {
        Ok(Value::Bool(!value.cast_to_bool()?))
    }

    fn unsupported_null(left: &Value, right: &Value, op: BinaryOp) -> Result<(), EvaluationError> {
        if let (Value::Null, _ ) | (_, Value::Null) = (left, right){
            return Err(BinaryOperationError::Unsupported {
                left: left.r#type(),
                right: right.r#type(),
                operator: op
            }.into())
        };

        Ok(())
    }
}


/// Represents possible errors of performing a binary operation on two [`Value`]s.
#[derive(Error, Debug)]
pub enum BinaryOperationError {
    #[error("Unsupported operation '{operator}' between types '{left}' and '{right}'")]
    Unsupported { left: Type, right: Type, operator: BinaryOp },
    #[error("Failed to perform operation '{operation}' between values '{left}' and '{right}'. \nReason: {reason}")]
    Failed {
        operation: BinaryOp,
        left: Value,
        right: Value,
        reason: String,
    },
}

/// Represents possible errors of performing a unary operation on a [`Value`].
#[derive(Error, Debug)]
pub enum UnaryOperationError {
    #[error("Unsupported unary operation '{operation}' on type '{r#type}'")]
    Unsupported { r#type: Type, operation: BinaryOp },
    #[error("Failed to perform unary operation '{operation}' on value '{value}'. \nReason: {reason}")]
    Failed {
        operation: BinaryOp,
        value: Value,
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;
    use crate::query::evaluator::value::Number;
    use super::*;

    #[test]
    fn gt_same_types() {
        let left = Value::Number(Number::from(10));
        let right = Value::Number(Number::from(11));

        assert!(matches!(Value::gt(&left, &right), Ok(Value::Bool(false))));
    }

    #[test]
    fn gt_different_types() {
        let left = Value::String("2024-12-12 20:20".to_string());
        let right = Value::DateTime(NaiveDateTime::parse_from_str("2020-12-12 20:20", "%Y-%m-%d %H:%M")
            .unwrap()
            .and_utc());

        assert!(matches!(Value::gt(&left, &right), Ok(Value::Bool(true))));
    }

    #[test]
    fn gt_null() {
        let left = Value::Number(Number::from(10));
        let right = Value::Null;

        assert!(matches!(Value::gt(&left, &right), Err(EvaluationError::BinaryOperation(BinaryOperationError::Unsupported { .. }))));
    }

    #[test]
    fn eq_null() {
        let left = Value::Number(Number::from(10));
        let right = Value::Null;

        assert!(matches!(Value::eq(&left, &right), Ok(Value::Bool(false))));
    }

    #[test]
    fn and_no_bool() {
        let left = Value::String("2024-12-12 20:20".to_string());
        let right = Value::DateTime(NaiveDateTime::parse_from_str("2020-12-12 20:20", "%Y-%m-%d %H:%M")
            .unwrap()
            .and_utc());

        assert!(matches!(Value::and(&left, &right), Err(EvaluationError::BinaryOperation(BinaryOperationError::Unsupported { .. }))));
    }

    #[test]
    fn and() {
        let left = Value::Bool(true);
        let right = Value::Bool(false);

        assert!(matches!(Value::and(&left, &right), Ok(Value::Bool(false))));
    }

    #[test]
    fn like_not_string_pattern() {
        let left = Value::String("string".to_string());
        let pattern = Value::Bool(false);

        assert!(matches!(Value::like(&left, &pattern), Err(EvaluationError::BinaryOperation(BinaryOperationError::Unsupported { .. }))));
    }

    #[test]
    fn like() {
        let left = Value::String("string".to_string());
        let pattern = Value::String("str".to_string());

        assert!(matches!(Value::like(&left, &pattern), Ok(Value::Bool(true))));
    }
}