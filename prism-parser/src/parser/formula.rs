use super::{expression_parser, identifier_parser, E};
use crate::parser::attributes::attributes_parser;
use crate::{ParserSpan, Token};
use chumsky::input::ValueInput;
use chumsky::prelude::just;
use chumsky::Parser;
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
    attributes_parser()
        .then_ignore(just(Token::Formula))
        .then(identifier_parser())
        .then_ignore(just(Token::Equal))
        .then(expression_parser())
        .then_ignore(just(Token::Semicolon))
        .map_with(|((attributes, name), condition), e| prism_model::Formula {
            name,
            condition,
            span: e.span(),
            attributes,
        })
        .labelled("formula")
        .as_context()
}
