use crate::{ParserError, ParserSpan};
use prism_model::{Expression, ExpressionNamedVars, VariableReference};

pub type Model = prism_model::Model<VariableReference, ParserSpan>;
pub type UnprocessedModel = prism_model::ModelNamedVars<ParserSpan>;

pub type Query = probabilistic_properties::Query<
    Expression<VariableReference, ParserSpan>,
    Expression<VariableReference, ParserSpan>,
    Expression<VariableReference, ParserSpan>,
>;
pub type UnprocessedQuery = probabilistic_properties::Query<
    ExpressionNamedVars<ParserSpan>,
    ExpressionNamedVars<ParserSpan>,
    ExpressionNamedVars<ParserSpan>,
>;

pub type Error<'a> = ParserError<'a, ParserSpan, String>;

pub struct ModelAndPropsResult<'a, M = Model, Q = Query> {
    pub model: Result<M, Vec<Error<'a>>>,
    pub properties: Vec<Result<Q, Vec<Error<'a>>>>,
}
impl<'a, M, Q> ModelAndPropsResult<'a, M, Q> {
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

pub struct ModelAndPropResult<'a, M = Model, Q = Query> {
    pub model: Result<M, Vec<Error<'a>>>,
    pub property: Result<Q, Vec<Error<'a>>>,
}

impl<'a, M, Q> ModelAndPropResult<'a, M, Q> {
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

pub type UnprocessedModelAndPropsResult<'a> =
    ModelAndPropsResult<'a, UnprocessedModel, UnprocessedQuery>;
pub type UnprocessedModelAndPropResult<'a> =
    ModelAndPropResult<'a, UnprocessedModel, UnprocessedQuery>;

pub struct ModelAndProps<M = Model, Q = Query> {
    pub model: M,
    pub properties: Vec<Q>,
}

pub struct ModelAndProp<M = Model, Q = Query> {
    pub model: M,
    pub property: Q,
}

pub type UnprocessedModelAndProps = ModelAndProps<UnprocessedModel, UnprocessedQuery>;
pub type UnprocessedModelAndProp = ModelAndProp<UnprocessedModel, UnprocessedQuery>;

pub struct ErrorWithSource<'a> {
    pub source: ErrorSource,
    pub error: Error<'a>,
}

impl<'a> ErrorWithSource<'a> {
    pub fn model(error: Error<'a>) -> Self {
        Self {
            source: ErrorSource::Model,
            error,
        }
    }
    pub fn property(error: Error<'a>, property_index: usize) -> Self {
        Self {
            source: ErrorSource::Property {
                index: property_index,
            },
            error,
        }
    }
}

pub enum ErrorSource {
    Model,
    Property { index: usize },
}
