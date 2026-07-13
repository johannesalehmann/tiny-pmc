#![warn(missing_docs)]
//! Parser for the probabilistic modelling language PRISM.
//!
//! # Features
//!
//! - parse Markov decision processes (MDPs), discrete-time Markov chains (DTMCs) and more!
//! - parse pCTL objectives
//! - expand formulas, labels and renamed modules (or [get the raw model](parse_unprocessed_model_and_props))
//! - high-quality error messages
//!
//! # Usage
//!
//! To parse a model and several properties at once, use [`parse_model_and_props()`]:
//!
//! ```
//! use prism_parser::{parse_model_and_props, Query};
//! use probabilistic_properties::NonDeterminismKind;
//! let source = r#"
//! mdp
//!
//! const int N = 10;
//! const double p = 0.5;
//! label "goal" = x=N;
//!
//! module main
//!     x: [0..N] init N / 2;
//!     [alpha] (x < N) -> (x'=x+1);
//!     [beta] (x >= 2 & x <= N-2) -> p: (x'=x+2) + (1-p): (x'=x-2);
//! endmodule"#;
//!
//! let obj1 = "Pmin=? [F \"goal\"]";
//! let obj2 = "Pmax>0.7 [G x > 0]";
//! let objs = &[obj1, obj2];
//!
//! let parsed = parse_model_and_props(source, objs);
//! match parsed.model {
//!     Ok(model) => {
//!         println!("The model's first module is called: {}", model.modules.get(0).unwrap().name)
//!     },
//!     Err(errs) => {
//!         panic!("Failed to parse the model: {:?}", errs)
//!     }
//! }
//!
//! for property in parsed.properties {
//!     match property {
//!         Ok(prop) => {
//!             // Do something cool with the property!
//!         },
//!         Err(errs) => {
//!             panic!("Failed to parse properties: {:?}", errs)
//!         }
//!     }
//! }
//! ```
//!
//!
//! To parse a model with a single property, use [`parse_model_and_prop()`]. To parse a model
//! without properties, use [`parse_model()`].
//!
//! A property cannot be parsed without a model. This limitation will be lifted in future versions.
//! # Processed and unprocessed models
//!
//! After parsing, the following operations are applied:
//! - formulas are expanded
//! - labels are expanded (only in properties)
//! - renamed modules are expanded
//! - in expressions, `Identifier` is replaced by `VariableReference`
//!
//! During this process, cyclic dependencies in formulas, incorrect renaming rules and undeclared
//! variables are detected.
//!
//! If you need the raw model without these transformations, use
//! [`parse_unprocessed_model_and_props()`].
//!
//! The individual transformations are exposed by `prism-model` and can be applied individually.
//! Note however that they have to be applied in the above order to preserve PRISM semantics (for
//! example, formulas must be expanded before expanding renamed modules).
//!
//! # Results
//!
//! When parsing a model and list of properties, it is possible that parsing the model succeeds,
//! while one of the properties contains an error[^1]. The model and properties are given in
//! separate `Result`s to model this.
//!
//! If you want a single `Result`, use `parse_model_and_props().`[`all_ok()`](ModelAndPropsResult::all_ok()),
//! which either returns `Ok(`[`ModelAndProps`]`)` or a list of all errors with [`ErrorSource`].
//!
//! # Printing errors
//!
//! With the feature `pretty-print` (enabled by default), high-quality errors can be printed to
//! stdout. Consider the following program:
//!
//! ```
//! # use prism_parser::{parse_model};
//! let filename = "model.prism";
//! let source = r#"
//! mdp
//! const int n = 10;
//! module main
//!     n: [0..15] init 3;
//! endmodule
//! "#;
//! println!("Parsing");
//! match parse_model(source) {
//!     Ok(model) => { /* do something */ }
//!     Err(errs) => {
//!         for err in errs {
//!             err.print(Some(filename), source);
//!         }
//!     }
//! }
//! ```
//!
//! This produces the following error:
//! ```cli
//! Error: Duplicate name
//!    ╭─[ model.prism:5:5 ]
//!    │
//!  3 │ const int n = 10;
//!    │ ────────┬────────
//!    │         ╰────────── First defined here
//!    │
//!  5 │     n: [0..15] init 3;
//!    │     ─────────┬────────
//!    │              ╰────────── Defined again here
//! ───╯
//! ```
//!
//! # Spans
//!
//! The output is *spanned*: Every component has a span that stores which section of source code
//! corresponds to this section of the program.
//!
//! Continuing the above example:
//!
//! ```
//! # use prism_parser::{parse_model_and_props, Query};
//! # use probabilistic_properties::NonDeterminismKind;
//! # let source = r#"
//! # mdp
//! #
//! # const int N = 10;
//! # const double p = 0.5;
//! # label "goal" = x=N;
//! #
//! # module main
//! #     x: [0..N] init N / 2;
//! #     [alpha] (x < N) -> (x'=x+1);
//! #     [beta] (x >= 2 & x <= N-2) -> p: (x'=x+2) + (1-p): (x'=x-2);
//! # endmodule"#;
//! # let objs = &[];
//! # let parsed = parse_model_and_props(source, objs);
//! use prism_model::Span;
//! let model = parsed.model.expect("Failed to parse model");
//!
//! let n = model.variable_manager.get_by_str("N").unwrap();
//!
//! let n_span = &n.span;
//! assert_eq!(Some(6..23), n_span.range());
//! assert_eq!("const int N = 10;", &source[n_span.range().unwrap()]);
//! ```
//!
//! ## Characters to lines
//!
//! Spans count characters from the document's start. If you need to convert this into a line
//! number, consider using [`CharacterToLineMap`].
//!
//! [^1]: Conversely, a property cannot be processed if the model failed to parse, because the
//!       property relies on the model for variable declarations, etc.
//!
//!       Therefore, [`parse_model_and_props()`] and [`parse_model_and_prop()`] will return errors
//!       for all properties if the model failed to parse.
//!
//!       On the other hand [`parse_unprocessed_model_and_props()`] and
//!       [`parse_unprocessed_model_and_prop()`] may return an error for the model, but a
//!       successfully parsed property.

