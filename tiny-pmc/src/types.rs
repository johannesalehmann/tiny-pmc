use prism_model::{Expression, FullSpan, Identifier, VariableReference};
use probabilistic_models::annotations::{AtomicPropositions, RewardAnnotations};
use probabilistic_models::base_model::Mdp;
use probabilistic_models::initial_states::InitialStates;
use probabilistic_models::labels::{BranchLabels, ChoiceLabels};
use probabilistic_models::predecessors::Predecessors;
use probabilistic_models::valuations::Valuations;
use probabilistic_models::{AtomicPropositionIndex, DefaultIx, Model};
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

pub type API = AtomicPropositionIndex<DefaultIx>;
pub type ExplicitModel = Model<
    Mdp<DefaultIx>,
    Option<InitialStates<DefaultIx>>,
    (),
    (),
    (),
    Option<AtomicPropositions<DefaultIx>>,
    Option<RewardAnnotations<DefaultIx>>,
    (),
    (),
    (),
>;
pub type ExplicitQueries = probabilistic_properties::NamedQueries<i64, f64, API>;

// The model that can be given to the model checker
pub type OptionalModel = Model<
    Mdp<DefaultIx>,
    Option<InitialStates<DefaultIx>>,
    Option<ChoiceLabels<DefaultIx, String>>,
    Option<BranchLabels<DefaultIx, String>>,
    Option<Infallible>, // Infallible because Observations currently cannot be constructed
    Option<AtomicPropositions<DefaultIx>>,
    Option<RewardAnnotations<DefaultIx>>,
    Option<Infallible>, // Infallible because arbitrary Annotations currently cannot be constructed
    Option<Valuations<DefaultIx>>,
    Option<Predecessors<DefaultIx>>,
>;
