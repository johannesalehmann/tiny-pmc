use crate::CheckerError;
use probabilistic_model_algorithms::state_description::StateDescription;
use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadInitialStates, ReadPredecessors, ReadStateSpace,
};
use probabilistic_models::typed_index_collections::To1;
use probabilistic_properties::{NonDeterminismKind, PathFormula, Query, StateFormula};

pub fn check_mdp<
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
    state: <M as ReadStateSpace>::StateIdx,
    eps: f64,
) -> Result<f64, CheckerError> {
    match query {
        Query::ProbabilityValue {
            non_determinism,
            path,
        } => {
            let result = compute_path_value(model, non_determinism, &path, eps)?;
            Ok(result[state])
        }
        Query::StateFormula(state_formula) => {
            let result = compute_state_value(model, &state_formula, eps)?;
            let as_float = match result.is_set(state) {
                false => 0.0,
                true => 1.0,
            };
            Ok(as_float)
        }
        Query::RewardBound { .. } => Err(CheckerError::NoSuitableAlgorithm),
        Query::RewardValue { .. } => Err(CheckerError::NoSuitableAlgorithm),
        Query::TimeBound { .. } => Err(CheckerError::NoSuitableAlgorithm),
        Query::TimeValue { .. } => Err(CheckerError::NoSuitableAlgorithm),
    }
}

pub fn compute_path_value<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        > + ReadInitialStates<StateIdx = <M as ReadStateSpace>::StateIdx>,
>(
    model: &M,
    non_determinism: Option<NonDeterminismKind>,
    formula: &PathFormula<i64, f64, M::APIdx>,
    eps: f64,
) -> Result<To1<<M as ReadStateSpace>::StateIdx, f64>, CheckerError> {
    match formula {
        PathFormula::Until { .. } => Err(CheckerError::NoSuitableAlgorithm),
        PathFormula::Eventually { condition } => {
            let condition_values = compute_state_value(model, condition, eps)?;
            match non_determinism {
                None => {
                    panic!("Must specify non-determinism explicitly!")
                }
                Some(NonDeterminismKind::Maximise) => Ok(
                    probabilistic_model_algorithms::value_iteration::optimistic_value_iteration_max(
                        model,
                        &condition_values,
                        eps,
                    ),
                ),
                Some(NonDeterminismKind::Minimise) => {
                    todo!()
                }
            }
        }
        PathFormula::BoundedUntil { .. } => Err(CheckerError::NoSuitableAlgorithm),
        PathFormula::BoundedEventually { .. } => Err(CheckerError::NoSuitableAlgorithm),
        PathFormula::Generally { .. } => Err(CheckerError::NoSuitableAlgorithm),
    }
}

pub fn compute_state_value<
    'a,
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        > + ReadInitialStates<StateIdx = <M as ReadStateSpace>::StateIdx>,
>(
    model: &'a M,
    formula: &StateFormula<i64, f64, M::APIdx>,
    eps: f64,
) -> Result<StateDescription<'a, M>, CheckerError> {
    match formula {
        StateFormula::Expression(e) => Ok(StateDescription::AtomicProposition {
            model,
            ap_index: *e,
        }),
        StateFormula::ProbabilityBound {
            non_determinism,
            bound,
            path,
        } => {
            // TODO: Handle qualitative bounds (=0, >0, <1, >=1) separately
            let path_value = compute_path_value(model, *non_determinism, path, eps)?;
            Ok(StateDescription::Flags(
                path_value.map(|e| bound.accepts(e)),
            ))
        }
        StateFormula::LongRunAverage { .. } => Err(CheckerError::NoSuitableAlgorithm),
    }
}
