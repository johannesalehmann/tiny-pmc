#![warn(missing_docs)]

//! Provides a representation of a model checker query, combining a pCTL [^1] with query operators
//! for getting values or comparing them to bounds.
//!
//! This crate can represent the (single-objective) queries for the STORM model checker, [as
//! described here](https://www.stormchecker.org/documentation/background/properties.html).
//!
//! # Example
//!
//! The query `Pmax=? [F "goal"]` is expressed as follows
//! ```
//! use probabilistic_properties::{NonDeterminismKind, PathFormula, Query, StateFormula};
//!
//! let goal = "goal"; // You can use other types, e.g. expressions or atomic proposition indices
//! let query: Query<i64, f64, &str> = Query::ProbabilityValue {
//!     non_determinism: Some(NonDeterminismKind::Maximise),
//!     path: PathFormula::Eventually {
//!         condition: Box::new(StateFormula::Expression(goal)),
//!     },
//! };
//! ```
//!
//! # Generics
//!
//! The following generic types are used:
//!
//! - `I`: Type used to describe integers
//! - `F`: Type used to describe floats
//! - `E`: Type used to describe sets of states
//!
//! Usually, either `I`, `F` and `E` are all modeled as expressions or `I` uses `i64`, `F` uses
//! `f64` and `E` stores an atomic proposition that describes the states.
//!
//! [^1]: probabilistic Computation Tree Logic
//!

/// Represents a pCTL query that can be answered by a model checker.
///
/// A query can either ask for a value (a probability, a reward value or a time value) or compare
/// a value to a bound.
#[derive(Clone)]
pub enum Query<I, F, E> {
    /// A query for a probability value, expressed as `P=? [path]`, `Pmin=? [path]` or
    /// `Pmax=? [path]`.
    ///
    /// There is no `ProbabilityBound` variant, as this can be modeled with [`Query::StateFormula`].
    ProbabilityValue {
        /// How non-determinism should be resolved. If `None`, this corresponds to `P=? [path]`,
        /// e.g. because the model does not exhibit non-determinism.
        non_determinism: Option<NonDeterminismKind>,
        /// The path formula for which the probability is queried
        path: PathFormula<I, F, E>,
    },
    /// A query for a state formula, which evaluates to a bool (or float for long-run average). This
    /// variant is used for `P>=threshold [path]`.
    StateFormula(StateFormula<I, F, E>),

    /// A query for whether a rewards formula adheres to a bound, e.g. `Rmin <= t [F "goal"]`,
    /// `R{"name"}max >= t [C<=k]` or `R < t [LRA]`.
    RewardBound {
        /// How non-determinism should be resolved. If `None`, no `min`/`max` is specified, e.g.
        /// because the model does not exhibit non-determinism.
        non_determinism: Option<NonDeterminismKind>,
        /// The name of the rewards structure
        name: Option<String>,
        /// The bounds against which the rewards value is checked, i.e. a comparison operator and a
        /// float value.
        bound: Bound<F>,
        /// The rewards formula for which the value is queried
        reward: RewardFormula<I, F, E>,
    },
    /// A query for the value of a rewards formula, e.g. `R=? [I=k]` or `R{"name"}min=? [F "goal"]`.
    RewardValue {
        /// How non-determinism should be resolved. If `None`, no `min`/`max` is specified, e.g.
        /// because the model does not exhibit non-determinism.
        non_determinism: Option<NonDeterminismKind>,
        /// The name of the rewards structure
        name: Option<String>,
        /// The rewards formula for which the value is queried
        reward: RewardFormula<I, F, E>,
    },
    /// A query for whether the (implicit) "time" rewards structure adheres to a bound.
    ///
    /// The time rewards structure has a constant reward of `1` per step.
    TimeBound {
        /// How non-determinism should be resolved. If `None`, no `min`/`max` is specified, e.g.
        /// because the model does not exhibit non-determinism.
        non_determinism: Option<NonDeterminismKind>,
        /// The bounds against which the time value is checked, i.e. a comparison operator and a
        /// float value.
        bound: Bound<F>,
        /// The rewards formula for which the value is queried
        reward: RewardFormula<I, F, E>,
    },
    /// A query for the value of the (implicit) "time" rewards structure.
    ///
    /// The time rewards structure has a constant reward of `1` per step.
    TimeValue {
        /// How non-determinism should be resolved. If `None`, no `min`/`max` is specified, e.g.
        /// because the model does not exhibit non-determinism.
        non_determinism: Option<NonDeterminismKind>,
        /// The rewards formula for which the value is queried
        reward: RewardFormula<I, F, E>,
    },
}

