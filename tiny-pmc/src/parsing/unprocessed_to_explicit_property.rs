use crate::{API, ExplicitQueries, UnprocessedPrismQueries};
use prism_model::{Expression, FullSpan, Identifier};
use probabilistic_models::traits::ReadAtomicPropositions;
use probabilistic_properties::{
    Bound, NamedQueries, NamedQuery, PathFormula, Query, RewardFormula, StateFormula,
};

#[derive(Debug, Clone, PartialEq)]
pub enum FormulaProcessingError {
    NonLiteralExpression(Expression<Identifier<FullSpan>, FullSpan>),
    NonLabelExpression(Expression<Identifier<FullSpan>, FullSpan>),
    UnknownLabel(String),
}

pub fn from_unprocessed_to_explicit_properties<M: ReadAtomicPropositions<APIdx = API>>(
    queries: UnprocessedPrismQueries,
    model: &M,
) -> Result<ExplicitQueries, FormulaProcessingError> {
    // TODO: This function is currently very limited in scope and does not use all the information
    //  that is available. It could be extended as follows:
    //  1. If a query refers to variables in a state formula, we can dynamically build a new atomic
    //     proposition entry for each state, assuming the model comes with valuations.
    //  2. If a query refers to constants in a bounds (or if the bounds are more complex than an
    //     integer or float literal), this could be evaluated, as long as the const values are
    //     available, either from the model metadata or provided explicitly.
    //  The first extension is currently blocked by parsing UMB valuations.
    //  The second extension is currently blocked by the expression parser not being available
    //  outside of the model builder. It should probably live in its own crate.

    let mut result = NamedQueries::with_capacity(queries.len());
    for named_query in queries {
        let query = handle_query(named_query.query, model)?;
        result
            .add(NamedQuery::new(named_query.name, query))
            .unwrap() // Query names are unique, as they were unique in the unprocessed queries
    }
    Ok(result)
}

fn handle_query<M: ReadAtomicPropositions>(
    query: Query<
        Expression<Identifier<FullSpan>, FullSpan>,
        Expression<Identifier<FullSpan>, FullSpan>,
        Expression<Identifier<FullSpan>, FullSpan>,
    >,
    model: &M,
) -> Result<Query<i64, f64, M::APIdx>, FormulaProcessingError> {
    match query {
        Query::ProbabilityValue {
            non_determinism,
            path,
        } => Ok(Query::ProbabilityValue {
            non_determinism,
            path: handle_path_formula(path, model)?,
        }),
        Query::StateFormula(state_formula) => Ok(Query::StateFormula(handle_state_formula(
            state_formula,
            model,
        )?)),
        Query::RewardBound {
            non_determinism,
            name,
            bound,
            reward,
        } => Ok(Query::RewardBound {
            non_determinism,
            name,
            bound: handle_float_bound(bound)?,
            reward: handle_rewards_formula(reward, model)?,
        }),
        Query::RewardValue {
            non_determinism,
            name,
            reward,
        } => Ok(Query::RewardValue {
            non_determinism,
            name,
            reward: handle_rewards_formula(reward, model)?,
        }),
        Query::TimeBound {
            non_determinism,
            bound,
            reward,
        } => Ok(Query::TimeBound {
            non_determinism,
            bound: handle_float_bound(bound)?,
            reward: handle_rewards_formula(reward, model)?,
        }),
        Query::TimeValue {
            non_determinism,
            reward,
        } => Ok(Query::TimeValue {
            non_determinism,
            reward: handle_rewards_formula(reward, model)?,
        }),
    }
}
fn handle_state_formula<M: ReadAtomicPropositions>(
    state_formula: StateFormula<
        Expression<Identifier<FullSpan>, FullSpan>,
        Expression<Identifier<FullSpan>, FullSpan>,
        Expression<Identifier<FullSpan>, FullSpan>,
    >,
    model: &M,
) -> Result<StateFormula<i64, f64, M::APIdx>, FormulaProcessingError> {
    match state_formula {
        StateFormula::Expression(e) => match e {
            Expression::Label(name, _) => match model.atomic_proposition_by_name(&name.name) {
                Some(index) => Ok(StateFormula::Expression(index)),
                None => Err(FormulaProcessingError::UnknownLabel(name.name)),
            },
            _ => Err(FormulaProcessingError::NonLabelExpression(e.clone())),
        },
        StateFormula::ProbabilityBound {
            non_determinism,
            bound,
            path,
        } => Ok(StateFormula::ProbabilityBound {
            non_determinism,
            bound: handle_float_bound(bound)?,
            path: Box::new(handle_path_formula(*path, model)?),
        }),
        StateFormula::LongRunAverage {
            non_determinism,
            bound,
            states,
        } => Ok(StateFormula::LongRunAverage {
            non_determinism,
            bound: handle_float_bound(bound)?,
            states: Box::new(handle_state_formula(*states, model)?),
        }),
    }
}

