use crate::{ParserError, ParserSpan};
use prism_model::{Expression, ExpressionNamedVars, VariableReference};

/// The PRISM model produced by the parser after [processing](crate#processed-and-unprocessed-models).
pub type Model = prism_model::Model<VariableReference, ParserSpan>;

/// The PRISM model produced by the parser without [processing](crate#processed-and-unprocessed-models).
pub type UnprocessedModel = prism_model::ModelNamedVars<ParserSpan>;

/// A query (also known as objective or property) produced by the parser after
/// [processing](crate#processed-and-unprocessed-models).
pub type Query = probabilistic_properties::Query<
    Expression<VariableReference, ParserSpan>,
    Expression<VariableReference, ParserSpan>,
    Expression<VariableReference, ParserSpan>,
>;

/// A query (also known as objective or property) produced by the parser without
/// [processing](crate#processed-and-unprocessed-models).
pub type UnprocessedQuery = probabilistic_properties::Query<
    ExpressionNamedVars<ParserSpan>,
    ExpressionNamedVars<ParserSpan>,
    ExpressionNamedVars<ParserSpan>,
>;

/// The error type produced by the parser
pub type Error<'a> = ParserError<'a, ParserSpan, String>;

/// Contains parsing results for a model and several properties.
///
/// The model and each property are stored in a separate `Result`. Use
/// [`.all_ok()`](ModelAndPropsResult::all_ok()) to transform this into a single result.
pub struct ModelAndPropsResult<'a, M = Model, Q = Query> {
    /// The parsed model, or a list of errors encountered while parsing.
    pub model: Result<M, Vec<Error<'a>>>,

    /// The parsed properties. Each entry either stores the property or a list of errors encountered
    /// while parsing it.
    pub properties: Vec<Result<Q, Vec<Error<'a>>>>,
}
impl<'a, M, Q> ModelAndPropsResult<'a, M, Q> {
    /// If both the model and all properties are `Ok(...)`, returns the model and properties.
    ///
    /// If the model or at least one property is `Err(...)`, returns an accumulated list of errors.
    /// Each error is enriched with an [`ErrorSource`] marking whether it was produced while parsing
    /// the model or one of the properties.
    pub fn all_ok(self) -> Result<ModelAndProps<M, Q>, Vec<ErrorWithSource<'a>>> {
        let (model, mut errors) = match self.model {
            Ok(model) => (Some(model), Vec::new()),
            Err(err) => (
                None,
                err.into_iter().map(|e| ErrorWithSource::model(e)).collect(),
            ),
        };
        let mut properties = Vec::with_capacity(self.properties.len());
        let mut all_ok = true;
        for (property_index, property) in self.properties.into_iter().enumerate() {
            match property {
                Ok(prop) => properties.push(prop),
                Err(errs) => {
                    all_ok = false;
                    for err in errs {
                        errors.push(ErrorWithSource::property(err, property_index))
                    }
                }
            }
        }

        if let Some(model) = model
            && all_ok
        {
            Ok(ModelAndProps { model, properties })
        } else {
            Err(errors)
        }
    }
}

/// Contains parsing results for a model and a single property.
///
/// The model and property are stored in separate `Result`s. Use
/// [`.all_ok()`](ModelAndPropResult::all_ok()) to transform this into a single result.
pub struct ModelAndPropResult<'a, M = Model, Q = Query> {
    /// The parsed model, or a list of errors encountered while parsing.
    pub model: Result<M, Vec<Error<'a>>>,

    /// The parsed property, or a list of errors encountered while parsing it.
    pub property: Result<Q, Vec<Error<'a>>>,
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
    ModelAndPropsResult<'a, UnprocessedModel, UnprocessedQuery>;

/// Contains parsing results for a model and a single property, without
/// [processing](crate#processed-and-unprocessed-models).
pub type UnprocessedModelAndPropResult<'a> =
    ModelAndPropResult<'a, UnprocessedModel, UnprocessedQuery>;

/// A successfully parsed model and several properties after
/// [processing](crate#processed-and-unprocessed-models).
pub struct ModelAndProps<M = Model, Q = Query> {
    /// The parsed model.
    pub model: M,

    /// The parsed properties.
    pub properties: Vec<Q>,
}

/// A successfully parsed model and a single property after
/// [processing](crate#processed-and-unprocessed-models).
pub struct ModelAndProp<M = Model, Q = Query> {
    /// The parsed model.
    pub model: M,

    /// The parsed property.
    pub property: Q,
}

/// A successfully parsed model and several properties without
/// [processing](crate#processed-and-unprocessed-models).
pub type UnprocessedModelAndProps = ModelAndProps<UnprocessedModel, UnprocessedQuery>;

/// A successfully parsed model and a single property without
/// [processing](crate#processed-and-unprocessed-models).
pub type UnprocessedModelAndProp = ModelAndProp<UnprocessedModel, UnprocessedQuery>;

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
                index: property_index,
            },
            error,
        }
    }
}

/// Indicates which part of the input a parse error originated from.
pub enum ErrorSource {
    /// The error was produced while parsing the model.
    Model,

    /// The error was produced while parsing the property at the given index.
    ///
    /// For [`ModelAndPropResult`], the index is always 0.
    Property {
        /// The index of the property
        index: usize,
    },
}
