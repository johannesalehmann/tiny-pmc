use prism_model::{Expression, FullSpan, Identifier, VariableReference};
use std::fmt::Formatter;

pub mod building;
pub mod checking;
pub mod parsing;

pub type PrismModel = prism_model::Model<
    VariableReference,
    FullSpan,
    Expression<VariableReference, FullSpan>,
    Identifier<FullSpan>,
>;
pub type PrismQuery = probabilistic_properties::Query<
    Expression<VariableReference, FullSpan>,
    Expression<VariableReference, FullSpan>,
    Expression<VariableReference, FullSpan>,
>;

pub enum CheckerError {
    NoSuitableAlgorithm,
}
impl std::fmt::Debug for CheckerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckerError::NoSuitableAlgorithm => write!(f, "No suitable model-checking algorithm"),
        }
    }
}