impl<I, F, E> Query<I, F, E> {
    /// Returns a query where the integers, floats and expressions are stored as mutable references.
    pub fn as_mut(&mut self) -> Query<&mut I, &mut F, &mut E> {
        match self {
            Query::ProbabilityValue {
                non_determinism,
                path,
            } => Query::ProbabilityValue {
                non_determinism: *non_determinism,
                path: path.as_mut(),
            },
            Query::StateFormula(formula) => Query::StateFormula(formula.as_mut()),
            Query::RewardBound {
                non_determinism,
                name,
                bound,
                reward,
            } => {
                Query::RewardBound {
                    non_determinism: *non_determinism,
                    name: name.clone(), // TODO: Avoid cloning name here?
                    bound: bound.as_mut(),
                    reward: reward.as_mut(),
                }
            }
            Query::RewardValue {
                non_determinism,
                name,
                reward,
            } => Query::RewardValue {
                non_determinism: *non_determinism,
                name: name.clone(),
                reward: reward.as_mut(),
            },
            Query::TimeBound {
                non_determinism,
                bound,
                reward,
            } => Query::TimeBound {
                non_determinism: *non_determinism,
                bound: bound.as_mut(),
                reward: reward.as_mut(),
            },
            Query::TimeValue {
                non_determinism,
                reward,
            } => Query::TimeValue {
                non_determinism: *non_determinism,
                reward: reward.as_mut(),
            },
        }
    }

