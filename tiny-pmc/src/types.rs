use prism_model::{Expression, FullSpan, Identifier, VariableReference};
use probabilistic_models::annotations::{AtomicPropositions, RewardAnnotations};
use probabilistic_models::base_model::Mdp;
use probabilistic_models::initial_states::{InitialStates, InitialStatesEnum};
use probabilistic_models::labels::Labels;
use probabilistic_models::predecessors::Predecessors;
use probabilistic_models::valuations::Valuations;
use probabilistic_models::{
    ActionIndex, AnnotationEntryIndex, AnnotationIndex, AtomicPropositionIndex, BranchIndex,
    ChoiceIndex, Model, PlayerIndex, PredecessorIndex, StateIndex, ValuationClassEntryIndex,
    ValuationClassIndex, ValuationIndex,
};
use std::convert::Infallible;

pub type PrismModel = prism_model::Model<
    VariableReference,
    FullSpan,
    Expression<VariableReference, FullSpan>,
    Identifier<FullSpan>,
>;
pub type PrismQuery = probabilistic_properties::NamedQuery<
    Expression<VariableReference, FullSpan>,
    Expression<VariableReference, FullSpan>,
    Expression<VariableReference, FullSpan>,
>;
pub type PrismQueries = probabilistic_properties::NamedQueries<
    Expression<VariableReference, FullSpan>,
    Expression<VariableReference, FullSpan>,
    Expression<VariableReference, FullSpan>,
>;
pub type UnprocessedPrismQueries = probabilistic_properties::NamedQueries<
    Expression<Identifier<FullSpan>, FullSpan>,
    Expression<Identifier<FullSpan>, FullSpan>,
    Expression<Identifier<FullSpan>, FullSpan>,
>;

pub type SI = StateIndex<usize>;
pub type CI = ChoiceIndex<usize>;
pub type BI = BranchIndex<usize>;
pub type AI = ActionIndex<usize>;
pub type PI = PlayerIndex<usize>;
pub type AEI = AnnotationEntryIndex<usize>;
pub type API = AtomicPropositionIndex<usize>;
pub type RI = AnnotationIndex<usize>;
pub type VCI = ValuationClassIndex<usize>;
pub type VCEI = ValuationClassEntryIndex<usize>;
pub type VI = ValuationIndex<usize>;
pub type PredI = PredecessorIndex<usize>;
pub type ExplicitModel = Model<
    Mdp<SI, CI, BI>,
    Option<InitialStates<SI>>,
    (),
    (),
    (),
    Option<AtomicPropositions<API, SI, AEI>>,
    Option<RewardAnnotations<RI, SI, CI, AEI>>,
    (),
    (),
    (),
>;
pub type ExplicitQueries = probabilistic_properties::NamedQueries<i64, f64, API>;

// The model that can be given to the model checker
pub type OptionalModel = Model<
    Mdp<SI, CI, BI>,
    Option<InitialStates<SI>>,
    Option<Labels<CI, AI, String>>,
    Option<Labels<BI, AI, String>>,
    Option<Infallible>, // Infallible because Observations currently cannot be constructed
    Option<AtomicPropositions<API, SI, AEI>>,
    Option<RewardAnnotations<RI, SI, CI, AEI>>,
    Option<Infallible>, // Infallible because arbitrary Annotations currently cannot be constructed
    Option<Valuations<SI, VCI, VCEI, VI>>,
    Option<Predecessors<SI, CI, BI, PredI>>,
>;
