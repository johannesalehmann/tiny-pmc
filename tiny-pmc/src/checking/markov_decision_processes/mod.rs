use crate::CheckerError;
use crate::checking::CheckerOptions;
use probabilistic_model_algorithms::state_description::StateDescription;
use probabilistic_model_algorithms::value_iteration::NonDeterminism;
use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadInitialStates, ReadPredecessors, ReadRewards, ReadStateSpace,
};
use probabilistic_models::typed_index_collections::To1;
use probabilistic_properties::{
    NonDeterminismKind, PathFormula, Query, RewardFormula, StateFormula,
};

pub fn check_mdp<
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
    state: M::StateIndex,
    options: &CheckerOptions,
) -> Result<f64, CheckerError> {
    match query {
        Query::ProbabilityValue {
            non_determinism,
            path,
        } => {
            let result = compute_path_value(model, non_determinism, &path, options)?;
            Ok(result[state])
        }
        Query::StateFormula(state_formula) => {
            let result = compute_state_value(model, &state_formula, options)?;
            let as_float = match result.is_set(state) {
                false => 0.0,
                true => 1.0,
            };
            Ok(as_float)
        }
        Query::RewardBound {
            non_determinism,
            name,
            bound,
            reward,
        } => {
            let result = compute_reward_value(model, non_determinism, name, &reward, options)?;
            let as_float = match bound.accepts(result[state]) {
                false => 0.0,
                true => 1.0,
            };
            Ok(as_float)
        }
        Query::RewardValue {
            non_determinism,
            name,
            reward,
        } => {
            let result = compute_reward_value(model, non_determinism, name, &reward, options)?;
            Ok(result[state])
        }
        Query::TimeBound { .. } => Err(CheckerError::NoSuitableAlgorithm),
        Query::TimeValue { .. } => Err(CheckerError::NoSuitableAlgorithm),
    }
}

pub fn compute_path_value<
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
    non_determinism: Option<NonDeterminismKind>,
    formula: &PathFormula<i64, f64, M::APIdx>,
    options: &CheckerOptions,
) -> Result<To1<M::StateIndex, f64>, CheckerError> {
    match formula {
        PathFormula::Until { .. } => Err(CheckerError::NoSuitableAlgorithm),
        PathFormula::Eventually { condition } => {
            let condition_values = compute_state_value(model, condition, options)?;
            let non_determinism = match non_determinism {
                None => {
                    panic!("Must specify non-determinism explicitly!")
                }
                Some(NonDeterminismKind::Maximise) => NonDeterminism::Maximise,
                Some(NonDeterminismKind::Minimise) => NonDeterminism::Minimise,
            };
            match options.sound {
                true => Ok(
                    probabilistic_model_algorithms::value_iteration::optimistic_value_iteration(
                        model,
                        &condition_values,
                        non_determinism,
                        options.value_iteration_config(),
                    ),
                ),
                false => Ok(
                    probabilistic_model_algorithms::value_iteration::value_iteration(
                        model,
                        &condition_values,
                        non_determinism,
                        options.value_iteration_config(),
                    ),
                ),
            }
        }
        PathFormula::BoundedUntil { .. } => Err(CheckerError::NoSuitableAlgorithm),
        PathFormula::BoundedEventually { .. } => Err(CheckerError::NoSuitableAlgorithm),
        PathFormula::Generally { .. } => Err(CheckerError::NoSuitableAlgorithm),
    }
}

pub fn compute_reward_value<
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
    non_determinism: Option<NonDeterminismKind>,
    name: Option<String>,
    formula: &RewardFormula<i64, f64, M::APIdx>,
    options: &CheckerOptions,
) -> Result<To1<M::StateIndex, f64>, CheckerError> {
    let Some(rewards) = model.reward_structure(name.as_deref()) else {
        return Err(CheckerError::UnknownRewardStructure { name });
    };
    match formula {
        RewardFormula::Finally { states } => {
            let goal = compute_state_value(model, states, options)?;
            let non_determinism = match non_determinism {
                None => {
                    panic!("Must specify non-determinism explicitly!")
                }
                Some(NonDeterminismKind::Maximise) => NonDeterminism::Maximise,
                Some(NonDeterminismKind::Minimise) => NonDeterminism::Minimise,
            };
            match options.sound {
                true => Ok(
                    probabilistic_model_algorithms::value_iteration::optimistic_value_iteration_rewards(
                        model,
                        &goal,
                        rewards,
                        non_determinism,
                        options.value_iteration_config()
                    ),
                ),
                false => Ok(
                    probabilistic_model_algorithms::value_iteration::value_iteration_rewards(
                        model,
                        &goal,
                        rewards,
                        non_determinism,
                        options.value_iteration_config()
                    ),
                ),
            }
        }
        RewardFormula::Instantaneous { .. } => Err(CheckerError::NoSuitableAlgorithm),
        RewardFormula::Cumulative { .. } => Err(CheckerError::NoSuitableAlgorithm),
        RewardFormula::LongRunAverage => Err(CheckerError::NoSuitableAlgorithm),
    }
}

pub fn compute_state_value<
    'a,
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = M::StateIndex>
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadInitialStates<StateIdx = M::StateIndex>
        + ReadRewards<StateIdx = M::StateIndex, ChoiceIdx = M::ChoiceIndex>,
>(
    model: &'a M,
    formula: &StateFormula<i64, f64, M::APIdx>,
    options: &CheckerOptions,
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
            let path_value = compute_path_value(model, *non_determinism, path, options)?;
            Ok(StateDescription::Flags(
                path_value.map(|e| bound.accepts(e)),
            ))
        }
        StateFormula::LongRunAverage { .. } => Err(CheckerError::NoSuitableAlgorithm),
    }
}
