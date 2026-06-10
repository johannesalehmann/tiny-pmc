mod character_to_line;
mod error;
mod lexer;
mod outputs;
mod parser;
mod substitutable_query;

pub use outputs::*;

use crate::parser::E;
use crate::substitutable_query::SubstitutableQuery;
pub use character_to_line::CharacterToLineMap;
use chumsky::input::MappedInput;
use chumsky::prelude::*;
pub use error::{ParserError, ValidationError};
pub use lexer::{ParserSpan, Token};
use prism_model::{FullSpan, Span};
use std::borrow::Cow;

fn lex(
    source: &str,
    errors: &mut Vec<ParserError<ParserSpan, String>>,
) -> Option<Vec<lexer::Spanned<Token>>> {
    let (lexer_output, lexer_errors) = lexer::raw_lex(source).into_output_errors();
    if !lexer_errors.is_empty() {
        for error in lexer_errors {
            errors.push(error.map_token(|c| c.to_string()).into_owned())
        }
        None
    } else {
        lexer_output
    }
}

type LexedInput<'a> = MappedInput<
    Token,
    FullSpan,
    &'a [(Token, FullSpan)],
    fn(&'a (Token, FullSpan)) -> (&'a Token, &'a FullSpan),
>;

fn parse_and_lex<'b, O>(
    source: &'b str,
    make_parser: impl for<'a> Fn(&'a [(Token, FullSpan)]) -> Boxed<'a, 'a, LexedInput<'a>, O, E<'a>>,
) -> Result<O, Vec<ParserError<'b, ParserSpan, String>>> {
    let mut model_errors = Vec::new();
    if let Some(lexer_output) = lex(source, &mut model_errors) {
        let tokens = lexer_output.as_slice();
        let mapper: fn(&(Token, FullSpan)) -> (&Token, &FullSpan) = |(t, s)| (t, s);
        let (output, parse_errors) = make_parser(tokens)
            .map_with(|ast, e| (ast, e.span()))
            .parse(tokens.map(
                ParserSpan::from_start_end(source.len(), source.len()),
                mapper,
            ))
            .into_output_errors();
        process_parser_errors(&mut model_errors, parse_errors);
        let output = output.map(|(o, _)| o);
        if let Some(output) = output
            && model_errors.is_empty()
        {
            Ok(output)
        } else {
            Err(model_errors)
        }
    } else {
        Err(model_errors)
    }
}

pub fn parse_unprocessed_model_and_props<'a>(
    source: &'a str,
    properties: &'a [&'a str],
) -> UnprocessedModelAndPropsResult<'a> {
    let model = parse_and_lex(source, |_| parser::program_parser().boxed());
    let properties = properties
        .iter()
        .map(|p| parse_and_lex(p.as_ref(), |_| parser::query_parser().boxed()))
        .collect::<Vec<_>>();
    UnprocessedModelAndPropsResult { model, properties }
}

pub fn parse_unprocessed_model_and_prop<'a>(
    source: &'a str,
    property: &'a str,
) -> UnprocessedModelAndPropResult<'a> {
    let model = parse_and_lex(source, |_| parser::program_parser().boxed());
    let property = parse_and_lex(property, |_| parser::query_parser().boxed());
    UnprocessedModelAndPropResult { model, property }
}

pub fn parse_unprocessed_model<'a>(source: &'a str) -> Result<UnprocessedModel, Vec<Error<'a>>> {
    parse_and_lex(source, |_| parser::program_parser().boxed())
}

pub fn parse_model_and_props<'a>(
    source: &'a str,
    properties: &'a [&'a str],
) -> ModelAndPropsResult<'a> {
    let unprocessed = parse_unprocessed_model_and_props(source, properties);
    let properties = unprocessed
        .properties
        .into_iter()
        .map(|p| process_property(&unprocessed.model, p))
        .collect();
    let model = process_model(unprocessed.model);
    ModelAndPropsResult { model, properties }
}

pub fn parse_model_and_prop<'a>(source: &'a str, property: &'a str) -> ModelAndPropResult<'a> {
    let unprocessed = parse_unprocessed_model_and_prop(source, property);
    let property = process_property(&unprocessed.model, unprocessed.property);
    let model = process_model(unprocessed.model);
    ModelAndPropResult { model, property }
}

pub fn parse_model<'a>(source: &'a str) -> Result<Model, Vec<Error<'a>>> {
    let unprocessed = parse_unprocessed_model(source);
    process_model(unprocessed)
}

fn process_model(model: Result<UnprocessedModel, Vec<Error>>) -> Result<Model, Vec<Error>> {
    match model {
        Err(err) => Err(err),
        Ok(mut model) => {
            match model.substitute_formulas() {
                Ok(_) => (),
                Err(err) => return Err(vec![err.into()]),
            };
            match model.expand_renamed_modules() {
                Ok(_) => (),
                Err(err) => return Err(vec![err.into()]),
            };
            match model.replace_identifiers_by_variable_indices() {
                Ok(model) => Ok(model),
                Err(err) => Err(err.into_iter().map(|e| e.into()).collect()),
            }
        }
    }
}

fn process_property<'a>(
    model: &Result<UnprocessedModel, Vec<Error<'a>>>,
    property: Result<UnprocessedQuery, Vec<Error<'a>>>,
) -> Result<Query, Vec<Error<'a>>> {
    let (model, mut property) = match (model, property) {
        (Ok(model), Ok(property)) => (model, property),
        (_, Err(errs)) => return Err(errs),
        (Err(_), _) => return Err(Vec::new()),
    };

    property.substitute_labels(&model.labels);
    match property.substitute_formulas(&model.formulas) {
        Ok(_) => (),
        Err(err) => return Err(vec![err.into()]),
    }

    match property.replace_identifiers_by_variable_indices(&model.variable_manager) {
        Ok(property) => Ok(property),
        Err(err) => Err(err.into_iter().map(|e| e.into()).collect()),
    }
}

fn process_parser_errors(
    errors: &mut Vec<ParserError<ParserSpan, String>>,
    parse_errors: Vec<ParserError<ParserSpan, Token>>,
) {
    for mut error in parse_errors {
        if let ParserError::ExpectedFound {
            expected,
            contexts,
            help,
            ..
        } = &mut error
        {
            // If a reserved keyword is used in a declaration, an understandable error is
            // emitted, but if the same keyword is used in an expression, this instead
            // produces the error "exected (, found ...)", because the reserved keyword is
            // treated as the first part of a function declaration. To make this error less
            // confusing, the add some context here:
            if expected.len() == 1
                && expected[0]
                    == chumsky::error::RichPattern::Token(chumsky::util::Maybe::Val(
                        Token::LeftBracket,
                    ))
                && !contexts.is_empty()
                && contexts.first().unwrap().0
                    == chumsky::error::RichPattern::Label(Cow::Borrowed("expression"))
            {
                *help = Some(
                    "This error is often caused by using variables with reserved names".to_string(),
                );
            }
        }
        errors.push(error.map_token(|t| format!("{}", t)).into_owned())
    }
}
