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
//! without properties, use [`parse_model()`]. Properties can be parsed with
//! [`parse_unprocessed_props()`].
//!
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
//! Continuing the initial example:
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
//  - unprocessed properties can be parsed stand-alone [DONE]
//  - processing properties requires the model to preserve its list of formulas
//  - then they can be processed given a model as context

// TODO: Use consistent name for the property inputs. Currently, they're sometimes called input
//  files, sometimes property inputs, etc. in documentation.

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
use probabilistic_properties::NamedQueries;
use std::borrow::Cow;
use std::collections::HashMap;

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
/// Each entry of `property_sources` may be a single property or a property file with multiple
/// semicolon-concatenated properties.
///
/// To get a processed model, use [`parse_model_and_props()`].
/// If you only want to parse a single property, use [`parse_unprocessed_model_and_prop()`].
/// To parse the model without any properties, use [`parse_unprocessed_model()`].
///
/// The output contains a separate `Result` for the model and each property file. Use
/// [`.all_ok()`](UnprocessedModelAndPropsResult::all_ok) to get a single result.
pub fn parse_unprocessed_model_and_props<'a>(
    source: &'a str,
    property_sources: &'a [&'a str],
) -> UnprocessedModelAndPropsResult<'a> {
    let model = parse_and_lex(source, |_| parser::program_parser().boxed());
    let properties = parse_unprocessed_props(property_sources);
    UnprocessedModelAndPropsResult { model, properties }
}

