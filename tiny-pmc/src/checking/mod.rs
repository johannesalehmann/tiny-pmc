// mod markov_chains;
// pub use markov_chains::check_markov_chain;

// mod stochastic_games;
// pub use stochastic_games::check_stochastic_game;

// mod transition_systems;
// pub use transition_systems::check_transition_system;

// mod markov_decision_processes;
// pub use markov_decision_processes::check_mdp;
use probabilistic_models::traits::StateSet;
// mod nonstochastic_games;
// pub use nonstochastic_games::check_nonstochastic_game;

use crate::CheckerError;
use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadInitialStates, ReadPredecessors, ReadStateSpace,
};
use probabilistic_properties::{NonDeterminismKind, PathFormula, Query, StateFormula};

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
    match query {
        Query::ProbabilityValue {
            non_determinism,
            path,
        } => {
            let non_determinism = match non_determinism {
                None => {
                    panic!("Non-determinism must be specified explicitly")
                }
                Some(non_determinism) => non_determinism,
            };
            match path {
                PathFormula::Eventually { condition } => {
                    if let StateFormula::Expression(e) = &*condition {
                        let eps = 0.000001;
                        let values = match non_determinism {
                            NonDeterminismKind::Maximise => {
                                println!("Performing optimistic value iteration");
                                probabilistic_model_algorithms::value_iteration::optimistic_value_iteration_max(
                                    model, *e, eps,
                                )
                            }
                            NonDeterminismKind::Minimise => {
                                println!("Performing unsound value iteration with eps={eps}");
                                probabilistic_model_algorithms::value_iteration::value_iteration_min(
                                    model, *e, eps,
                                )
                            }
                        };
                        Ok(values[initial_state])
                    } else {
                        Err(CheckerError::NoSuitableAlgorithm)
                    }
                }
                _ => Err(CheckerError::NoSuitableAlgorithm),
            }
        }
        Query::StateFormula(_) => Err(CheckerError::NoSuitableAlgorithm),
        Query::RewardBound { .. } => Err(CheckerError::NoSuitableAlgorithm),
        Query::RewardValue { .. } => Err(CheckerError::NoSuitableAlgorithm),
        Query::TimeBound { .. } => Err(CheckerError::NoSuitableAlgorithm),
        Query::TimeValue { .. } => Err(CheckerError::NoSuitableAlgorithm),
    }

    // let features = model.get_model_features();

    // if features.representable_as_transition_system() {
    //     let ts: TransitionSystem<
    //         M::Predecessors,
    //         M::Valuation,
    //         M::AtomicPropositions,
    //         M::InitialStates,
    //     > = model.into_iter().map_owners(|_| ()).collect();
    //     let ts: TransitionSystem<
    //         VectorPredecessors,
    //         M::Valuation,
    //         M::AtomicPropositions,
    //         M::InitialStates,
    //     > = ts.rebuild_and_transform_predecessors();
    //
    //     let result = check_transition_system(ts, &query);
    //
    //     let model = match result {
    //         Ok(result) => return Ok(result),
    //         Err((CheckerError::NoSuitableAlgorithm)) => (),
    //     };
    // };
    // if features.representable_as_markov_decision_process() {
    //     let mdp: Mdp<M::Predecessors, M::Valuation, M::AtomicPropositions, M::InitialStates> =
    //         model.into_iter().map_owners(|_| ()).collect();
    //     let mdp: Mdp<VectorPredecessors, M::Valuation, M::AtomicPropositions, M::InitialStates> =
    //         mdp.rebuild_and_transform_predecessors();
    //
    //     let result = check_mdp(mdp, query);
    //
    //     match result {
    //         Ok(result) => return Ok(result),
    //         Err(CheckerError::NoSuitableAlgorithm) => (),
    //     };
    // }
}
