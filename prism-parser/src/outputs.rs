use crate::{ParserError, ParserSpan};
use prism_model::{Expression, ExpressionNamedVars, VariableReference};
use std::ops::Range;
// TODO: Consistently call queries "queries" and not "properties"

/// The PRISM model produced by the parser after [processing](crate#processed-and-unprocessed-models).
pub type Model = prism_model::Model<VariableReference, ParserSpan>;

/// The PRISM model produced by the parser without [processing](crate#processed-and-unprocessed-models).
pub type UnprocessedModel = prism_model::ModelNamedVars<ParserSpan>;

type Expr = Expression<VariableReference, ParserSpan>;
type UnprocessedExpr = ExpressionNamedVars<ParserSpan>;

/// A query (also known as objective or property) produced by the parser after
/// [processing](crate#processed-and-unprocessed-models).
pub type Query = probabilistic_properties::NamedQuery<Expr, Expr, Expr>;

/// A query (also known as objective or property) produced by the parser without
/// [processing](crate#processed-and-unprocessed-models).
pub type UnprocessedQuery =
    probabilistic_properties::NamedQuery<UnprocessedExpr, UnprocessedExpr, UnprocessedExpr>;

/// A collection of queries (also known as objectives or properties) produced by the parser after
/// [processing](crate#processed-and-unprocessed-models).
pub type Queries = probabilistic_properties::NamedQueries<Expr, Expr, Expr>;

/// A collection of queries (also known as objectives or properties) produced by the parser without
/// [processing](crate#processed-and-unprocessed-models).
pub type UnprocessedQueries =
    probabilistic_properties::NamedQueries<UnprocessedExpr, UnprocessedExpr, UnprocessedExpr>;

/// The error type produced by the parser
pub type Error<'a> = ParserError<'a, ParserSpan, String>;

/// Contains parsing results for a model and several properties.
///
/// The model and each property input file have a separate `Result`, allowing fine-grained error
/// handling. Use [`.all_ok()`](ModelAndPropsResult::all_ok()) to transform this into a single
/// result.
pub struct ModelAndPropsResult<'a, M = Model, I = Expr, F = Expr, E = Expr> {
    /// The parsed model, or a list of errors encountered while parsing.
    pub model: Result<M, Vec<Error<'a>>>,

    /// Collection of all queries that were successfully parsed.
    pub properties: PropsResult<'a, I, F, E>,
}
impl<'a, M, I, F, E> ModelAndPropsResult<'a, M, I, F, E> {
    /// If both the model and all properties are `Ok(...)`, returns the model and properties.
    ///
    /// If the model or at least one property is `Err(...)`, returns an accumulated list of errors.
    /// Each error is enriched with an [`ErrorSource`] marking whether it was produced while parsing
    /// the model or one of the properties.
    pub fn all_ok(self) -> Result<ModelAndProps<M, I, F, E>, Vec<ErrorWithSource<'a>>> {
        let (model, mut errors) = match self.model {
            Ok(model) => (Some(model), Vec::new()),
            Err(err) => (
                None,
                err.into_iter().map(|e| ErrorWithSource::model(e)).collect(),
            ),
        };
        match self.properties.all_ok() {
            Ok(properties) => {
                if let Some(model) = model {
                    Ok(ModelAndProps { model, properties })
                } else {
                    Err(errors)
                }
            }
            Err(mut errs) => {
                errors.append(&mut errs);
                Err(errors)
            }
        }
    }
}

/// A collection of properties and associated errors, grouped by input sources.
pub struct PropsResult<'a, I, F, E> {
    /// All successfully parsed properties.
    ///
    /// A single input source may produce multiple properties. To determine which property originated
    /// from which input source, use the ranges in [`property_error`](Self::property_errors).
    pub properties: probabilistic_properties::NamedQueries<I, F, E>,

    /// For each input source, stores `n..m` such that  [`properties`](Self::properties)`[n..]`
    /// contains the properties that were produced by this input source
    pub input_source_to_properties: Vec<Range<usize>>,

    /// For each input source, stores either `Ok(())` or `Err(errors)`, containing all errors
    /// produced when parsing this input source.
    ///
    /// Use `Self::all_ok()` to get a flat list of all errors.
    pub errors: Vec<Result<(), Vec<Error<'a>>>>,
}

