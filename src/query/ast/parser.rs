use super::expression::{
    BinaryOp, BinaryOperation, Expression, Identifier, Literal, Number, Operation, UnaryOp,
    UnaryOperation,
};
use super::{Field, FieldsProjection, Predicate, Query};
use nom::branch::alt;
use nom::bytes::complete::{escaped, tag, tag_no_case};
use nom::character::complete::{alpha1, alphanumeric1, char, i64, multispace0, none_of, one_of};
use nom::combinator::{cut, map, not, opt, recognize, value};
use nom::error::{ParseError, VerboseError};
use nom::multi::{many0_count, separated_list1};
use nom::number::complete::double;
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::{IResult, Parser};

type ParseResult<'a, O> = IResult<&'a str, O, VerboseError<&'a str>>;

/// Skips surrounding whitespace
pub fn ws<'a, O, E: ParseError<&'a str>>(
    wrapped: impl Parser<&'a str, Output = O, Error = E>,
) -> impl Parser<&'a str, Output = O, Error = E> {
    delimited(multispace0, wrapped, multispace0)
}

pub fn literal(input: &str) -> ParseResult<Literal> {
    alt((
        map(null, |_| Literal::Null),
        map(number, Literal::Number),
        map(boolean, Literal::Bool),
        map(string, Literal::String),
    ))
    .parse(input)
}

pub fn null(input: &str) -> ParseResult<()> {
    value((), tag_no_case("null")).parse(input)
}
pub fn number(input: &str) -> ParseResult<Number> {
    alt((
        map(terminated(i64, not(one_of(".eE"))), Number::Int),
        map(double, Number::Float),
    ))
    .parse(input)
}

pub fn boolean(input: &str) -> ParseResult<bool> {
    alt((value(false, tag("false")), value(true, tag("true")))).parse(input)
}

pub fn string(input: &str) -> ParseResult<String> {
    alt((
        delimited(char('\''), escaped_single_quote_string, cut(char('\''))),
        delimited(char('"'), escaped_double_quote_string, cut(char('"'))),
    ))
    .parse(input)
}

