use crate::query::evaluator::reflect::{Reflectable};
use crate::query::evaluator::value::Value;
use crate::query::ast::expression::{BinaryOp, BinaryOperation, Expression, Identifier, Literal, Operation, UnaryOp, UnaryOperation};
use crate::query::EvaluationError;

impl Expression{
    /// Evaluate this expression with a given `context`.
    pub fn eval<C: Reflectable + ?Sized>(&self, context: &C) -> Result<Value, EvaluationError>{
        match self {
            Expression::Identifier(identifier) => identifier.read(context),
            Expression::Literal(literal) => Ok(literal.value()),
            Expression::Operation(operation) => operation.apply(context)
        }
    }
}

impl Operation{
    /// Apply this operation with a given `context`.
    pub fn apply<C: Reflectable + ?Sized>(&self, context: &C) -> Result<Value, EvaluationError>{
        match self {
            Operation::Unary(binary_operator) => binary_operator.apply(context),
            Operation::Binary(unary_operator) => unary_operator.apply(context)
        }
    }
}

impl BinaryOperation{
    /// Apply this binary operation with a given `context`.
    pub fn apply<C: Reflectable + ?Sized>(&self, context: &C) -> Result<Value, EvaluationError>{
        let left = self.left_expression.eval(context)?;
        let right = self.right_expression.eval(context)?;

        match self.op {
            BinaryOp::Gt => Value::gt(&left, &right),
            BinaryOp::Lt => Value::lt(&left, &right),
            BinaryOp::Gte => Value::gte(&left, &right),
            BinaryOp::Lte => Value::lte(&left, &right),
            BinaryOp::Eq => Value::eq(&left, &right),
            BinaryOp::Like => Value::like(&left, &right),
            BinaryOp::And => Value::and(&left, &right),
            BinaryOp::Or => Value::or(&left, &right),
        }
    }
}

impl UnaryOperation{
    /// Apply this unary operation with a given `context`.
    pub fn apply<C: Reflectable + ?Sized>(&self, context: &C) -> Result<Value, EvaluationError>{
        let value = self.expression.eval(context)?;

        match self.op {
            UnaryOp::Not => Value::not(&value)
        }
    }
}

impl Identifier{
    /// Read the value of identifier for a given `context`.
    pub fn read<C: Reflectable + ?Sized>(&self, context: &C) -> Result<Value, EvaluationError>{
        Ok(context.get_field(&self.0)?)
    }
}

impl Literal{
    /// Convert literal to the value.
    pub fn value(&self) -> Value{
        Value::from(self)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::ast::expression::Number;
    use crate::query::evaluator::reflect::tests::TestReflect;
    use crate::query::evaluator::value::conversion::ConversionError;
    use crate::query::evaluator::value::operations::{BinaryOperationError};
    use crate::query::reflect::{ReflectError};
    use crate::query::reflect::tests::EmptyContext;

    #[test]
    fn valid_identifier() {
        let test_context = TestReflect::default();

        let identifier = Identifier("number".to_string());

        let value = identifier.read(&test_context);

        assert!(matches!(value, Ok(Value::Number(Number::Int(125)))));
    }

    #[test]
    fn invalid_identifier() {
        let test_context = TestReflect::default();

        let identifier = Identifier("no_field".to_string());

        let no_field = identifier.read(&test_context);

        assert!(matches!(no_field, Err(EvaluationError::Reflect(ReflectError::NoField(_)))));
    }

    #[test]
    fn valid_unary_operation() {
        let exp = UnaryOperation{
            expression: Expression::Literal(Literal::Bool(true)),
            op: UnaryOp::Not
        };

        let value = exp.apply(&EmptyContext);

        assert!(matches!(value, Ok(Value::Bool(false))));
    }

    #[test]
    fn invalid_unary_operation() {
        let exp = UnaryOperation{
            expression: Expression::Literal(Literal::Null),
            op: UnaryOp::Not
        };

        let value = exp.apply(&EmptyContext);

        assert!(matches!(value, Err(EvaluationError::Conversion(ConversionError::NotAllowed { .. }))));
    }

    #[test]
    fn valid_binary_operation() {
        let test_reflect = TestReflect::default();

        let exp = BinaryOperation{
            left_expression: Expression::Literal(Literal::String("Default string".to_string())),
            op: BinaryOp::Eq,
            right_expression: Expression::Identifier(Identifier("string".to_string())),
        };

        let value = exp.apply(&test_reflect);

        assert!(matches!(value, Ok(Value::Bool(true))));
    }

    #[test]
    fn invalid_binary_operation() {
        let test_reflect = TestReflect::default();

        let exp = BinaryOperation{
            left_expression: Expression::Literal(Literal::String("String".to_string())),
            op: BinaryOp::Like,
            right_expression: Expression::Identifier(Identifier("number".to_string())),
        };

        let value = exp.apply(&test_reflect);

        assert!(matches!(value, Err(EvaluationError::BinaryOperation(BinaryOperationError::Unsupported { .. }))));
    }
}