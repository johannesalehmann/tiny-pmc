use super::{E, expression_parser, identifier_parser};
use crate::{ParserSpan, Token};
use chumsky::Parser;
use chumsky::input::ValueInput;
use chumsky::prelude::just;
use prism_model::{Expression, Identifier};

pub fn formula_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    prism_model::Formula<ParserSpan, Expression<Identifier<ParserSpan>, ParserSpan>>,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    just(Token::Formula)
        .ignore_then(identifier_parser())
        .then_ignore(just(Token::Equal))
        .then(expression_parser())
        .then_ignore(just(Token::Semicolon))
        .map_with(|(name, expression), e| prism_model::Formula::new(name, expression, e.span()))
        .labelled("formula")
        .as_context()
}