// TODO: Support property parsing without model:
//  - unprocessed properties can be parsed stand-alone
//  - processing properties requires the model to preserve its list of formulas
//  - then they can be processed given a model as context

mod character_to_line;
mod error;
mod lexer;
mod outputs;
mod parser;
mod substitutable_query;

#[cfg(test)]
#[allow(missing_docs)]
mod tests;

pub use outputs::*;

use crate::parser::E;
use crate::substitutable_query::SubstitutableQuery;
pub use character_to_line::CharacterToLineMap;
use chumsky::input::MappedInput;
use chumsky::prelude::*;
pub use error::{ElementKind, ParserError, ValidationError};
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

/// Parses a model and list of properties, without [doing processing](self#processed-and-unprocessed-models).
///
/// To get a processed model, use [`parse_model_and_props()`].
/// If you only want to parse a single property, use [`parse_unprocessed_model_and_prop()`].
/// To parse the model without any properties, use [`parse_unprocessed_model()`].
///
/// The output contains a separate `Result` for the model and each property. Use
/// [`.all_ok()`](UnprocessedModelAndPropsResult::all_ok) to get a single result.
// TODO: Support parsing property files with multiple entries. This requires
//  - a new parser function that repeatedly calls the property parser
//  - more complex property-to-source maps as there is no longer a one-to-one correspondence
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

/// Parses a model and a single property, without [doing processing](self#processed-and-unprocessed-models).
///
/// To get a processed model, use [`parse_model_and_prop()`].
/// To parse multiple properties at once, use [`parse_unprocessed_model_and_props()`].
/// To parse the model without any properties, use [`parse_unprocessed_model()`].
///
/// The output contains a separate `Result` for the model and the property. Use
/// [`.all_ok()`](UnprocessedModelAndPropResult::all_ok) to get a single result.
pub fn parse_unprocessed_model_and_prop<'a>(
    source: &'a str,
    property: &'a str,
) -> UnprocessedModelAndPropResult<'a> {
    let model = parse_and_lex(source, |_| parser::program_parser().boxed());
    let property = parse_and_lex(property, |_| parser::query_parser().boxed());
    UnprocessedModelAndPropResult { model, property }
}

