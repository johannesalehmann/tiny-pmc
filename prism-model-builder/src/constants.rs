use crate::ModelBuilder;
use prism_model::Span;

pub enum UserProvidedConstValue {
    Int(i64),
    Bool(bool),
    Float(f64),
}

impl<
    'a,
    S: Span,
    Q: crate::queries::QueryCollection,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    B: crate::bases::BaseModelBuilder,
    IB: crate::initial_states_builder::InitialStatesBuilder<StateIdx = B::StateIdx>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = B::StateIdx>,
> ModelBuilder<'a, S, Q, L, IS, B, IB, APs>
{
    pub fn with_constant(
        mut self,
        name: String,
        value: UserProvidedConstValue,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, APs> {
        self.constants.insert(name, value);
        self
    }
    pub fn with_int_constant(
        mut self,
        name: String,
        value: i64,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, APs> {
        self.constants
            .insert(name, UserProvidedConstValue::Int(value));
        self
    }
    pub fn with_bool_constant(
        mut self,
        name: String,
        value: bool,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, APs> {
        self.constants
            .insert(name, UserProvidedConstValue::Bool(value));
        self
    }
    pub fn with_float_constant(
        mut self,
        name: String,
        value: f64,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, APs> {
        self.constants
            .insert(name, UserProvidedConstValue::Float(value));
        self
    }
    pub fn with_constants(
        mut self,
        constants: impl IntoIterator<Item = (String, UserProvidedConstValue)>,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, APs> {
        for (name, value) in constants {
            self.constants.insert(name, value);
        }
        self
    }
}