/// Parses the given properties. Each entry of `property_sources` may contain any number of named
/// or unnamed properties.
///
/// The output contains a separate `Result` for the property file. Use
/// [`.all_ok()`](UnprocessedPropsResult::all_ok) to get a single result.
///
/// *There is no parse_props*, i.e. properties cannot be processed without a model. To process
/// properties, you must parse and process a model at the same time. This is because processing
/// properties relies on the model's formulas, labels and variable manager.
pub fn parse_unprocessed_props<'a>(property_sources: &'a [&'a str]) -> UnprocessedPropsResult<'a> {
    let mut properties = NamedQueries::new();
    let mut errors = Vec::new();
    let mut input_source_to_properties = Vec::new();
    for prop_source_result in property_sources
        .iter()
        .map(|p| parse_and_lex(p.as_ref(), |_| parser::named_queries_parser().boxed()))
        .collect::<Vec<_>>()
    {
        match prop_source_result {
            Ok(props) => {
                let mut duplicate_errors = Vec::new();
                let start_index = properties.len();
                for prop in props {
                    if let Err(err) = properties.add(prop) {
                        duplicate_errors.push(
                            ValidationError::DuplicateQueryName {
                                name: err.name,
                                previous_index: Some(err.existing_index),
                            }
                            .into(),
                        )
                    }
                }
                let end_index = properties.len();
                input_source_to_properties.push(start_index..end_index);
                if duplicate_errors.is_empty() {
                    errors.push(Ok(()))
                } else {
                    errors.push(Err(duplicate_errors))
                }
            }
            Err(errs) => {
                input_source_to_properties.push(properties.len()..properties.len());
                errors.push(Err(errs))
            }
        }
    }
    UnprocessedPropsResult {
        properties,
        input_source_to_properties,
        errors,
    }
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
    let property = parse_and_lex(property, |_| parser::named_query_parser().boxed());
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
    property_sources: &'a [&'a str],
) -> ModelAndPropsResult<'a> {
    let unprocessed = parse_unprocessed_model_and_props(source, property_sources);

    // Build vector that stores source_file_index for each property
    let mut source_file_indices = Vec::new();
    for (source_file_index, range) in unprocessed
        .properties
        .input_source_to_properties
        .into_iter()
        .enumerate()
    {
        for prop_index in range {
            // We assume that input_source_to_properties contains consecutive ranges
            assert_eq!(prop_index, source_file_indices.len());
            source_file_indices.push(source_file_index);
        }
    }

    // Build vector that stores (source_file_index, query) tuples
    let mut unprocessed_properties = Vec::new();
    for (source_file_index, prop) in source_file_indices
        .into_iter()
        .zip(unprocessed.properties.properties.into_iter())
    {
        unprocessed_properties.push((source_file_index, prop))
    }

    // Substitute labels and formulas
    let partially_processed_properties: Vec<_> = unprocessed_properties
        .into_iter()
        .map(|(i, p)| {
            (
                i,
                substitute_labels_and_formulas_in_property(&unprocessed.model, Ok(p)),
            )
        })
        .collect();

    // Process the model
    let model = process_model(unprocessed.model);

    // Replace identifiers by variable indices
    let processed_properties: Vec<_> = partially_processed_properties
        .into_iter()
        .map(|(i, p)| {
            (
                i,
                replace_identifiers_by_variable_indices_in_property(&model, p),
            )
        })
        .collect();

    // Separate queries and errors and rebuild the input_source_to_properties map.
    let mut old_to_new_index = HashMap::new();
    let mut properties = NamedQueries::new();
    let mut input_source_to_properties = Vec::new();
    let mut new_errors = Vec::new(); // Collects new errors and which input_source they occurred in
    for (old_index, (input_source_index, prop)) in processed_properties.into_iter().enumerate() {
        match prop {
            Ok(prop) => {
                let new_index = properties.len();
                old_to_new_index.insert(old_index, new_index);
                properties.add(prop).unwrap(); // We can unwrap here, as name conflicts are handled in `parse_unprocessed_model_and_props` already
                while input_source_to_properties.len() <= input_source_index {
                    input_source_to_properties.push(new_index..new_index);
                }
                input_source_to_properties.last_mut().unwrap().end = new_index + 1;
            }
            Err(errs) => {
                for err in errs {
                    new_errors.push((input_source_index, err));
                }
            }
        }
    }
    // Make sure input_source_to_properties has the right length. This loop will run when the last
    // input source(s) did not contain any (valid) properties
    while input_source_to_properties.len() < property_sources.len() {
        input_source_to_properties.push(properties.len()..properties.len());
    }

    let mut errors = unprocessed.properties.errors;
    // Update the indices of DuplicateQueryName errors
    for result in &mut errors {
        if let Err(errors) = result {
            for error in errors {
                if let Error::Validation(ValidationError::DuplicateQueryName {
                    previous_index,
                    ..
                }) = error
                {
                    if let Some(i) = previous_index {
                        *previous_index = old_to_new_index.get(i).cloned();
                    }
                }
            }
        }
    }

    // Add new errors to the existing errors of the right input source
    for (input_source_index, new_error) in new_errors {
        let errors = &mut errors[input_source_index];
        if let Err(errs) = errors {
            errs.push(new_error);
        } else {
            *errors = Err(vec![new_error]);
        }
    }
    let properties = PropsResult {
        properties,
        input_source_to_properties,
        errors,
    };

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

    property.query.substitute_labels(&model.labels);
    match property.query.substitute_formulas(&model.formulas) {
        Ok(_) => Ok(property),
        Err(err) => return Err(vec![err.into()]),
    }
}

fn replace_identifiers_by_variable_indices_in_property<'a>(
    model: &Result<Model, Vec<Error<'a>>>,
    property: Result<UnprocessedQuery, Vec<Error<'a>>>,
) -> Result<Query, Vec<Error<'a>>> {
    let (model, property) = match (model, property) {
        (Ok(model), Ok(property)) => (model, property),
        (_, Err(errs)) => return Err(errs),
        (Err(_), _) => return Err(Vec::new()),
    };

    match property
        .query
        .replace_identifiers_by_variable_indices(&model.variable_manager)
    {
        Ok(query) => Ok(Query {
            name: property.name,
            query,
        }),
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
