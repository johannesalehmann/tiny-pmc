// mod markov_chains;
// pub use markov_chains::check_markov_chain;

// mod stochastic_games;
// pub use stochastic_games::check_stochastic_game;

// mod transition_systems;
// pub use transition_systems::check_transition_system;

mod markov_decision_processes;

pub use markov_decision_processes::check_mdp;
use std::fmt::Formatter;

use probabilistic_models::traits::StateSet;
// mod nonstochastic_games;
// pub use nonstochastic_games::check_nonstochastic_game;

pub use probabilistic_model_algorithms::sub_model::{AttractorChoiceMode, SubModelOrder};
use probabilistic_model_algorithms::value_iteration::ValueIterationConfig;
pub use probabilistic_model_algorithms::value_iteration::{
    CollapseMecs, EpsAllocationScheme, SccTimingOutput, SolveOrder, SubModelHook,
};
use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadInitialStates, ReadPredecessors, ReadRewards, ReadStateSpace,
};

pub enum CheckerError {
    NoSuitableAlgorithm,
    UnknownRewardStructure { name: Option<String> },
}
impl std::fmt::Debug for CheckerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckerError::NoSuitableAlgorithm => write!(f, "No suitable model-checking algorithm"),
            CheckerError::UnknownRewardStructure { name } => {
                let name = match name {
                    Some(name) => format!(" named \"{name}\""),
                    None => "".to_string(),
                };
                write!(f, "The model has no reward structure{name}")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CheckerOptions {
    pub eps: f64,
    pub sound: bool,
    pub solve_order: SolveOrder,
    pub collapse_mecs: CollapseMecs,
    pub sub_model_order: SubModelOrder,
    pub write_scc_timing: Option<SccTimingOutput>,
}

impl Default for CheckerOptions {
    fn default() -> Self {
        Self {
            eps: 0.000001,
            sound: true,
            solve_order: SolveOrder::Topological {
                eps_allocation_scheme: EpsAllocationScheme::Uniform,
                hook: None,
            },
            collapse_mecs: CollapseMecs::WhenNecessary,
            sub_model_order: SubModelOrder::BackToFront,
            write_scc_timing: None,
        }
    }
}

impl CheckerOptions {
    pub fn value_iteration_config(&self) -> ValueIterationConfig {
        ValueIterationConfig {
            collapse_mecs: self.collapse_mecs,
            solve_order: self.solve_order.clone(),
            sub_model_order: self.sub_model_order,
            eps: self.eps,
            write_sub_mdp_timing: self.write_scc_timing.clone(),
        }
    }
}

pub fn check<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = M::StateIndex>
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadInitialStates<StateIdx = M::StateIndex>
        + ReadRewards<StateIdx = M::StateIndex, ChoiceIdx = M::ChoiceIndex>,
>(
    model: &M,
    query: probabilistic_properties::Query<i64, f64, <M as ReadAtomicPropositions>::APIdx>,
    options: &CheckerOptions,
) -> Result<f64, CheckerError> {
    let initial_states = model.initial_states().iter().collect::<Vec<_>>();
    assert_eq!(
        initial_states.len(),
        1,
        "The model checker does not yet support models with multiple initial states"
    );
    let initial_state = initial_states[0];
    markov_decision_processes::check_mdp(model, query, initial_state, options)
}
