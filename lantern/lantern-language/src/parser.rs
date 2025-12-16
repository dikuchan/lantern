use chumsky::input::{Stream, ValueInput};
use chumsky::prelude::*;
use chumsky::Parser;

use crate::ast::{BinaryOperator, Command, Expression, Query};
use crate::lexer::{tokenizer, Token};
use crate::parser_error::ParserError;
use crate::span::Span;

pub struct QueryParser;

pub fn parse(source: &'_ str) -> Result<Query, ParserError<'_>> {
    let input = new_input(source);
    query_parser()
        .parse(input)
        .into_result()
        .map_err(|errors| ParserError::new(errors))
}

pub fn check(source: &'_ str) -> Result<(), ParserError<'_>> {
    let input = new_input(source);
    query_parser()
        .check(input)
        .into_result()
        .map_err(|errors| ParserError::new(errors))
}

fn new_input(source: &'_ str) -> impl ValueInput<'_, Token = Token<'_>, Span = Span> {
    let tokens = tokenizer(source);
    Stream::from_iter(tokens).map((0..source.len()).into(), |(token, span)| (token, span))
}

fn query_parser<'tokens, 'source: 'tokens, I>()
-> impl Parser<'tokens, I, Query, extra::Err<Rich<'tokens, Token<'source>, Span>>>
where
    I: ValueInput<'tokens, Token = Token<'source>, Span = Span>,
{
    let command = command_parser();

    just(Token::KeywordSource)
        .ignore_then(select! { Token::Identifier(i) => i.to_owned() })
        .then(just(Token::Pipe).ignore_then(command).repeated().collect())
        .map(|(source, commands)| Query { source, commands })
}

fn command_parser<'tokens, 'source: 'tokens, I>()
-> impl Parser<'tokens, I, Command, extra::Err<Rich<'tokens, Token<'source>, Span>>>
where
    I: ValueInput<'tokens, Token = Token<'source>, Span = Span>,
{
    let expression = expression_parser();

    let identifier = select! { Token::Identifier(i) => i.to_string() };
    let aggregation_item = choice((
        identifier
            .then_ignore(just(Token::OperatorAssign))
            .then(expression.clone())
            .map(|(alias, expression)| (expression, Some(alias))),
        expression.clone().map(|expr| (expr, None)),
    ));

    let by_clause = just(Token::KeywordBy)
        .ignore_then(
            expression
                .clone()
                .separated_by(just(Token::Comma))
                .collect(),
        )
        .or_not()
        .map(|option| option.unwrap_or_default());

    let command_aggregate = just(Token::KeywordAggregate)
        .ignore_then(aggregation_item.separated_by(just(Token::Comma)).collect())
        .then(by_clause)
        .map(|(aggregates, by)| Command::Aggregate { aggregates, by });

    let command_where = just(Token::KeywordWhere)
        .ignore_then(expression.clone())
        .map(Command::Where);
    let command_limit = just(Token::KeywordLimit)
        .ignore_then(select! { Token::Integer(n) => n })
        .map(Command::Limit);

    choice((command_where, command_limit, command_aggregate))
}

fn expression_parser<'tokens, 'source: 'tokens, I>()
-> impl Parser<'tokens, I, Expression, extra::Err<Rich<'tokens, Token<'source>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'source>, Span = Span>,
{
    let identifier = select! { Token::Identifier(i) => i.to_string() };
    let number = select! { Token::Integer(n) => Expression::Number(n as f64) };
    let string_literal = select! { Token::StringLiteral(s) => Expression::String(s.to_owned()) };

    recursive(|expression| {
        let call = identifier
            .then_ignore(just(Token::LeftParenthesis))
            .then(
                expression
                    .clone()
                    .separated_by(just(Token::Comma))
                    .collect(),
            )
            .then_ignore(just(Token::RightParenthesis))
            .map(|(name, arguments)| Expression::Call(name, arguments));

        let field = identifier.map(Expression::Field);

        let atom = choice((
            number,
            string_literal,
            call,
            field,
            expression.delimited_by(just(Token::LeftParenthesis), just(Token::RightParenthesis)),
        ));

        let product = atom.clone().foldl(
            just(Token::OperatorMultiply)
                .to(BinaryOperator::Multiply)
                .or(just(Token::OperatorDivide).to(BinaryOperator::Divide))
                .then(atom)
                .repeated(),
            |l, (operator, r)| Expression::Binary(operator, Box::new(l), Box::new(r)),
        );

        let sum = product.clone().foldl(
            just(Token::OperatorAdd)
                .to(BinaryOperator::Add)
                .or(just(Token::OperatorSubtract).to(BinaryOperator::Subtract))
                .then(product)
                .repeated(),
            |l, (operator, r)| Expression::Binary(operator, Box::new(l), Box::new(r)),
        );

        let comparison = sum.clone().foldl(
            just(Token::OperatorEqual)
                .to(BinaryOperator::Equal)
                .or(just(Token::OperatorNotEqual).to(BinaryOperator::NotEqual))
                .or(just(Token::OperatorGreaterThan).to(BinaryOperator::GreaterThan))
                .or(just(Token::OperatorGreaterThanOrEqual).to(BinaryOperator::GreaterThanOrEqual))
                .or(just(Token::OperatorLessThan).to(BinaryOperator::LessThan))
                .or(just(Token::OperatorLessThanOrEqual).to(BinaryOperator::LessThanOrEqual))
                .then(sum)
                .repeated(),
            |l, (operator, r)| Expression::Binary(operator, Box::new(l), Box::new(r)),
        );

        let logical = comparison.clone().foldl(
            just(Token::OperatorAnd)
                .to(BinaryOperator::And)
                .or(just(Token::OperatorOr).to(BinaryOperator::Or))
                .then(comparison)
                .repeated(),
            |l, (operator, r)| Expression::Binary(operator, Box::new(l), Box::new(r)),
        );

        logical
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(source: &str) -> Query {
        match parse(source) {
            Ok(pipeline) => pipeline,
            Err(error) => {
                error.eprint(source).unwrap();
                panic!("Parse failed for input: '{}'", source);
            }
        }
    }

    macro_rules! test_snapshots {
        ( $($name:ident: $input:expr),* $(,)? ) => {
            $(
                #[test]
                fn $name() {
                    let input = $input;
                    let ast = parse_ok(input);
                    insta::assert_debug_snapshot!(ast);
                }
            )*
        }
    }

    test_snapshots! {
        basic_source:
            "source test",

        basic_literal_filter:
            "source test | where status == 200",

        math_precedence:
            "source test | where a + b * c > 10",

        parenthesis_precedence:
            "source test | where (a or b) and c",

        string_quoting:
            r#"source test | where name == "O'Conner" "#,
    }

    #[test]
    fn test_should_fail() {
        let input = "source |";
        let ast = parse(input);
        assert!(ast.is_err());

        insta::assert_debug_snapshot!(ast.unwrap_err());
    }
}