/// Parse double-quoted string, escaping control characters
pub fn escaped_double_quote_string(input: &str) -> ParseResult<String> {
    map(
        map(
            opt(escaped(none_of(r#"\""#), '\\', one_of(r#""\/bfnrt"#))),
            Option::unwrap_or_default,
        ),
        ToString::to_string,
    )
    .parse(input)
}
/// Parse single-quoted string, escaping control characters
pub fn escaped_single_quote_string(input: &str) -> ParseResult<String> {
    map(
        map(
            opt(escaped(none_of(r#"\'"#), '\\', one_of(r#"'\/bfnrt"#))),
            Option::unwrap_or_default,
        ),
        ToString::to_string,
    )
        .parse(input)
}

pub fn identifier(input: &str) -> ParseResult<Identifier> {
    map(
        recognize(preceded(
            alt((alpha1, tag("_"))),
            many0_count(alt((alphanumeric1, tag("_")))),
        )),
        |identifier: &str| Identifier(identifier.to_string()),
    )
    .parse(input)
}

/// Parse operators with precedence 4
pub fn expression(input: &str) -> ParseResult<Expression> {
    alt((
        map(
            separated_pair(expression1, ws(tag_no_case("OR")), expression),
            |(left, right)| {
                Expression::Operation(Box::new(Operation::Binary(BinaryOperation {
                    left_expression: left,
                    op: BinaryOp::Or,
                    right_expression: right,
                })))
            },
        ),
        ws(expression1),
    ))
    .parse(input)
}

/// Parse operators with precedence 3
pub fn expression1(input: &str) -> ParseResult<Expression> {
    alt((
        map(
            separated_pair(expression2, ws(tag_no_case("AND")), expression1),
            |(left, right)| {
                Expression::Operation(Box::new(Operation::Binary(BinaryOperation {
                    left_expression: left,
                    op: BinaryOp::And,
                    right_expression: right,
                })))
            },
        ),
        ws(expression2),
    ))
    .parse(input)
}

/// Parse operators with precedence 2
pub fn expression2(input: &str) -> ParseResult<Expression> {
    alt((
        map(preceded(ws(tag_no_case("NOT")), expression2), |expr| {
            Expression::Operation(Box::new(Operation::Unary(UnaryOperation {
                op: UnaryOp::Not,
                expression: expr,
            })))
        }),
        ws(expression3),
    ))
    .parse(input)
}

/// Parse operators with precedence 1
pub fn expression3(input: &str) -> ParseResult<Expression> {
    alt((
        map(
            (expression4, ws(relation_operator), expression3),
            |(left, op, right)| {
                Expression::Operation(Box::new(Operation::Binary(BinaryOperation {
                    left_expression: left,
                    op,
                    right_expression: right,
                })))
            },
        ),
        ws(expression4),
    ))
    .parse(input)
}

/// Parse expressions in parentheses, literals and identifiers
pub fn expression4(input: &str) -> ParseResult<Expression> {
    alt((
        delimited(tag("("), ws(expression), cut(tag(")"))),
        map(literal, Expression::Literal),
        map(identifier, Expression::Identifier),
    ))
    .parse(input)
}

pub fn relation_operator(input: &str) -> ParseResult<BinaryOp> {
    alt((
        value(BinaryOp::Like, tag("LIKE")),
        value(BinaryOp::Gte, tag(">=")),
        value(BinaryOp::Gt, tag(">")),
        value(BinaryOp::Lte, tag("<=")),
        value(BinaryOp::Lt, tag("<")),
        value(BinaryOp::Eq, tag("=")),
    ))
    .parse(input)
}

/// Parse predicate
pub fn predicate(input: &str) -> ParseResult<Predicate> {
    map(expression, |expr| Predicate { expr }).parse(input)
}
/// Parse query
pub fn query(input: &str) -> ParseResult<Query> {
    map(
        ws((
            preceded(ws(tag_no_case("SELECT")), fields_projection),
            opt(preceded(ws(tag_no_case("WHERE")), predicate)),
        )),
        |(fields_projection, predicate)| Query {
            fields_projection,
            predicate,
        },
    )
    .parse(input)
}

/// Parse fields projection
pub fn fields_projection(input: &str) -> ParseResult<FieldsProjection> {
    map(separated_list1(ws(char(',')), field), FieldsProjection).parse(input)
}

pub fn field(input: &str) -> ParseResult<Field> {
    alt((
        map(identifier, Field::Name),
        value(Field::Asterisk, char('*')),
    ))
    .parse(input)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_number() {
        let input = "30.1";

        let valid = number(input);

        assert!(matches!(valid, Ok(("", Number::Float(30.1)))));

        let input = "d31.12";

        let invalid = number(input);

        assert!(matches!(invalid, Err(_)));
    }

    #[test]
    fn parse_single_quoted_string() {
        let input = "'string'";

        let valid = string(input);

        assert!(matches!(valid, Ok(("", str)) if str == "string"));

        let input = r#"'string""#;

        let invalid = string(input);

        assert!(matches!(invalid, Err(_)));
    }

    #[test]
    fn parse_double_quoted_string() {
        let input = r#""string""#;

        let valid = string(input);

        assert!(matches!(valid, Ok(("", str)) if str == "string"));

        let input = r#""string"#;

        let invalid = string(input);

        assert!(matches!(invalid, Err(_)));
    }

    #[test]
    fn parse_escaped_quoted_string() {
        let input = r#""str\"ing""#;

        let valid = string(input);

        assert!(matches!(valid, Ok(("", str)) if str == "str\\\"ing"));

        let input = r#""str"ing"#;

        let invalid = string(input);

        assert!(matches!(invalid, Ok(("ing", str)) if str == "str"));
    }

    #[test]
    fn parse_fields() {
        let input = "field1, field2, field3";

        let valid = fields_projection(input);

        assert!(matches!(valid, Ok(("", FieldsProjection(fields))) if fields.len() == 3));

        let input = r#"field1, field2, field3,"#;

        let invalid = fields_projection(input);

        assert!(matches!(invalid, Ok((",", FieldsProjection(_)))));
    }

    #[test]
    fn parse_identifier() {
        let input = "_identifier_123";

        let valid = identifier(input);

        assert!(matches!(valid, Ok(("", Identifier(_)))));

        let input = r#"123_identifier"#;

        let invalid = identifier(input);

        assert!(matches!(invalid, Err(_)));
    }

    #[test]
    fn check_operator_precedence() {
        let input = "value AND (NOT value > 1) OR value";

        let received = expression(input).unwrap().1;

        let expect = Expression::Operation(Box::new(Operation::Binary(BinaryOperation{
            op: BinaryOp::Or,
            right_expression: Expression::Identifier(Identifier("value".to_string())),
            left_expression: Expression::Operation(Box::new(Operation::Binary(BinaryOperation{
                op: BinaryOp::And,
                left_expression: Expression::Identifier(Identifier("value".to_string())),
                right_expression: Expression::Operation(Box::new(Operation::Unary(UnaryOperation{
                    op: UnaryOp::Not,
                    expression: Expression::Operation(Box::new(Operation::Binary(BinaryOperation {
                        op: BinaryOp::Gt,
                        left_expression: Expression::Identifier(Identifier("value".to_string())),
                        right_expression: Expression::Literal(Literal::Number(Number::Int(1)))
                    })))
                })))
            })))
        })));

        assert_eq!(received, expect)
    }
}