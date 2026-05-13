use super::E;
use crate::{ParserSpan, PrismParserValidationError, Token};
use chumsky::input::ValueInput;
use chumsky::{Parser, select};

pub fn identifier_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::Identifier<ParserSpan>, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    (select! {
        Token::Identifier(name) = e => prism_model::Identifier::new_spanned::<String>(name.clone(), e.span()),
        Token::T = e => prism_model::Identifier::new_spanned::<String>("T".to_string(), e.span())
    })
        .try_map_with(|i, e|
            i.map_err(|reason| PrismParserValidationError::InvalidIdentifierName { span: e.span(), reason }.into()))
        .labelled("identifier")
}

pub fn identifier_parser_potentially_reserved<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::Identifier<ParserSpan>, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    select! {
        Token::Identifier(name) = e => prism_model::Identifier::new_potentially_reserved_spanned::<String>(name.clone(), e.span()),
        Token::Min = e => prism_model::Identifier::new_potentially_reserved_spanned::<String>("min".to_string(), e.span()),
        Token::Max = e => prism_model::Identifier::new_potentially_reserved_spanned::<String>("max".to_string(), e.span()),
        Token::T = e => prism_model::Identifier::new_potentially_reserved_spanned::<String>("T".to_string(), e.span())
    }
        .try_map_with(|i, e|
            i.map_err(|reason| PrismParserValidationError::InvalidIdentifierName { span: e.span(), reason }.into()))
        .labelled("identifier")
}