fn handle_path_formula<M: ReadAtomicPropositions>(
    path_formula: PathFormula<
        Expression<Identifier<FullSpan>, FullSpan>,
        Expression<Identifier<FullSpan>, FullSpan>,
        Expression<Identifier<FullSpan>, FullSpan>,
    >,
    model: &M,
) -> Result<PathFormula<i64, f64, M::APIdx>, FormulaProcessingError> {
    match path_formula {
        PathFormula::Until { before, after } => Ok(PathFormula::Until {
            before: Box::new(handle_state_formula(*before, model)?),
            after: Box::new(handle_state_formula(*after, model)?),
        }),
        PathFormula::Eventually { condition } => Ok(PathFormula::Eventually {
            condition: Box::new(handle_state_formula(*condition, model)?),
        }),
        PathFormula::BoundedUntil {
            before,
            after,
            bound,
        } => Ok(PathFormula::BoundedUntil {
            before: Box::new(handle_state_formula(*before, model)?),
            after: Box::new(handle_state_formula(*after, model)?),
            bound: handle_integer_bound(bound)?,
        }),
        PathFormula::BoundedEventually { condition, bound } => Ok(PathFormula::BoundedEventually {
            condition: Box::new(handle_state_formula(*condition, model)?),
            bound: handle_integer_bound(bound)?,
        }),
        PathFormula::Generally { condition } => Ok(PathFormula::Generally {
            condition: Box::new(handle_state_formula(*condition, model)?),
        }),
    }
}
fn handle_rewards_formula<M: ReadAtomicPropositions>(
    rewards_formula: RewardFormula<
        Expression<Identifier<FullSpan>, FullSpan>,
        Expression<Identifier<FullSpan>, FullSpan>,
        Expression<Identifier<FullSpan>, FullSpan>,
    >,
    model: &M,
) -> Result<RewardFormula<i64, f64, M::APIdx>, FormulaProcessingError> {
    match rewards_formula {
        RewardFormula::Instantaneous { k } => Ok(RewardFormula::Instantaneous {
            k: handle_integer(k)?,
        }),
        RewardFormula::Cumulative { k } => Ok(RewardFormula::Cumulative {
            k: handle_integer(k)?,
        }),
        RewardFormula::Finally { states } => Ok(RewardFormula::Finally {
            states: handle_state_formula(states, model)?,
        }),
        RewardFormula::LongRunAverage => Ok(RewardFormula::LongRunAverage),
    }
}

fn handle_integer(
    value: Expression<Identifier<FullSpan>, FullSpan>,
) -> Result<i64, FormulaProcessingError> {
    match value {
        Expression::Int(value, _) => Ok(value),
        _ => Err(FormulaProcessingError::NonLiteralExpression(value)),
    }
}
fn handle_integer_bound(
    bound: Bound<Expression<Identifier<FullSpan>, FullSpan>>,
) -> Result<Bound<i64>, FormulaProcessingError> {
    Ok(Bound {
        operator: bound.operator,
        value: handle_integer(bound.value)?,
    })
}
fn handle_float_bound(
    bound: Bound<Expression<Identifier<FullSpan>, FullSpan>>,
) -> Result<Bound<f64>, FormulaProcessingError> {
    match bound.value {
        Expression::Int(value, _) => Ok(Bound {
            operator: bound.operator,
            value: value as f64,
        }),
        Expression::Float(value, _) => Ok(Bound {
            operator: bound.operator,
            value,
        }),
        _ => Err(FormulaProcessingError::NonLiteralExpression(
            bound.value.clone(),
        )),
    }
}