impl<'a, I, F, E> PropsResult<'a, I, F, E> {
    /// If all properties are `Ok(...)`, returns the properties.
    ///
    /// If at least one property is `Err(...)`, returns an accumulated list of errors.
    /// Each error is enriched with an [`ErrorSource`] marking whether it was produced while parsing
    /// the model or one of the properties.
    pub fn all_ok(
        self,
    ) -> Result<probabilistic_properties::NamedQueries<I, F, E>, Vec<ErrorWithSource<'a>>> {
        let mut errors = Vec::new();
        let mut all_ok = true;
        for (property_index, prop) in self.errors.into_iter().enumerate() {
            if let Err(errs) = prop {
                all_ok = false;
                for err in errs {
                    errors.push(ErrorWithSource::property(err, property_index))
                }
            }
        }
        if all_ok {
            Ok(self.properties)
        } else {
            Err(errors)
        }
    }

    /// Given the index of an input file, returns the queries of this file if it was successfully
    /// parsed and the errors if it was not successfully parsed.
    ///
    /// If only some of the queries were parsed correctly and others produced an error, this
    /// function returns `Err(...)`, not the queries. This only happens when the error is in
    /// processing – if the input file is malformed, it will never produce any queries.
    pub fn result_for_input_file(
        &self,
        index: usize,
    ) -> Result<&[probabilistic_properties::NamedQuery<I, F, E>], &[Error<'a>]> {
        // TODO: It would be cool to return a proper NamedQueries struct instead of a slice of
        //  NamedQuery. That way, the caller would have access to the names-to-queries map. However,
        //  to avoid allocating, this would require a new NamedQueriesSubset struct that would
        //  internally hold a reference to NamedQueries and store the restriction.
        match self.errors.get(index) {
            Some(Ok(())) => {
                let range = &self.input_source_to_properties[index];
                Ok(&self.properties[range.start..range.end])
            }
            Some(Err(errs)) => Err(&errs[..]),
            None => panic!(
                "Index out of bounds when calling `result_for_input_file` (index: {index}, but there were {} input files)",
                self.errors.len()
            ),
        }
    }
}

/// Contains parsing results for a model and a single property.
///
/// The model and property are stored in separate `Result`s. Use
/// [`.all_ok()`](ModelAndPropResult::all_ok()) to transform this into a single result.
pub struct ModelAndPropResult<'a, M = Model, I = Expr, F = Expr, E = Expr> {
    /// The parsed model, or a list of errors encountered while parsing.
    pub model: Result<M, Vec<Error<'a>>>,

    /// The parsed property, or a list of errors encountered while parsing it.
    pub property: Result<probabilistic_properties::NamedQuery<I, F, E>, Vec<Error<'a>>>,
}

impl<'a, M, Q> ModelAndPropResult<'a, M, Q> {
    /// If both the model and the property are `Ok(...)`, returns the model and property.
    ///
    /// If the model or the property is `Err(...)`, returns an accumulated list of errors.
    /// Each error is enriched with an [`ErrorSource`] marking whether it was produced while parsing
    /// the model or the property. (If an error was caused by the property,
    /// `ErrorSource::Property { index: 0}` is used.)
    pub fn all_ok(self) -> Result<ModelAndProp<M, Q>, Vec<ErrorWithSource<'a>>> {
        match (self.model, self.property) {
            (Ok(model), Ok(property)) => Ok(ModelAndProp { model, property }),
            (Ok(_), Err(prop_errs)) => Err(prop_errs
                .into_iter()
                .map(|e| ErrorWithSource::property(e, 0))
                .collect()),
            (Err(model_errs), Ok(_)) => Err(model_errs
                .into_iter()
                .map(|e| ErrorWithSource::model(e))
                .collect()),
            (Err(model_errs), Err(prop_errs)) => Err(model_errs
                .into_iter()
                .map(|e| ErrorWithSource::model(e))
                .chain(
                    prop_errs
                        .into_iter()
                        .map(|e| ErrorWithSource::property(e, 0)),
                )
                .collect()),
        }
    }
}