    /// Maps the integer values of the query according to the given mapping function.
    pub fn map_i<I2, M: FnMut(I) -> I2>(self, map: &mut M) -> Query<I2, F, E> {
        self.try_map_i(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the integer values of the query according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_i()` also returns `Err(...)`.
    pub fn try_map_i<Er, I2, M: FnMut(I) -> Result<I2, Er>>(
        self,
        map: &mut M,
    ) -> Result<Query<I2, F, E>, Er> {
        Ok(match self {
            Query::ProbabilityValue {
                non_determinism,
                path,
            } => Query::ProbabilityValue {
                non_determinism,
                path: path.try_map_i(map)?,
            },
            Query::StateFormula(formula) => Query::StateFormula(formula.try_map_i(map)?),
            Query::RewardBound {
                non_determinism,
                name,
                bound,
                reward,
            } => Query::RewardBound {
                non_determinism,
                name,
                bound,
                reward: reward.try_map_i(map)?,
            },
            Query::RewardValue {
                non_determinism,
                name,
                reward,
            } => Query::RewardValue {
                non_determinism,
                name,
                reward: reward.try_map_i(map)?,
            },
            Query::TimeBound {
                non_determinism,
                bound,
                reward,
            } => Query::TimeBound {
                non_determinism,
                bound,
                reward: reward.try_map_i(map)?,
            },
            Query::TimeValue {
                non_determinism,
                reward,
            } => Query::TimeValue {
                non_determinism,
                reward: reward.try_map_i(map)?,
            },
        })
    }

    /// Maps the float values of the query according to the given mapping function.
    pub fn map_f<F2, M: FnMut(F) -> F2>(self, map: &mut M) -> Query<I, F2, E> {
        self.try_map_f(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the float values of the query according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_f()` also returns `Err(...)`.
    pub fn try_map_f<Er, F2, M: FnMut(F) -> Result<F2, Er>>(
        self,
        map: &mut M,
    ) -> Result<Query<I, F2, E>, Er> {
        Ok(match self {
            Query::ProbabilityValue {
                non_determinism,
                path,
            } => Query::ProbabilityValue {
                non_determinism,
                path: path.try_map_f(map)?,
            },
            Query::StateFormula(formula) => Query::StateFormula(formula.try_map_f(map)?),
            Query::RewardBound {
                non_determinism,
                name,
                bound,
                reward,
            } => Query::RewardBound {
                non_determinism,
                name,
                bound: bound.try_map_value(map)?,
                reward: reward.try_map_f(map)?,
            },
            Query::RewardValue {
                non_determinism,
                name,
                reward,
            } => Query::RewardValue {
                non_determinism,
                name,
                reward: reward.try_map_f(map)?,
            },
            Query::TimeBound {
                non_determinism,
                bound,
                reward,
            } => Query::TimeBound {
                non_determinism,
                bound: bound.try_map_value(map)?,
                reward: reward.try_map_f(map)?,
            },
            Query::TimeValue {
                non_determinism,
                reward,
            } => Query::TimeValue {
                non_determinism,
                reward: reward.try_map_f(map)?,
            },
        })
    }

    /// Maps the expression values of the query according to the given mapping function.
    pub fn map_e<E2, M: FnMut(E) -> E2>(self, map: &mut M) -> Query<I, F, E2> {
        self.try_map_e(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the expression values of the query according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_e()` also returns `Err(...)`.
    pub fn try_map_e<Er, E2, M: FnMut(E) -> Result<E2, Er>>(
        self,
        map: &mut M,
    ) -> Result<Query<I, F, E2>, Er> {
        Ok(match self {
            Query::ProbabilityValue {
                non_determinism,
                path,
            } => Query::ProbabilityValue {
                non_determinism,
                path: path.try_map_e(map)?,
            },
            Query::StateFormula(formula) => Query::StateFormula(formula.try_map_e(map)?),
            Query::RewardBound {
                non_determinism,
                name,
                bound,
                reward,
            } => Query::RewardBound {
                non_determinism,
                name,
                bound,
                reward: reward.try_map_e(map)?,
            },
            Query::RewardValue {
                non_determinism,
                name,
                reward,
            } => Query::RewardValue {
                non_determinism,
                name,
                reward: reward.try_map_e(map)?,
            },
            Query::TimeBound {
                non_determinism,
                bound,
                reward,
            } => Query::TimeBound {
                non_determinism,
                bound,
                reward: reward.try_map_e(map)?,
            },
            Query::TimeValue {
                non_determinism,
                reward,
            } => Query::TimeValue {
                non_determinism,
                reward: reward.try_map_e(map)?,
            },
        })
    }
}

/// A state formula, describing a set of states.
#[derive(Clone)]
pub enum StateFormula<I, F, E> {
    /// A boolean expression over the models. A state is included if the expression evaluates to
    /// `true` in it.
    ///
    /// `E` can either store the expression explicitly or store some reference to it (for example,
    /// if the model has an atomic proposition corresponding to the expression, then `E` can be
    /// the index of the atomic proposition).
    Expression(E),

    /// A probability bound, e.g. of form `Pmax >= p [ path ]`, `P < p [path]` or `Pmin > p [path]`.
    /// A state is included if the probability of fulfilling `path` adheres to `bound`.
    ProbabilityBound {
        /// How non-determinism should be resolved. If `None`, no `min`/`max` is specified, e.g.
        /// because the model does not exhibit non-determinism.
        non_determinism: Option<NonDeterminismKind>,
        /// The bounds against which the probability is checked, i.e. a comparison operator and a
        /// float value.
        bound: Bound<F>,
        /// The path formula for which the probability is queried
        path: Box<PathFormula<I, F, E>>,
    },

    /// Describes the set of states from which the long-run average probability of being in a state
    /// that fulfils `condition` adheres to `bound`.
    LongRunAverage {
        /// How non-determinism should be resolved. If `None`, no `min`/`max` is specified, e.g.
        /// because the model does not exhibit non-determinism.
        non_determinism: Option<NonDeterminismKind>,
        /// The bounds against which the long-run average probability is checked, i.e. a comparison
        /// operator and a float value.
        bound: Bound<F>,
        /// The set of states for which the long-run average is computed.
        states: Box<StateFormula<I, F, E>>,
    },
}

impl<I, F, E> StateFormula<I, F, E> {
    /// Returns a state formula where the integers, floats and expressions are stored as mutable
    /// references.
    pub fn as_mut(&mut self) -> StateFormula<&mut I, &mut F, &mut E> {
        match self {
            StateFormula::Expression(e) => StateFormula::Expression(e),
            StateFormula::ProbabilityBound {
                non_determinism,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism: *non_determinism,
                bound: bound.as_mut(),
                path: Box::new(PathFormula::as_mut(path)),
            },
            StateFormula::LongRunAverage {
                non_determinism,
                bound,
                states,
            } => StateFormula::LongRunAverage {
                non_determinism: non_determinism.clone(),
                bound: bound.as_mut(),
                states: Box::new(StateFormula::as_mut(states)),
            },
        }
    }

    /// Maps the integer values of the state formula according to the given mapping function.
    pub fn map_i<I2, M: FnMut(I) -> I2>(self, map: &mut M) -> StateFormula<I2, F, E> {
        self.try_map_i(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the integer values of the state formula according to the given fallible mapping
    /// function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_i()` also returns `Err(...)`.
    pub fn try_map_i<Er, I2, M: FnMut(I) -> Result<I2, Er>>(
        self,
        map: &mut M,
    ) -> Result<StateFormula<I2, F, E>, Er> {
        Ok(match self {
            StateFormula::Expression(expression) => StateFormula::Expression(expression),
            StateFormula::ProbabilityBound {
                non_determinism,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism,
                bound,
                path: Box::new(path.try_map_i(map)?),
            },
            StateFormula::LongRunAverage {
                non_determinism,
                bound,
                states,
            } => StateFormula::LongRunAverage {
                non_determinism: non_determinism.clone(),
                bound,
                states: Box::new(states.try_map_i(map)?),
            },
        })
    }

    /// Maps the float values of the state formula according to the given mapping function.
    pub fn map_f<F2, M: FnMut(F) -> F2>(self, map: &mut M) -> StateFormula<I, F2, E> {
        self.try_map_f(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the float values of the state formula according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_f()` also returns `Err(...)`.
    pub fn try_map_f<Er, F2, M: FnMut(F) -> Result<F2, Er>>(
        self,
        map: &mut M,
    ) -> Result<StateFormula<I, F2, E>, Er> {
        Ok(match self {
            StateFormula::Expression(expression) => StateFormula::Expression(expression),
            StateFormula::ProbabilityBound {
                non_determinism,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism,
                bound: bound.try_map_value(map)?,
                path: Box::new(path.try_map_f(map)?),
            },
            StateFormula::LongRunAverage {
                non_determinism,
                bound,
                states,
            } => StateFormula::LongRunAverage {
                non_determinism: non_determinism.clone(),
                bound: bound.try_map_value(map)?,
                states: Box::new(states.try_map_f(map)?),
            },
        })
    }

    /// Maps the expression values of the state formula according to the given mapping function.
    pub fn map_e<E2, M: FnMut(E) -> E2>(self, map: &mut M) -> StateFormula<I, F, E2> {
        self.try_map_e(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the expression values of the state formula according to the given fallible mapping
    /// function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_e()` also returns `Err(...)`.
    pub fn try_map_e<Er, E2, M: FnMut(E) -> Result<E2, Er>>(
        self,
        map: &mut M,
    ) -> Result<StateFormula<I, F, E2>, Er> {
        Ok(match self {
            StateFormula::Expression(expression) => StateFormula::Expression(map(expression)?),
            StateFormula::ProbabilityBound {
                non_determinism,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism,
                bound,
                path: Box::new(path.try_map_e(map)?),
            },
            StateFormula::LongRunAverage {
                non_determinism,
                bound,
                states,
            } => StateFormula::LongRunAverage {
                non_determinism: non_determinism.clone(),
                bound,
                states: Box::new(states.try_map_e(map)?),
            },
        })
    }
}

/// A path formula describes a set of paths.
#[derive(Clone)]
pub enum PathFormula<I, F, E> {
    /// Until formula, corresponding to syntax `before U after`.
    ///
    /// A path is included if there is some `k >= 0` such that the first `k` states satisfy
    /// `before` and state `k+1` satisfies `after`.
    Until {
        /// The "before" condition (i.e. the first part) of the until formula
        before: Box<StateFormula<I, F, E>>,
        /// The "after" condition (i.e. the second part) of the until formula
        after: Box<StateFormula<I, F, E>>,
    },

    /// Eventually formula, corresponding to syntax `F condition`.
    ///
    /// A path is included if it contains a state that satisfies `condition`
    Eventually {
        /// The condition that must be met at least once along the path
        condition: Box<StateFormula<I, F, E>>,
    },

    /// A bounded until formula, corresponding e.g. to syntax `before U<=k after` or
    /// `before U>k after`.
    ///
    /// A path is included if there is some `k` that adheres to `bound` such that the first `k`
    /// states satisfy `before` and state `k+1` satisfies `after`.
    BoundedUntil {
        /// The "before" condition (i.e. the first part) of the until formula
        before: Box<StateFormula<I, F, E>>,
        /// The "after" condition (i.e. the second part) of the until formula
        after: Box<StateFormula<I, F, E>>,
        /// The bound that determines after how many steps `after` must be visited.
        bound: Bound<I>,
    },

    /// Bounded eventually formula, corresponding e.g. to syntax `F<k condition` or
    /// `F>=k condition`.
    ///
    /// A path is included if there is a `k` that adheres to `bound` such that the `k`th state of
    /// the path adheres to condition.
    BoundedEventually {
        /// The condition that must be met at least once along the path
        condition: Box<StateFormula<I, F, E>>,
        /// The bound that determines after how many steps `condition` must be satisfied.
        bound: Bound<I>,
    },

    /// Generally formula, corresponding to syntax `G condition`.
    ///
    /// A path is included if every state of the path satisfies `condition`.
    Generally {
        /// The condition that must be satisfied by every state along the path
        condition: Box<StateFormula<I, F, E>>,
    },
}

impl<I, F, E> PathFormula<I, F, E> {
    /// Returns the condition of the "eventually" formula if the path formula is an "eventually"
    /// formula, otherwise `None`.
    pub fn eventually_condition(&self) -> Option<&StateFormula<I, F, E>> {
        match self {
            PathFormula::Eventually { condition } => Some(condition),
            _ => None,
        }
    }

    /// Returns the condition of the "generally" formula if the path formula is a "generally"
    /// formula, otherwise `None`.
    pub fn generally_condition(&self) -> Option<&StateFormula<I, F, E>> {
        match self {
            PathFormula::Generally { condition } => Some(condition),
            _ => None,
        }
    }

    /// Returns a path formula where the integers, floats and expressions are stored as mutable
    /// references.
    pub fn as_mut(&mut self) -> PathFormula<&mut I, &mut F, &mut E> {
        match self {
            PathFormula::Until { before, after } => PathFormula::Until {
                before: Box::new(StateFormula::as_mut(before)),
                after: Box::new(StateFormula::as_mut(after)),
            },
            PathFormula::Eventually { condition } => PathFormula::Eventually {
                condition: Box::new(StateFormula::as_mut(condition)),
            },
            PathFormula::BoundedUntil {
                before,
                after,
                bound,
            } => PathFormula::BoundedUntil {
                before: Box::new(StateFormula::as_mut(before)),
                after: Box::new(StateFormula::as_mut(after)),
                bound: bound.as_mut(),
            },
            PathFormula::BoundedEventually { condition, bound } => PathFormula::BoundedEventually {
                condition: Box::new(StateFormula::as_mut(condition)),
                bound: bound.as_mut(),
            },
            PathFormula::Generally { condition } => PathFormula::Generally {
                condition: Box::new(StateFormula::as_mut(condition)),
            },
        }
    }

    /// Maps the integer values of the path formula according to the given mapping function.
    pub fn map_i<I2, M: FnMut(I) -> I2>(self, map: &mut M) -> PathFormula<I2, F, E> {
        self.try_map_i(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the integer values of the path formula according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_i()` also returns `Err(...)`.
    pub fn try_map_i<Er, I2, M: FnMut(I) -> Result<I2, Er>>(
        self,
        map: &mut M,
    ) -> Result<PathFormula<I2, F, E>, Er> {
        Ok(match self {
            PathFormula::Until { before, after } => PathFormula::Until {
                before: Box::new(before.try_map_i(map)?),
                after: Box::new(after.try_map_i(map)?),
            },
            PathFormula::Eventually { condition } => PathFormula::Eventually {
                condition: Box::new(condition.try_map_i(map)?),
            },
            PathFormula::BoundedUntil {
                before,
                after,
                bound,
            } => PathFormula::BoundedUntil {
                before: Box::new(before.try_map_i(map)?),
                after: Box::new(after.try_map_i(map)?),
                bound: bound.try_map_value(map)?,
            },
            PathFormula::BoundedEventually { condition, bound } => PathFormula::BoundedEventually {
                condition: Box::new(condition.try_map_i(map)?),
                bound: bound.try_map_value(map)?,
            },
            PathFormula::Generally { condition } => PathFormula::Generally {
                condition: Box::new(condition.try_map_i(map)?),
            },
        })
    }

    /// Maps the float values of the path formula according to the given mapping function.
    pub fn map_f<F2, M: FnMut(F) -> F2>(self, map: &mut M) -> PathFormula<I, F2, E> {
        self.try_map_f(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the float values of the path formula according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_f()` also returns `Err(...)`.
    pub fn try_map_f<Er, F2, M: FnMut(F) -> Result<F2, Er>>(
        self,
        map: &mut M,
    ) -> Result<PathFormula<I, F2, E>, Er> {
        Ok(match self {
            PathFormula::Until { before, after } => PathFormula::Until {
                before: Box::new(before.try_map_f(map)?),
                after: Box::new(after.try_map_f(map)?),
            },
            PathFormula::Eventually { condition } => PathFormula::Eventually {
                condition: Box::new(condition.try_map_f(map)?),
            },
            PathFormula::BoundedUntil {
                before,
                after,
                bound,
            } => PathFormula::BoundedUntil {
                before: Box::new(before.try_map_f(map)?),
                after: Box::new(after.try_map_f(map)?),
                bound,
            },
            PathFormula::BoundedEventually { condition, bound } => PathFormula::BoundedEventually {
                condition: Box::new(condition.try_map_f(map)?),
                bound,
            },
            PathFormula::Generally { condition } => PathFormula::Generally {
                condition: Box::new(condition.try_map_f(map)?),
            },
        })
    }

    /// Maps the expression values of the path formula according to the given mapping function.
    pub fn map_e<E2, M: FnMut(E) -> E2>(self, map: &mut M) -> PathFormula<I, F, E2> {
        self.try_map_e(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the expression values of the path formula according to the given fallible mapping
    /// function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_e()` also returns `Err(...)`.
    pub fn try_map_e<Er, E2, M: FnMut(E) -> Result<E2, Er>>(
        self,
        map: &mut M,
    ) -> Result<PathFormula<I, F, E2>, Er> {
        Ok(match self {
            PathFormula::Until { before, after } => PathFormula::Until {
                before: Box::new(before.try_map_e(map)?),
                after: Box::new(after.try_map_e(map)?),
            },
            PathFormula::Eventually { condition } => PathFormula::Eventually {
                condition: Box::new(condition.try_map_e(map)?),
            },
            PathFormula::BoundedUntil {
                before,
                after,
                bound,
            } => PathFormula::BoundedUntil {
                before: Box::new(before.try_map_e(map)?),
                after: Box::new(after.try_map_e(map)?),
                bound,
            },
            PathFormula::BoundedEventually { condition, bound } => PathFormula::BoundedEventually {
                condition: Box::new(condition.try_map_e(map)?),
                bound,
            },
            PathFormula::Generally { condition } => PathFormula::Generally {
                condition: Box::new(condition.try_map_e(map)?),
            },
        })
    }
}

/// A rewards formula, describing a value computed over a rewards structure of a model.
#[derive(Clone)]
pub enum RewardFormula<I, F, E> {
    /// An instantaneous reward, equal to the syntax `I=k`.
    ///
    /// The value of the formula is the expected reward after exactly `k` steps.
    Instantaneous {
        /// The number of steps after which the reward is evaluated.
        k: I,
    },

    /// A cumulative reward, equal to syntax `C<=k`.
    ///
    /// The value of the formula is the expected cumulative reward after `k` steps.
    Cumulative {
        /// The number of steps over which the reward is accumulated.
        k: I,
    },

    /// The expected reward until reaching `states`, equal to syntax `F states`.
    Finally {
        /// The set of states up to which the reward is accumulated.
        states: StateFormula<I, F, E>,
    },

    /// The long-run average reward
    LongRunAverage,
}

impl<I, F, E> RewardFormula<I, F, E> {
    /// Returns a reward formula where the integers, floats and expressions are stored as mutable
    /// references.
    pub fn as_mut(&mut self) -> RewardFormula<&mut I, &mut F, &mut E> {
        match self {
            RewardFormula::Instantaneous { k } => RewardFormula::Instantaneous { k },
            RewardFormula::Cumulative { k } => RewardFormula::Cumulative { k },
            RewardFormula::Finally { states: state } => RewardFormula::Finally {
                states: state.as_mut(),
            },
            RewardFormula::LongRunAverage => RewardFormula::LongRunAverage,
        }
    }

    /// Maps the integer values of the reward formula according to the given mapping function.
    pub fn map_i<I2, M: FnMut(I) -> I2>(self, map: &mut M) -> RewardFormula<I2, F, E> {
        self.try_map_i(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the integer values of the reward formula according to the given fallible mapping
    /// function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_i()` also returns `Err(...)`.
    pub fn try_map_i<Er, I2, M: FnMut(I) -> Result<I2, Er>>(
        self,
        map: &mut M,
    ) -> Result<RewardFormula<I2, F, E>, Er> {
        Ok(match self {
            RewardFormula::Instantaneous { k } => RewardFormula::Instantaneous { k: map(k)? },
            RewardFormula::Cumulative { k } => RewardFormula::Cumulative { k: map(k)? },
            RewardFormula::Finally { states: state } => RewardFormula::Finally {
                states: state.try_map_i(map)?,
            },
            RewardFormula::LongRunAverage => RewardFormula::LongRunAverage,
        })
    }

    /// Maps the float values of the reward formula according to the given mapping function.
    pub fn map_f<F2, M: FnMut(F) -> F2>(self, map: &mut M) -> RewardFormula<I, F2, E> {
        self.try_map_f(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the float values of the reward formula according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_f()` also returns `Err(...)`.
    pub fn try_map_f<Er, F2, M: FnMut(F) -> Result<F2, Er>>(
        self,
        map: &mut M,
    ) -> Result<RewardFormula<I, F2, E>, Er> {
        Ok(match self {
            RewardFormula::Instantaneous { k } => RewardFormula::Instantaneous { k },
            RewardFormula::Cumulative { k } => RewardFormula::Cumulative { k },
            RewardFormula::Finally { states: state } => RewardFormula::Finally {
                states: state.try_map_f(map)?,
            },
            RewardFormula::LongRunAverage => RewardFormula::LongRunAverage,
        })
    }

    /// Maps the expression values of the reward formula according to the given mapping function.
    pub fn map_e<E2, M: FnMut(E) -> E2>(self, map: &mut M) -> RewardFormula<I, F, E2> {
        self.try_map_e(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the expression values of the reward formula according to the given fallible mapping
    /// function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_e()` also returns `Err(...)`.
    pub fn try_map_e<Er, E2, M: FnMut(E) -> Result<E2, Er>>(
        self,
        map: &mut M,
    ) -> Result<RewardFormula<I, F, E2>, Er> {
        Ok(match self {
            RewardFormula::Instantaneous { k } => RewardFormula::Instantaneous { k },
            RewardFormula::Cumulative { k } => RewardFormula::Cumulative { k },
            RewardFormula::Finally { states: state } => RewardFormula::Finally {
                states: state.try_map_e(map)?,
            },
            RewardFormula::LongRunAverage => RewardFormula::LongRunAverage,
        })
    }
}

/// A bound over the given numeric type `V`. Corresponds to syntax `< value`, `<= value`, `> value`
/// or `>= value`.
#[derive(Clone)]
pub struct Bound<V> {
    /// The comparison operator of the bound
    pub operator: BoundOperator,

    /// The value of the bound
    pub value: V,
}

impl<V> Bound<V> {
    /// Returns a bound where the value is stored as a mutable reference.
    pub fn as_mut(&mut self) -> Bound<&mut V> {
        Bound {
            operator: self.operator,
            value: &mut self.value,
        }
    }

    /// Maps the value of the bound according to the given mapping function.
    pub fn map_value<V2, F: FnMut(V) -> V2>(self, map: &mut F) -> Bound<V2> {
        Bound {
            operator: self.operator,
            value: map(self.value),
        }
    }

    /// Maps the value of the bound according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_value()` also returns `Err(...)`.
    pub fn try_map_value<Er, V2, F: FnMut(V) -> Result<V2, Er>>(
        self,
        map: &mut F,
    ) -> Result<Bound<V2>, Er> {
        Ok(Bound {
            operator: self.operator,
            value: map(self.value)?,
        })
    }
}

/// A comparison operator, used in [`Bound`].
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BoundOperator {
    /// Less-than comparison (`<`)
    LessThan,
    /// Less-than-or-equal comparison (`<=`)
    LessOrEqual,
    /// Greater-than comparison (`>`)
    GreaterThan,
    /// Greater-than-or-equal comparison (`>=`)
    GreaterOrEqual,
}

/// How non-determinism should be resolved
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum NonDeterminismKind {
    /// Choose actions that maximise the value
    Maximise,
    /// Choose actions that minimise the value
    Minimise,
}
