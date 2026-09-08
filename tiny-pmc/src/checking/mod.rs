// mod markov_chains;
// pub use markov_chains::check_markov_chain;

// mod stochastic_games;
// pub use stochastic_games::check_stochastic_game;

// mod transition_systems;
// pub use transition_systems::check_transition_system;

mod markov_decision_processes;
pub use markov_decision_processes::check_mdp;

use probabilistic_models::traits::StateSet;
// mod nonstochastic_games;
// pub use nonstochastic_games::check_nonstochastic_game;

use crate::CheckerError;
use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadInitialStates, ReadPredecessors, ReadStateSpace,
};

pub fn check<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        > + ReadInitialStates<StateIdx = <M as ReadStateSpace>::StateIdx>,
>(
    model: &M,
    query: probabilistic_properties::Query<i64, f64, <M as ReadAtomicPropositions>::APIdx>,
) -> Result<f64, CheckerError> {
    let initial_states = model.initial_states().iter().collect::<Vec<_>>();
    assert_eq!(
        initial_states.len(),
        1,
        "The model checker does not yet support models with multiple initial states"
    );
    let initial_state = initial_states[0];
    markov_decision_processes::check_mdp(model, query, initial_state, 0.000001)
}
