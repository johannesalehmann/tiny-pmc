use super::{E, expression_parser};
use crate::{ParserSpan, Token};
use chumsky::Parser;
use chumsky::input::ValueInput;
use chumsky::prelude::just;
use prism_model::Identifier;

pub fn init_constraint_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    (
        prism_model::Expression<Identifier<ParserSpan>, ParserSpan>,
        ParserSpan,
    ),
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    just(Token::Init)
        .ignore_then(expression_parser())
        .then_ignore(just(Token::EndInit))
        .map_with(|i, e| (i, e.span()))
        .labelled("init constraint")
        .as_context()
}
