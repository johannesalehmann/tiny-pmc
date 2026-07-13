use super::{expression_parser, identifier_parser, E};
use crate::parser::attributes::attributes_parser;
use crate::{ParserSpan, Token};
use chumsky::input::ValueInput;
use chumsky::prelude::just;
use chumsky::Parser;
use prism_model::{Expression, Identifier, VariableInfo, VariableScope};

pub fn const_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    prism_model::VariableInfo<ParserSpan, Expression<Identifier<ParserSpan>, ParserSpan>>,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    attributes_parser()
        .then_ignore(just(Token::Const))
        .then(variable_domain_parser().or_not().map_with(|t, e| {
            t.unwrap_or(prism_model::VariableRange::UnboundedInt { span: e.span() })
        }))
        .then(identifier_parser())
        .then(just(Token::Equal).ignore_then(expression_parser()).or_not())
        .then_ignore(just(Token::Semicolon))
        .map_with(
            |(((attributes, const_type), name), value), e| VariableInfo {
                name,
                range: const_type,
                scope: VariableScope::GlobalConstant,
                initial_value: value,
                span: e.span(),
                attributes,
            },
        )
        .labelled("constant")
        .as_context()
}
pub fn global_variable_declaration_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    prism_model::VariableInfo<ParserSpan, Expression<Identifier<ParserSpan>, ParserSpan>>,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    let init_parser = just(Token::Init).ignore_then(expression_parser());
    attributes_parser()
        .then_ignore(just(Token::Global))
        .then(identifier_parser())
        .then_ignore(just(Token::Colon))
        .then(variable_domain_parser())
        .then(init_parser.or_not())
        .then_ignore(just(Token::Semicolon))
        .map_with(
            |(((attributes, name), range), initial_value), e| VariableInfo {
                name,
                range,
                scope: VariableScope::GlobalVariable,
                initial_value,
                span: e.span(),
                attributes,
            },
        )
        .labelled("global variable declaration")
        .as_context()
}

pub fn variable_declaration_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    prism_model::VariableInfo<ParserSpan, Expression<Identifier<ParserSpan>, ParserSpan>>,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    let init_parser = just(Token::Init).ignore_then(expression_parser());
    attributes_parser()
        .then(identifier_parser())
        .then_ignore(just(Token::Colon))
        .then(variable_domain_parser())
        .then(init_parser.or_not())
        .then_ignore(just(Token::Semicolon))
        .map_with(|(((attributes, name), range), initial_value), e| {
            VariableInfo {
                // Once the module's index is known, this scope will be overwritten later on
                scope: VariableScope::GlobalVariable,
                range,
                name,
                initial_value,
                span: e.span(),
                attributes,
            }
        })
        .labelled("variable declaration")
        .as_context()
}

fn variable_domain_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    prism_model::VariableRange<ParserSpan, Expression<Identifier<ParserSpan>, ParserSpan>>,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    let range_parser = just(Token::LeftSqBracket)
        .ignore_then(
            expression_parser()
                .then_ignore(just(Token::DotDot))
                .then(expression_parser()),
        )
        .then_ignore(just(Token::RightSqBracket))
        .map_with(|(min, max), e| prism_model::VariableRange::BoundedInt {
            min,
            max,
            span: e.span(),
        });

    range_parser
        .or(just(Token::Int)
            .map_with(|_, e| prism_model::VariableRange::UnboundedInt { span: e.span() }))
        .or(just(Token::Bool)
            .map_with(|_, e| prism_model::VariableRange::Boolean { span: e.span() }))
        .or(just(Token::Double)
            .map_with(|_, e| prism_model::VariableRange::Float { span: e.span() }))
        .labelled("variable domain ([n..m], int, bool or double)")
        .as_context()
}
