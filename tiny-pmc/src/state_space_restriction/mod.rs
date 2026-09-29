use prism_model::{Expression, Span, VariableReference};
use probabilistic_properties::{PathFormula, Query, RewardFormula, StateFormula};

pub fn get_state_space_restriction<S: Span>(
    query: &Query<
        Expression<VariableReference, S>,
        Expression<VariableReference, S>,
        Expression<VariableReference, S>,
    >,
) -> Option<Expression<VariableReference, S>> {
    match query {
        Query::StateFormula(StateFormula::ProbabilityBound { path, .. }) => {
            condition_of_top_level_path(path)
        }
        Query::ProbabilityValue { path, .. } => condition_of_top_level_path(path),
        Query::RewardBound { reward, .. } => condition_of_reward_formula(reward),
        Query::RewardValue { reward, .. } => condition_of_reward_formula(reward),
        Query::TimeBound { reward, .. } => condition_of_reward_formula(reward),
        Query::TimeValue { reward, .. } => condition_of_reward_formula(reward),
        _ => None,
    }
}

fn condition_of_top_level_path<S: Span>(
    path: &PathFormula<
        Expression<VariableReference, S>,
        Expression<VariableReference, S>,
        Expression<VariableReference, S>,
    >,
) -> Option<Expression<VariableReference, S>> {
    match path {
        PathFormula::Until { before, after } => Some(
            condition_of_state_formula(&before)?
                .and(condition_of_state_formula(&after)?.negate_bool()),
        ),
        PathFormula::Eventually { condition } => {
            Some(condition_of_state_formula(condition)?.negate_bool())
        }
        PathFormula::BoundedUntil { .. } => None,
        PathFormula::BoundedEventually { .. } => None,
        PathFormula::Generally { condition } => condition_of_state_formula(condition),
    }
}

fn condition_of_state_formula<S: Span>(
    formula: &StateFormula<
        Expression<VariableReference, S>,
        Expression<VariableReference, S>,
        Expression<VariableReference, S>,
    >,
) -> Option<Expression<VariableReference, S>> {
    match formula {
        StateFormula::Expression(e) => Some(e.clone()),
        StateFormula::ProbabilityBound { .. } => None,
        StateFormula::LongRunAverage { .. } => None,
    }
}

fn condition_of_reward_formula<S: Span>(
    formula: &RewardFormula<
        Expression<VariableReference, S>,
        Expression<VariableReference, S>,
        Expression<VariableReference, S>,
    >,
) -> Option<Expression<VariableReference, S>> {
    match formula {
        RewardFormula::Finally {
            states: StateFormula::Expression(e),
        } => Some(e.clone().negate_bool()),
        _ => None,
    }
}