/// Parses a model without [doing processing](self#processed-and-unprocessed-models) and without
/// any properties.
///
/// To also parse properties, use [`parse_unprocessed_model_and_props()`] or
/// [`parse_unprocessed_model_and_prop()`]. To get a processed model, use [`parse_model()`].
pub fn parse_unprocessed_model<'a>(source: &'a str) -> Result<UnprocessedModel, Vec<Error<'a>>> {
    parse_and_lex(source, |_| parser::program_parser().boxed())
}

/// Parses a model and list of properties and [processes the result](self#processed-and-unprocessed-models).
///
/// To get a "raw" model without processing, use [`parse_unprocessed_model_and_props()`].
/// If you only want to parse a single property, use [`parse_model_and_prop()`].
/// To parse the model without any properties, use [`parse_model()`].
///
/// The output contains a separate `Result` for the model and each property. Use
/// [`.all_ok()`](ModelAndPropsResult::all_ok) to get a single result.
pub fn parse_model_and_props<'a>(
    source: &'a str,
    properties: &'a [&'a str],
) -> ModelAndPropsResult<'a> {
    let unprocessed = parse_unprocessed_model_and_props(source, properties);
    let properties: Vec<_> = unprocessed
        .properties
        .into_iter()
        .map(|p| substitute_labels_and_formulas_in_property(&unprocessed.model, p))
        .collect();
    let model = process_model(unprocessed.model);
    let properties = properties
        .into_iter()
        .map(|p| replace_identifiers_by_variable_indices_in_property(&model, p))
        .collect();
    ModelAndPropsResult { model, properties }
}

/// Parses a model and a single property and [processes the result](self#processed-and-unprocessed-models).
///
/// To get a "raw" model without processing, use [`parse_unprocessed_model_and_prop()`].
/// To parse multiple properties at once, use [`parse_model_and_props()`].
/// To parse the model without any properties, use [`parse_model()`].
///
/// The output contains a separate `Result` for the model and the property. Use
/// [`.all_ok()`](ModelAndPropResult::all_ok) to get a single result.
pub fn parse_model_and_prop<'a>(source: &'a str, property: &'a str) -> ModelAndPropResult<'a> {
    let unprocessed = parse_unprocessed_model_and_prop(source, property);
    let property =
        substitute_labels_and_formulas_in_property(&unprocessed.model, unprocessed.property);
    let model = process_model(unprocessed.model);
    let property = replace_identifiers_by_variable_indices_in_property(&model, property);
    ModelAndPropResult { model, property }
}

/// Parses a model and [processes the result](self#processed-and-unprocessed-models), without any
/// properties.
///
/// To get a "raw" model without processing, use [`parse_unprocessed_model()`].
/// To also parse properties, use [`parse_model_and_props()`] or [`parse_model_and_prop()`].
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

fn substitute_labels_and_formulas_in_property<'a>(
    model: &Result<UnprocessedModel, Vec<Error<'a>>>,
    property: Result<UnprocessedQuery, Vec<Error<'a>>>,
) -> Result<UnprocessedQuery, Vec<Error<'a>>> {
    let (model, mut property) = match (model, property) {
        (Ok(model), Ok(property)) => (model, property),
        (_, Err(errs)) => return Err(errs),
        (Err(_), _) => return Err(Vec::new()),
    };

    property.substitute_labels(&model.labels);
    match property.substitute_formulas(&model.formulas) {
        Ok(_) => Ok(property),
        Err(err) => return Err(vec![err.into()]),
    }
}

fn replace_identifiers_by_variable_indices_in_property<'a>(
    model: &Result<Model, Vec<Error<'a>>>,
    property: Result<UnprocessedQuery, Vec<Error<'a>>>,
) -> Result<Query, Vec<Error<'a>>> {
    let (model, mut property) = match (model, property) {
        (Ok(model), Ok(property)) => (model, property),
        (_, Err(errs)) => return Err(errs),
        (Err(_), _) => return Err(Vec::new()),
    };

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
            // produces the error "expected (, found ...)", because the reserved keyword is
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
