use super::{E, expression_parser, identifier_parser};
use crate::parser::attributes::attributes_parser;
use crate::{ParserSpan, Token};
use chumsky::Parser;
use chumsky::input::ValueInput;
use chumsky::prelude::just;
use prism_model::{Expression, Identifier};

pub fn label_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    prism_model::Label<ParserSpan, Expression<Identifier<ParserSpan>, ParserSpan>>,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    attributes_parser()
        .then_ignore(just(Token::Label))
        .then(identifier_parser().delimited_by(just(Token::Quote), just(Token::Quote)))
        .then_ignore(just(Token::Equal))
        .then(expression_parser())
        .then_ignore(just(Token::Semicolon))
        .map_with(|((attributes, name), condition), e| prism_model::Label {
            name,
            condition,
            span: e.span(),
            attributes,
        })
        .labelled("label")
        .as_context()
}