/// Contains parsing results for a model and several properties, without
/// [processing](crate#processed-and-unprocessed-models).
pub type UnprocessedModelAndPropsResult<'a> =
    ModelAndPropsResult<'a, UnprocessedModel, UnprocessedExpr, UnprocessedExpr, UnprocessedExpr>;

/// Contains parsing results for a model and a single property, without
/// [processing](crate#processed-and-unprocessed-models).
pub type UnprocessedModelAndPropResult<'a> =
    ModelAndPropResult<'a, UnprocessedModel, UnprocessedExpr, UnprocessedExpr, UnprocessedExpr>;

/// Contains parsing results for several properties, without
/// [processing](crate#processed-and-unprocessed-models).
pub type UnprocessedPropsResult<'a> =
    PropsResult<'a, UnprocessedExpr, UnprocessedExpr, UnprocessedExpr>;

/// A successfully parsed model and several properties after
/// [processing](crate#processed-and-unprocessed-models).
pub struct ModelAndProps<M = Model, I = Expr, F = Expr, E = Expr> {
    /// The parsed model.
    pub model: M,

    /// The parsed properties.
    pub properties: probabilistic_properties::NamedQueries<I, F, E>,
}

/// A successfully parsed model and a single property after
/// [processing](crate#processed-and-unprocessed-models).
pub struct ModelAndProp<M = Model, I = Expr, F = Expr, E = Expr> {
    /// The parsed model.
    pub model: M,

    /// The parsed property.
    pub property: probabilistic_properties::NamedQuery<I, F, E>,
}

/// A successfully parsed model and several properties without
/// [processing](crate#processed-and-unprocessed-models).
pub type UnprocessedModelAndProps =
    ModelAndProps<UnprocessedModel, UnprocessedExpr, UnprocessedExpr, UnprocessedExpr>;

/// A successfully parsed model and a single property without
/// [processing](crate#processed-and-unprocessed-models).
pub type UnprocessedModelAndProp =
    ModelAndProp<UnprocessedModel, UnprocessedExpr, UnprocessedExpr, UnprocessedExpr>;

/// A parse error together with information about whether this error was produced while parsing
/// the model or one of the properties.
pub struct ErrorWithSource<'a> {
    /// Indicates whether the error was produced while parsing the model or a property.
    pub source: ErrorSource,

    /// The underlying error.
    pub error: Error<'a>,
}

impl<'a> ErrorWithSource<'a> {
    /// Adds `source: `[`ErrorSource::Model`] to the given error.
    pub fn model(error: Error<'a>) -> Self {
        Self {
            source: ErrorSource::Model,
            error,
        }
    }

    /// Adds `source: `[`ErrorSource::Property { index: property_index} `](ErrorSource::Property)
    /// to the given error.
    pub fn property(error: Error<'a>, property_index: usize) -> Self {
        Self {
            source: ErrorSource::Property {
                property_file_index: property_index,
            },
            error,
        }
    }
}

/// Indicates which part of the input a parse error originated from.
#[derive(Debug, PartialEq, Clone)]
pub enum ErrorSource {
    /// The error was produced while parsing the model.
    Model,

    /// The error was produced while parsing the property file at the given index. Note that this
    /// refers to *property files*, not properties.
    ///
    /// For [`ModelAndPropResult`], the index is always 0.
    Property {
        /// The index of the property source.
        property_file_index: usize,
    },
}
