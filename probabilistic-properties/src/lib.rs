#![warn(missing_docs)]

//! Provides a representation of a model checker query, combining a pCTL [^1] or rPATL [^2] formula
//! with query operators for getting values or comparing them to bounds.
//!
//! This crate can represent the (single-objective) queries for the STORM model checker, [as
//! described here](https://www.stormchecker.org/documentation/background/properties.html), as
//! well as rPATL queries with coalitions as used by PRISM-games.
//!
//! # Example
//!
//! The query `Pmax=? [F "goal"]` is expressed as follows
//! ```
//! use probabilistic_properties::{
//!     NoCoalition, NonDeterminismKind, PathFormula, Query, QueryEnum, StateFormula,
//! };
//!
//! let goal = "goal"; // You can use other types, e.g. expressions or atomic proposition indices
//! let query: Query<i64, f64, &str, NoCoalition> = Query::ProbabilityValue {
//!     non_determinism: Some(NonDeterminismKind::Maximise),
//!     coalition: NoCoalition,
//!     path: PathFormula::Eventually {
//!         condition: Box::new(StateFormula::Expression(goal)),
//!     },
//! };
//! let query: QueryEnum<i64, f64, &str> = QueryEnum::Pctl(query);
//! ```
//!
//! # Generics
//!
//! The following generic types are used:
//!
//! - `I`: Type used to describe integers
//! - `F`: Type used to describe floats
//! - `E`: Type used to describe sets of states
//! - `C`: Type used to describe coalitions. All operators of a [`Query`] use the same coalition
//!   type, so a query either has a coalition on every operator or on none of them. pCTL queries
//!   use [`NoCoalition`], rPATL queries usually use [`Coalition`].
//!
//! Usually, either `I`, `F` and `E` are all modeled as expressions or `I` uses `i64`, `F` uses
//! `f64` and `E` stores an atomic proposition that describes the states.
//!
//! [^1]: probabilistic Computation Tree Logic
//!
//! [^2]: probabilistic Alternating-time Temporal Logic with Rewards
//!

mod coalitions;
mod named_queries;
pub use coalitions::{Coalition, CoalitionAsMut, CoalitionAsRef, NoCoalition};
pub use named_queries::{NamedQueries, NamedQuery, QueryExists};

use std::borrow::Borrow;
use std::fmt::{Display, Formatter};

/// A query that is either a pCTL query (without coalitions) or an rPATL query (with a coalition on
/// every operator).
///
/// The coalition type `C` of rPATL queries is generic, so that coalitions can be mapped to other
/// representations (e.g. resolved player indices) with [`QueryEnum::map_c`].
#[derive(Clone, PartialEq, Eq)]
pub enum QueryEnum<I, F, E, C = Coalition> {
    /// A pCTL query, i.e. a query without coalitions.
    Pctl(Query<I, F, E, NoCoalition>),
    /// An rPATL query, i.e. a query with a coalition on every operator.
    Rpatl(Query<I, F, E, C>),
}

impl<I, F, E, C> QueryEnum<I, F, E, C> {
    /// Returns a query where the integers, floats, expressions and coalitions are stored as mutable
    /// references.
    pub fn as_mut(&mut self) -> QueryEnum<&mut I, &mut F, &mut E, C::Output<'_>>
    where
        C: CoalitionAsMut,
    {
        match self {
            QueryEnum::Pctl(query) => QueryEnum::Pctl(query.as_mut()),
            QueryEnum::Rpatl(query) => QueryEnum::Rpatl(query.as_mut()),
        }
    }

    /// Returns a query where the integers, floats, expressions and coalitions are stored as
    /// references.
    pub fn as_ref(&self) -> QueryEnum<&I, &F, &E, C::Output<'_>>
    where
        C: CoalitionAsRef,
    {
        match self {
            QueryEnum::Pctl(query) => QueryEnum::Pctl(query.as_ref()),
            QueryEnum::Rpatl(query) => QueryEnum::Rpatl(query.as_ref()),
        }
    }

    /// Maps the integer values of the query according to the given mapping function.
    pub fn map_i<I2, M: FnMut(I) -> I2>(self, map: &mut M) -> QueryEnum<I2, F, E, C> {
        self.try_map_i(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the integer values of the query according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_i()` also returns `Err(...)`.
    pub fn try_map_i<Er, I2, M: FnMut(I) -> Result<I2, Er>>(
        self,
        map: &mut M,
    ) -> Result<QueryEnum<I2, F, E, C>, Er> {
        Ok(match self {
            QueryEnum::Pctl(query) => QueryEnum::Pctl(query.try_map_i(map)?),
            QueryEnum::Rpatl(query) => QueryEnum::Rpatl(query.try_map_i(map)?),
        })
    }

    /// Maps the float values of the query according to the given mapping function.
    pub fn map_f<F2, M: FnMut(F) -> F2>(self, map: &mut M) -> QueryEnum<I, F2, E, C> {
        self.try_map_f(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the float values of the query according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_f()` also returns `Err(...)`.
    pub fn try_map_f<Er, F2, M: FnMut(F) -> Result<F2, Er>>(
        self,
        map: &mut M,
    ) -> Result<QueryEnum<I, F2, E, C>, Er> {
        Ok(match self {
            QueryEnum::Pctl(query) => QueryEnum::Pctl(query.try_map_f(map)?),
            QueryEnum::Rpatl(query) => QueryEnum::Rpatl(query.try_map_f(map)?),
        })
    }

    /// Maps the expression values of the query according to the given mapping function.
    pub fn map_e<E2, M: FnMut(E) -> E2>(self, map: &mut M) -> QueryEnum<I, F, E2, C> {
        self.try_map_e(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the expression values of the query according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_e()` also returns `Err(...)`.
    pub fn try_map_e<Er, E2, M: FnMut(E) -> Result<E2, Er>>(
        self,
        map: &mut M,
    ) -> Result<QueryEnum<I, F, E2, C>, Er> {
        Ok(match self {
            QueryEnum::Pctl(query) => QueryEnum::Pctl(query.try_map_e(map)?),
            QueryEnum::Rpatl(query) => QueryEnum::Rpatl(query.try_map_e(map)?),
        })
    }

    /// Maps the coalitions of the query according to the given mapping function.
    ///
    /// pCTL queries do not contain coalitions and are therefore returned unchanged.
    pub fn map_c<C2, M: FnMut(C) -> C2>(self, map: &mut M) -> QueryEnum<I, F, E, C2> {
        self.try_map_c(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the coalitions of the query according to the given fallible mapping function.
    ///
    /// pCTL queries do not contain coalitions and are therefore returned unchanged.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_c()` also returns `Err(...)`.
    pub fn try_map_c<Er, C2, M: FnMut(C) -> Result<C2, Er>>(
        self,
        map: &mut M,
    ) -> Result<QueryEnum<I, F, E, C2>, Er> {
        Ok(match self {
            QueryEnum::Pctl(query) => QueryEnum::Pctl(query),
            QueryEnum::Rpatl(query) => QueryEnum::Rpatl(query.try_map_c(map)?),
        })
    }
}

impl<I: Display, F: Display, E: Display, C: Display> Display for QueryEnum<I, F, E, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryEnum::Pctl(query) => write!(f, "{query}"),
            QueryEnum::Rpatl(query) => write!(f, "{query}"),
        }
    }
}

/// Represents a query that can be answered by a model checker.
///
/// A query can either ask for a value (a probability, a reward value or a time value) or compare
/// a value to a bound.
///
/// # Generics
/// - `I`: The type used for integers (e.g. the number of steps)
/// - `F`: The type used for floats (e.g. probability bounds)
/// - `E`: The type used for conditions (e.g. a predicate over states)
/// - `C`: The type used for coalitions. This can be [`NoCoalition`] for queries on non-game models,
///        or [`Coalition`] for queries on games.
///
/// `I`, `F` and `E` are usually either an expression or the computed result of this expression.
#[derive(Clone, PartialEq, Eq)]
pub enum Query<I, F, E, C> {
    /// A query for a probability value, expressed as `P=? [path]`, `Pmin=? [path]` or
    /// `Pmax=? [path]`.
    ///
    /// There is no `ProbabilityBound` variant, as this can be modeled with [`Query::StateFormula`].
    ProbabilityValue {
        /// How non-determinism should be resolved. If `None`, this corresponds to `P=? [path]`,
        /// e.g. because the model does not exhibit non-determinism.
        non_determinism: Option<NonDeterminismKind>,
        /// The coalition of players that behave according to
        /// [`non_determinism`](Query::ProbabilityValue::non_determinism). For pCTL queries, this is
        /// [`NoCoalition`].
        coalition: C,
        /// The path formula for which the probability is queried
        path: PathFormula<I, F, E, C>,
    },
    /// A query for a state formula, which evaluates to a bool (or float for long-run average). This
    /// variant is used for `P>=threshold [path]`.
    StateFormula(StateFormula<I, F, E, C>),

    // TODO: Could we move RewardBound to StateFormula? That way, it could occur in nested queries.
    /// A query for whether a rewards formula adheres to a bound, e.g. `Rmin <= t [F "goal"]`,
    /// `R{"name"}max >= t [C<=k]` or `R < t [LRA]`.
    RewardBound {
        /// How non-determinism should be resolved. If `None`, no `min`/`max` is specified, e.g.
        /// because the model does not exhibit non-determinism.
        non_determinism: Option<NonDeterminismKind>,
        /// The coalition of players that behave according to
        /// [`non_determinism`](Query::RewardBound::non_determinism). For pCTL queries, this is
        /// [`NoCoalition`].
        coalition: C,
        /// The name of the rewards structure
        name: Option<String>,
        /// The bounds against which the rewards value is checked, i.e. a comparison operator and a
        /// float value.
        bound: Bound<F>,
        /// The rewards formula for which the value is queried
        reward: RewardFormula<I, F, E, C>,
    },
    /// A query for the value of a rewards formula, e.g. `R=? [I=k]` or `R{"name"}min=? [F "goal"]`.
    RewardValue {
        /// How non-determinism should be resolved. If `None`, no `min`/`max` is specified, e.g.
        /// because the model does not exhibit non-determinism.
        non_determinism: Option<NonDeterminismKind>,
        /// The coalition of players that behave according to
        /// [`non_determinism`](Query::RewardValue::non_determinism). For pCTL queries, this is
        /// [`NoCoalition`].
        coalition: C,
        /// The name of the rewards structure
        name: Option<String>,
        /// The rewards formula for which the value is queried
        reward: RewardFormula<I, F, E, C>,
    },
    /// A query for whether the (implicit) "time" rewards structure adheres to a bound.
    ///
    /// The time rewards structure has a constant reward of `1` per step.
    TimeBound {
        /// How non-determinism should be resolved. If `None`, no `min`/`max` is specified, e.g.
        /// because the model does not exhibit non-determinism.
        non_determinism: Option<NonDeterminismKind>,
        /// The coalition of players that behave according to
        /// [`non_determinism`](Query::TimeBound::non_determinism). For pCTL queries, this is
        /// [`NoCoalition`].
        coalition: C,
        /// The bounds against which the time value is checked, i.e. a comparison operator and a
        /// float value.
        bound: Bound<F>,
        /// The rewards formula for which the value is queried
        reward: RewardFormula<I, F, E, C>,
    },
    /// A query for the value of the (implicit) "time" rewards structure.
    ///
    /// The time rewards structure has a constant reward of `1` per step.
    TimeValue {
        /// How non-determinism should be resolved. If `None`, no `min`/`max` is specified, e.g.
        /// because the model does not exhibit non-determinism.
        non_determinism: Option<NonDeterminismKind>,
        /// The coalition of players that behave according to
        /// [`non_determinism`](Query::TimeValue::non_determinism). For pCTL queries, this is
        /// [`NoCoalition`].
        coalition: C,
        /// The rewards formula for which the value is queried
        reward: RewardFormula<I, F, E, C>,
    },
}

impl<I, F, E> Query<I, F, E, NoCoalition> {
    /// If the query is a boolean query (i.e. not a `...Value` query), this returns the negation
    /// of the query by inverting the bounds and updating the boolean operations in state formulas.
    /// If the query is a `...Value` query, it returns `None`.
    ///
    /// The parameter `invert_expression` is a function that inverts the given expression. This is
    /// necessary because expressions are of type `E`, which is opaque to this library. Therefore,
    /// it cannot invert the expression itself.
    #[must_use]
    pub fn invert<IE: Fn(E) -> E>(self, invert_expression: IE) -> Option<Self> {
        match self {
            Query::ProbabilityValue { .. } => None,
            Query::StateFormula(state_formula) => Some(Query::StateFormula(
                state_formula.invert(&invert_expression),
            )),
            Query::RewardBound {
                non_determinism,
                coalition,
                name,
                bound,
                reward,
            } => Some(Query::RewardBound {
                non_determinism,
                coalition,
                name,
                bound: bound.invert(),
                reward,
            }),
            Query::RewardValue { .. } => None,
            Query::TimeBound {
                non_determinism,
                coalition,
                bound,
                reward,
            } => Some(Query::TimeBound {
                non_determinism,
                coalition,
                bound: bound.invert(),
                reward,
            }),
            Query::TimeValue { .. } => None,
        }
    }
}

impl<I, F, E, C> Query<I, F, E, C> {
    /// Returns a query where the integers, floats, expressions and coalitions are stored as mutable
    /// references.
    pub fn as_mut(&mut self) -> Query<&mut I, &mut F, &mut E, C::Output<'_>>
    where
        C: CoalitionAsMut,
    {
        match self {
            Query::ProbabilityValue {
                non_determinism,
                coalition,
                path,
            } => Query::ProbabilityValue {
                non_determinism: *non_determinism,
                coalition: CoalitionAsMut::as_mut(coalition),
                path: path.as_mut(),
            },
            Query::StateFormula(formula) => Query::StateFormula(formula.as_mut()),
            Query::RewardBound {
                non_determinism,
                coalition,
                name,
                bound,
                reward,
            } => {
                Query::RewardBound {
                    non_determinism: *non_determinism,
                    coalition: CoalitionAsMut::as_mut(coalition),
                    name: name.clone(), // TODO: Avoid cloning name here?
                    bound: bound.as_mut(),
                    reward: reward.as_mut(),
                }
            }
            Query::RewardValue {
                non_determinism,
                coalition,
                name,
                reward,
            } => Query::RewardValue {
                non_determinism: *non_determinism,
                coalition: CoalitionAsMut::as_mut(coalition),
                name: name.clone(),
                reward: reward.as_mut(),
            },
            Query::TimeBound {
                non_determinism,
                coalition,
                bound,
                reward,
            } => Query::TimeBound {
                non_determinism: *non_determinism,
                coalition: CoalitionAsMut::as_mut(coalition),
                bound: bound.as_mut(),
                reward: reward.as_mut(),
            },
            Query::TimeValue {
                non_determinism,
                coalition,
                reward,
            } => Query::TimeValue {
                non_determinism: *non_determinism,
                coalition: CoalitionAsMut::as_mut(coalition),
                reward: reward.as_mut(),
            },
        }
    }

    /// Returns a query where the integers, floats, expressions and coalitions are stored as
    /// references.
    ///
    /// For [`NoCoalition`], the coalition type stays [`NoCoalition`], see [`CoalitionAsRef`].
    pub fn as_ref(&self) -> Query<&I, &F, &E, C::Output<'_>>
    where
        C: CoalitionAsRef,
    {
        match self {
            Query::ProbabilityValue {
                non_determinism,
                coalition,
                path,
            } => Query::ProbabilityValue {
                non_determinism: *non_determinism,
                coalition: CoalitionAsRef::as_ref(coalition),
                path: path.as_ref(),
            },
            Query::StateFormula(formula) => Query::StateFormula(formula.as_ref()),
            Query::RewardBound {
                non_determinism,
                coalition,
                name,
                bound,
                reward,
            } => {
                Query::RewardBound {
                    non_determinism: *non_determinism,
                    coalition: CoalitionAsRef::as_ref(coalition),
                    name: name.clone(), // TODO: Avoid cloning name here?
                    bound: bound.as_ref(),
                    reward: reward.as_ref(),
                }
            }
            Query::RewardValue {
                non_determinism,
                coalition,
                name,
                reward,
            } => Query::RewardValue {
                non_determinism: *non_determinism,
                coalition: CoalitionAsRef::as_ref(coalition),
                name: name.clone(),
                reward: reward.as_ref(),
            },
            Query::TimeBound {
                non_determinism,
                coalition,
                bound,
                reward,
            } => Query::TimeBound {
                non_determinism: *non_determinism,
                coalition: CoalitionAsRef::as_ref(coalition),
                bound: bound.as_ref(),
                reward: reward.as_ref(),
            },
            Query::TimeValue {
                non_determinism,
                coalition,
                reward,
            } => Query::TimeValue {
                non_determinism: *non_determinism,
                coalition: CoalitionAsRef::as_ref(coalition),
                reward: reward.as_ref(),
            },
        }
    }

    /// Maps the integer values of the query according to the given mapping function.
    pub fn map_i<I2, M: FnMut(I) -> I2>(self, map: &mut M) -> Query<I2, F, E, C> {
        self.try_map_i(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the integer values of the query according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_i()` also returns `Err(...)`.
    pub fn try_map_i<Er, I2, M: FnMut(I) -> Result<I2, Er>>(
        self,
        map: &mut M,
    ) -> Result<Query<I2, F, E, C>, Er> {
        Ok(match self {
            Query::ProbabilityValue {
                non_determinism,
                coalition,
                path,
            } => Query::ProbabilityValue {
                non_determinism,
                coalition,
                path: path.try_map_i(map)?,
            },
            Query::StateFormula(formula) => Query::StateFormula(formula.try_map_i(map)?),
            Query::RewardBound {
                non_determinism,
                coalition,
                name,
                bound,
                reward,
            } => Query::RewardBound {
                non_determinism,
                coalition,
                name,
                bound,
                reward: reward.try_map_i(map)?,
            },
            Query::RewardValue {
                non_determinism,
                coalition,
                name,
                reward,
            } => Query::RewardValue {
                non_determinism,
                coalition,
                name,
                reward: reward.try_map_i(map)?,
            },
            Query::TimeBound {
                non_determinism,
                coalition,
                bound,
                reward,
            } => Query::TimeBound {
                non_determinism,
                coalition,
                bound,
                reward: reward.try_map_i(map)?,
            },
            Query::TimeValue {
                non_determinism,
                coalition,
                reward,
            } => Query::TimeValue {
                non_determinism,
                coalition,
                reward: reward.try_map_i(map)?,
            },
        })
    }

    /// Maps the float values of the query according to the given mapping function.
    pub fn map_f<F2, M: FnMut(F) -> F2>(self, map: &mut M) -> Query<I, F2, E, C> {
        self.try_map_f(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the float values of the query according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_f()` also returns `Err(...)`.
    pub fn try_map_f<Er, F2, M: FnMut(F) -> Result<F2, Er>>(
        self,
        map: &mut M,
    ) -> Result<Query<I, F2, E, C>, Er> {
        Ok(match self {
            Query::ProbabilityValue {
                non_determinism,
                coalition,
                path,
            } => Query::ProbabilityValue {
                non_determinism,
                coalition,
                path: path.try_map_f(map)?,
            },
            Query::StateFormula(formula) => Query::StateFormula(formula.try_map_f(map)?),
            Query::RewardBound {
                non_determinism,
                coalition,
                name,
                bound,
                reward,
            } => Query::RewardBound {
                non_determinism,
                coalition,
                name,
                bound: bound.try_map_value(map)?,
                reward: reward.try_map_f(map)?,
            },
            Query::RewardValue {
                non_determinism,
                coalition,
                name,
                reward,
            } => Query::RewardValue {
                non_determinism,
                coalition,
                name,
                reward: reward.try_map_f(map)?,
            },
            Query::TimeBound {
                non_determinism,
                coalition,
                bound,
                reward,
            } => Query::TimeBound {
                non_determinism,
                coalition,
                bound: bound.try_map_value(map)?,
                reward: reward.try_map_f(map)?,
            },
            Query::TimeValue {
                non_determinism,
                coalition,
                reward,
            } => Query::TimeValue {
                non_determinism,
                coalition,
                reward: reward.try_map_f(map)?,
            },
        })
    }

    /// Maps the expression values of the query according to the given mapping function.
    pub fn map_e<E2, M: FnMut(E) -> E2>(self, map: &mut M) -> Query<I, F, E2, C> {
        self.try_map_e(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the expression values of the query according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_e()` also returns `Err(...)`.
    pub fn try_map_e<Er, E2, M: FnMut(E) -> Result<E2, Er>>(
        self,
        map: &mut M,
    ) -> Result<Query<I, F, E2, C>, Er> {
        Ok(match self {
            Query::ProbabilityValue {
                non_determinism,
                coalition,
                path,
            } => Query::ProbabilityValue {
                non_determinism,
                coalition,
                path: path.try_map_e(map)?,
            },
            Query::StateFormula(formula) => Query::StateFormula(formula.try_map_e(map)?),
            Query::RewardBound {
                non_determinism,
                coalition,
                name,
                bound,
                reward,
            } => Query::RewardBound {
                non_determinism,
                coalition,
                name,
                bound,
                reward: reward.try_map_e(map)?,
            },
            Query::RewardValue {
                non_determinism,
                coalition,
                name,
                reward,
            } => Query::RewardValue {
                non_determinism,
                coalition,
                name,
                reward: reward.try_map_e(map)?,
            },
            Query::TimeBound {
                non_determinism,
                coalition,
                bound,
                reward,
            } => Query::TimeBound {
                non_determinism,
                coalition,
                bound,
                reward: reward.try_map_e(map)?,
            },
            Query::TimeValue {
                non_determinism,
                coalition,
                reward,
            } => Query::TimeValue {
                non_determinism,
                coalition,
                reward: reward.try_map_e(map)?,
            },
        })
    }

    /// Maps the coalitions of the query according to the given mapping function.
    pub fn map_c<C2, M: FnMut(C) -> C2>(self, map: &mut M) -> Query<I, F, E, C2> {
        self.try_map_c(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the coalitions of the query according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_c()` also returns `Err(...)`.
    pub fn try_map_c<Er, C2, M: FnMut(C) -> Result<C2, Er>>(
        self,
        map: &mut M,
    ) -> Result<Query<I, F, E, C2>, Er> {
        Ok(match self {
            Query::ProbabilityValue {
                non_determinism,
                coalition,
                path,
            } => Query::ProbabilityValue {
                non_determinism,
                coalition: map(coalition)?,
                path: path.try_map_c(map)?,
            },
            Query::StateFormula(formula) => Query::StateFormula(formula.try_map_c(map)?),
            Query::RewardBound {
                non_determinism,
                coalition,
                name,
                bound,
                reward,
            } => Query::RewardBound {
                non_determinism,
                coalition: map(coalition)?,
                name,
                bound,
                reward: reward.try_map_c(map)?,
            },
            Query::RewardValue {
                non_determinism,
                coalition,
                name,
                reward,
            } => Query::RewardValue {
                non_determinism,
                coalition: map(coalition)?,
                name,
                reward: reward.try_map_c(map)?,
            },
            Query::TimeBound {
                non_determinism,
                coalition,
                bound,
                reward,
            } => Query::TimeBound {
                non_determinism,
                coalition: map(coalition)?,
                bound,
                reward: reward.try_map_c(map)?,
            },
            Query::TimeValue {
                non_determinism,
                coalition,
                reward,
            } => Query::TimeValue {
                non_determinism,
                coalition: map(coalition)?,
                reward: reward.try_map_c(map)?,
            },
        })
    }
}

// TODO: The Display implementation does not yet properly parenthesise nested formulas.
impl<I: Display, F: Display, E: Display, C: Display> Display for Query<I, F, E, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Query::ProbabilityValue {
                non_determinism,
                coalition,
                path,
            } => {
                write!(f, "{coalition}P{}=? [{}]", non_determinism.to_str(), path)
            }
            Query::StateFormula(state_formula) => {
                write!(f, "{}", state_formula)
            }
            Query::RewardBound {
                non_determinism,
                coalition,
                name,
                bound,
                reward,
            } => {
                write!(
                    f,
                    "{coalition}R{}{}{} [{}]",
                    name.display(),
                    non_determinism.to_str(),
                    bound,
                    reward
                )
            }
            Query::RewardValue {
                non_determinism,
                coalition,
                name,
                reward,
            } => {
                write!(
                    f,
                    "{coalition}R{}{}=? [{}]",
                    name.display(),
                    non_determinism.to_str(),
                    reward
                )
            }
            Query::TimeBound {
                non_determinism,
                coalition,
                bound,
                reward,
            } => {
                write!(
                    f,
                    "{coalition}T{}{} [{}]",
                    non_determinism.to_str(),
                    bound,
                    reward
                )
            }
            Query::TimeValue {
                non_determinism,
                coalition,
                reward,
            } => {
                write!(f, "{coalition}T{}=? [{}]", non_determinism.to_str(), reward)
            }
        }
    }
}

/// A state formula, describing a set of states.
#[derive(Clone, PartialEq, Eq)]
pub enum StateFormula<I, F, E, C> {
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
        /// The coalition of players that behave according to
        /// [`non_determinism`](StateFormula::ProbabilityBound::non_determinism). For pCTL queries,
        /// this is [`NoCoalition`].
        coalition: C,
        /// The bounds against which the probability is checked, i.e. a comparison operator and a
        /// float value.
        bound: Bound<F>,
        /// The path formula for which the probability is queried
        path: Box<PathFormula<I, F, E, C>>,
    },

    /// Describes the set of states from which the long-run average probability of being in a state
    /// that fulfils `condition` adheres to `bound`.
    LongRunAverage {
        /// How non-determinism should be resolved. If `None`, no `min`/`max` is specified, e.g.
        /// because the model does not exhibit non-determinism.
        non_determinism: Option<NonDeterminismKind>,
        /// The coalition of players that behave according to
        /// [`non_determinism`](StateFormula::LongRunAverage::non_determinism). For pCTL queries,
        /// this is [`NoCoalition`].
        coalition: C,
        /// The bounds against which the long-run average probability is checked, i.e. a comparison
        /// operator and a float value.
        bound: Bound<F>,
        /// The set of states for which the long-run average is computed.
        states: Box<StateFormula<I, F, E, C>>,
    },

    /// A negation of the given state formula.
    Not(Box<StateFormula<I, F, E, C>>),

    /// The conjunction of two state formulas.
    And(Box<StateFormula<I, F, E, C>>, Box<StateFormula<I, F, E, C>>),

    /// The disjunction of two state formulas.
    Or(Box<StateFormula<I, F, E, C>>, Box<StateFormula<I, F, E, C>>),
}

impl<I, F, E> StateFormula<I, F, E, NoCoalition> {
    /// Returns the negation of the state formula.
    ///
    /// See [`Query::invert`] for details.
    #[must_use]
    pub fn invert<IE: Fn(E) -> E>(self, invert_expression: &IE) -> Self {
        match self {
            StateFormula::Expression(e) => StateFormula::Expression(invert_expression(e)),
            StateFormula::ProbabilityBound {
                non_determinism,
                coalition,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism,
                coalition,
                bound: bound.invert(),
                path,
            },
            StateFormula::LongRunAverage {
                non_determinism,
                coalition,
                bound,
                states,
            } => StateFormula::LongRunAverage {
                non_determinism,
                coalition,
                bound: bound.invert(),
                states,
            },
            StateFormula::Not(inner) => *inner,
            StateFormula::And(lhs, rhs) => StateFormula::Or(
                Box::new(lhs.invert(invert_expression)),
                Box::new(rhs.invert(invert_expression)),
            ),
            StateFormula::Or(lhs, rhs) => StateFormula::And(
                Box::new(lhs.invert(invert_expression)),
                Box::new(rhs.invert(invert_expression)),
            ),
        }
    }
}

impl<I, F, E, C> StateFormula<I, F, E, C> {
    /// Returns a state formula where the integers, floats, expressions and coalitions are stored as
    /// mutable references.
    pub fn as_mut(&mut self) -> StateFormula<&mut I, &mut F, &mut E, C::Output<'_>>
    where
        C: CoalitionAsMut,
    {
        match self {
            StateFormula::Expression(e) => StateFormula::Expression(e),
            StateFormula::Not(inner) => StateFormula::Not(Box::new(StateFormula::as_mut(inner))),
            StateFormula::And(lhs, rhs) => StateFormula::And(
                Box::new(StateFormula::as_mut(lhs)),
                Box::new(StateFormula::as_mut(rhs)),
            ),
            StateFormula::Or(lhs, rhs) => StateFormula::Or(
                Box::new(StateFormula::as_mut(lhs)),
                Box::new(StateFormula::as_mut(rhs)),
            ),
            StateFormula::ProbabilityBound {
                non_determinism,
                coalition,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism: *non_determinism,
                coalition: CoalitionAsMut::as_mut(coalition),
                bound: bound.as_mut(),
                path: Box::new(PathFormula::as_mut(path)),
            },
            StateFormula::LongRunAverage {
                non_determinism,
                coalition,
                bound,
                states,
            } => StateFormula::LongRunAverage {
                non_determinism: non_determinism.clone(),
                coalition: CoalitionAsMut::as_mut(coalition),
                bound: bound.as_mut(),
                states: Box::new(StateFormula::as_mut(states)),
            },
        }
    }

    /// Returns a state formula where the integers, floats, expressions and coalitions are stored as
    /// references.
    pub fn as_ref(&self) -> StateFormula<&I, &F, &E, C::Output<'_>>
    where
        C: CoalitionAsRef,
    {
        match self {
            StateFormula::Expression(e) => StateFormula::Expression(e),
            StateFormula::Not(inner) => StateFormula::Not(Box::new(StateFormula::as_ref(inner))),
            StateFormula::And(lhs, rhs) => StateFormula::And(
                Box::new(StateFormula::as_ref(lhs)),
                Box::new(StateFormula::as_ref(rhs)),
            ),
            StateFormula::Or(lhs, rhs) => StateFormula::Or(
                Box::new(StateFormula::as_ref(lhs)),
                Box::new(StateFormula::as_ref(rhs)),
            ),
            StateFormula::ProbabilityBound {
                non_determinism,
                coalition,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism: *non_determinism,
                coalition: CoalitionAsRef::as_ref(coalition),
                bound: bound.as_ref(),
                path: Box::new(PathFormula::as_ref(path)),
            },
            StateFormula::LongRunAverage {
                non_determinism,
                coalition,
                bound,
                states,
            } => StateFormula::LongRunAverage {
                non_determinism: non_determinism.clone(),
                coalition: CoalitionAsRef::as_ref(coalition),
                bound: bound.as_ref(),
                states: Box::new(StateFormula::as_ref(states)),
            },
        }
    }

    /// Maps the integer values of the state formula according to the given mapping function.
    pub fn map_i<I2, M: FnMut(I) -> I2>(self, map: &mut M) -> StateFormula<I2, F, E, C> {
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
    ) -> Result<StateFormula<I2, F, E, C>, Er> {
        Ok(match self {
            StateFormula::Expression(expression) => StateFormula::Expression(expression),
            StateFormula::Not(inner) => StateFormula::Not(Box::new(inner.try_map_i(map)?)),
            StateFormula::And(lhs, rhs) => {
                StateFormula::And(Box::new(lhs.try_map_i(map)?), Box::new(rhs.try_map_i(map)?))
            }
            StateFormula::Or(lhs, rhs) => {
                StateFormula::Or(Box::new(lhs.try_map_i(map)?), Box::new(rhs.try_map_i(map)?))
            }
            StateFormula::ProbabilityBound {
                non_determinism,
                coalition,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism,
                coalition,
                bound,
                path: Box::new(path.try_map_i(map)?),
            },
            StateFormula::LongRunAverage {
                non_determinism,
                coalition,
                bound,
                states,
            } => StateFormula::LongRunAverage {
                non_determinism: non_determinism.clone(),
                coalition,
                bound,
                states: Box::new(states.try_map_i(map)?),
            },
        })
    }

    /// Maps the float values of the state formula according to the given mapping function.
    pub fn map_f<F2, M: FnMut(F) -> F2>(self, map: &mut M) -> StateFormula<I, F2, E, C> {
        self.try_map_f(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the float values of the state formula according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_f()` also returns `Err(...)`.
    pub fn try_map_f<Er, F2, M: FnMut(F) -> Result<F2, Er>>(
        self,
        map: &mut M,
    ) -> Result<StateFormula<I, F2, E, C>, Er> {
        Ok(match self {
            StateFormula::Expression(expression) => StateFormula::Expression(expression),
            StateFormula::Not(inner) => StateFormula::Not(Box::new(inner.try_map_f(map)?)),
            StateFormula::And(lhs, rhs) => {
                StateFormula::And(Box::new(lhs.try_map_f(map)?), Box::new(rhs.try_map_f(map)?))
            }
            StateFormula::Or(lhs, rhs) => {
                StateFormula::Or(Box::new(lhs.try_map_f(map)?), Box::new(rhs.try_map_f(map)?))
            }
            StateFormula::ProbabilityBound {
                non_determinism,
                coalition,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism,
                coalition,
                bound: bound.try_map_value(map)?,
                path: Box::new(path.try_map_f(map)?),
            },
            StateFormula::LongRunAverage {
                non_determinism,
                coalition,
                bound,
                states,
            } => StateFormula::LongRunAverage {
                non_determinism: non_determinism.clone(),
                coalition,
                bound: bound.try_map_value(map)?,
                states: Box::new(states.try_map_f(map)?),
            },
        })
    }

    /// Maps the expression values of the state formula according to the given mapping function.
    pub fn map_e<E2, M: FnMut(E) -> E2>(self, map: &mut M) -> StateFormula<I, F, E2, C> {
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
    ) -> Result<StateFormula<I, F, E2, C>, Er> {
        Ok(match self {
            StateFormula::Expression(expression) => StateFormula::Expression(map(expression)?),
            StateFormula::Not(inner) => StateFormula::Not(Box::new(inner.try_map_e(map)?)),
            StateFormula::And(lhs, rhs) => {
                StateFormula::And(Box::new(lhs.try_map_e(map)?), Box::new(rhs.try_map_e(map)?))
            }
            StateFormula::Or(lhs, rhs) => {
                StateFormula::Or(Box::new(lhs.try_map_e(map)?), Box::new(rhs.try_map_e(map)?))
            }
            StateFormula::ProbabilityBound {
                non_determinism,
                coalition,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism,
                coalition,
                bound,
                path: Box::new(path.try_map_e(map)?),
            },
            StateFormula::LongRunAverage {
                non_determinism,
                coalition,
                bound,
                states,
            } => StateFormula::LongRunAverage {
                non_determinism: non_determinism.clone(),
                coalition,
                bound,
                states: Box::new(states.try_map_e(map)?),
            },
        })
    }

    /// Maps the coalitions of the state formula according to the given mapping function.
    pub fn map_c<C2, M: FnMut(C) -> C2>(self, map: &mut M) -> StateFormula<I, F, E, C2> {
        self.try_map_c(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the coalitions of the state formula according to the given fallible mapping
    /// function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_c()` also returns `Err(...)`.
    pub fn try_map_c<Er, C2, M: FnMut(C) -> Result<C2, Er>>(
        self,
        map: &mut M,
    ) -> Result<StateFormula<I, F, E, C2>, Er> {
        Ok(match self {
            StateFormula::Expression(expression) => StateFormula::Expression(expression),
            StateFormula::Not(inner) => StateFormula::Not(Box::new(inner.try_map_c(map)?)),
            StateFormula::And(lhs, rhs) => {
                StateFormula::And(Box::new(lhs.try_map_c(map)?), Box::new(rhs.try_map_c(map)?))
            }
            StateFormula::Or(lhs, rhs) => {
                StateFormula::Or(Box::new(lhs.try_map_c(map)?), Box::new(rhs.try_map_c(map)?))
            }
            StateFormula::ProbabilityBound {
                non_determinism,
                coalition,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism,
                coalition: map(coalition)?,
                bound,
                path: Box::new(path.try_map_c(map)?),
            },
            StateFormula::LongRunAverage {
                non_determinism,
                coalition,
                bound,
                states,
            } => StateFormula::LongRunAverage {
                non_determinism,
                coalition: map(coalition)?,
                bound,
                states: Box::new(states.try_map_c(map)?),
            },
        })
    }
}

impl<I: Display, F: Display, E: Display, C: Display> Display for StateFormula<I, F, E, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StateFormula::Expression(e) => {
                write!(f, "{e}")
            }
            StateFormula::ProbabilityBound {
                non_determinism,
                coalition,
                bound,
                path,
            } => {
                write!(
                    f,
                    "{coalition}P{}{bound} [ {path} ]",
                    non_determinism.to_str()
                )
            }
            StateFormula::LongRunAverage {
                non_determinism,
                coalition,
                bound,
                states,
            } => {
                write!(
                    f,
                    "{coalition}LRA{}{bound} [{states}",
                    non_determinism.to_str()
                )
            }
            StateFormula::Not(inner) => write!(f, "!({inner})"),
            StateFormula::And(lhs, rhs) => write!(f, "({lhs}) & ({rhs})"),
            StateFormula::Or(lhs, rhs) => write!(f, "({lhs}) | ({rhs})"),
        }
    }
}

/// A path formula describes a set of paths.
#[derive(Clone, PartialEq, Eq)]
pub enum PathFormula<I, F, E, C> {
    /// Until formula, corresponding to syntax `before U after`.
    ///
    /// A path is included if there is some `k >= 0` such that the first `k` states satisfy
    /// `before` and state `k+1` satisfies `after`.
    Until {
        /// The "before" condition (i.e. the first part) of the until formula
        before: Box<StateFormula<I, F, E, C>>,
        /// The "after" condition (i.e. the second part) of the until formula
        after: Box<StateFormula<I, F, E, C>>,
    },

    /// Eventually formula, corresponding to syntax `F condition`.
    ///
    /// A path is included if it contains a state that satisfies `condition`
    Eventually {
        /// The condition that must be met at least once along the path
        condition: Box<StateFormula<I, F, E, C>>,
    },

    /// A bounded until formula, corresponding e.g. to syntax `before U<=k after` or
    /// `before U>k after`.
    ///
    /// A path is included if there is some `k` that adheres to `bound` such that the first `k`
    /// states satisfy `before` and state `k+1` satisfies `after`.
    BoundedUntil {
        /// The "before" condition (i.e. the first part) of the until formula
        before: Box<StateFormula<I, F, E, C>>,
        /// The "after" condition (i.e. the second part) of the until formula
        after: Box<StateFormula<I, F, E, C>>,
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
        condition: Box<StateFormula<I, F, E, C>>,
        /// The bound that determines after how many steps `condition` must be satisfied.
        bound: Bound<I>,
    },

    /// Generally formula, corresponding to syntax `G condition`.
    ///
    /// A path is included if every state of the path satisfies `condition`.
    Generally {
        /// The condition that must be satisfied by every state along the path
        condition: Box<StateFormula<I, F, E, C>>,
    },
}

impl<I, F, E, C> PathFormula<I, F, E, C> {
    /// Returns the condition of the "eventually" formula if the path formula is an "eventually"
    /// formula, otherwise `None`.
    pub fn eventually_condition(&self) -> Option<&StateFormula<I, F, E, C>> {
        match self {
            PathFormula::Eventually { condition } => Some(condition),
            _ => None,
        }
    }

    /// Returns the condition of the "generally" formula if the path formula is a "generally"
    /// formula, otherwise `None`.
    pub fn generally_condition(&self) -> Option<&StateFormula<I, F, E, C>> {
        match self {
            PathFormula::Generally { condition } => Some(condition),
            _ => None,
        }
    }

    /// Returns a path formula where the integers, floats, expressions and coalitions are stored as
    /// mutable references.
    pub fn as_mut(&mut self) -> PathFormula<&mut I, &mut F, &mut E, C::Output<'_>>
    where
        C: CoalitionAsMut,
    {
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

    /// Returns a path formula where the integers, floats, expressions and coalitions are stored as
    /// references.
    pub fn as_ref(&self) -> PathFormula<&I, &F, &E, C::Output<'_>>
    where
        C: CoalitionAsRef,
    {
        match self {
            PathFormula::Until { before, after } => PathFormula::Until {
                before: Box::new(StateFormula::as_ref(before)),
                after: Box::new(StateFormula::as_ref(after)),
            },
            PathFormula::Eventually { condition } => PathFormula::Eventually {
                condition: Box::new(StateFormula::as_ref(condition)),
            },
            PathFormula::BoundedUntil {
                before,
                after,
                bound,
            } => PathFormula::BoundedUntil {
                before: Box::new(StateFormula::as_ref(before)),
                after: Box::new(StateFormula::as_ref(after)),
                bound: bound.as_ref(),
            },
            PathFormula::BoundedEventually { condition, bound } => PathFormula::BoundedEventually {
                condition: Box::new(StateFormula::as_ref(condition)),
                bound: bound.as_ref(),
            },
            PathFormula::Generally { condition } => PathFormula::Generally {
                condition: Box::new(StateFormula::as_ref(condition)),
            },
        }
    }

    /// Maps the integer values of the path formula according to the given mapping function.
    pub fn map_i<I2, M: FnMut(I) -> I2>(self, map: &mut M) -> PathFormula<I2, F, E, C> {
        self.try_map_i(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the integer values of the path formula according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_i()` also returns `Err(...)`.
    pub fn try_map_i<Er, I2, M: FnMut(I) -> Result<I2, Er>>(
        self,
        map: &mut M,
    ) -> Result<PathFormula<I2, F, E, C>, Er> {
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
    pub fn map_f<F2, M: FnMut(F) -> F2>(self, map: &mut M) -> PathFormula<I, F2, E, C> {
        self.try_map_f(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the float values of the path formula according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_f()` also returns `Err(...)`.
    pub fn try_map_f<Er, F2, M: FnMut(F) -> Result<F2, Er>>(
        self,
        map: &mut M,
    ) -> Result<PathFormula<I, F2, E, C>, Er> {
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
    pub fn map_e<E2, M: FnMut(E) -> E2>(self, map: &mut M) -> PathFormula<I, F, E2, C> {
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
    ) -> Result<PathFormula<I, F, E2, C>, Er> {
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

    /// Maps the coalitions of the path formula according to the given mapping function.
    pub fn map_c<C2, M: FnMut(C) -> C2>(self, map: &mut M) -> PathFormula<I, F, E, C2> {
        self.try_map_c(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the coalitions of the path formula according to the given fallible mapping
    /// function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_c()` also returns `Err(...)`.
    pub fn try_map_c<Er, C2, M: FnMut(C) -> Result<C2, Er>>(
        self,
        map: &mut M,
    ) -> Result<PathFormula<I, F, E, C2>, Er> {
        Ok(match self {
            PathFormula::Until { before, after } => PathFormula::Until {
                before: Box::new(before.try_map_c(map)?),
                after: Box::new(after.try_map_c(map)?),
            },
            PathFormula::Eventually { condition } => PathFormula::Eventually {
                condition: Box::new(condition.try_map_c(map)?),
            },
            PathFormula::BoundedUntil {
                before,
                after,
                bound,
            } => PathFormula::BoundedUntil {
                before: Box::new(before.try_map_c(map)?),
                after: Box::new(after.try_map_c(map)?),
                bound,
            },
            PathFormula::BoundedEventually { condition, bound } => PathFormula::BoundedEventually {
                condition: Box::new(condition.try_map_c(map)?),
                bound,
            },
            PathFormula::Generally { condition } => PathFormula::Generally {
                condition: Box::new(condition.try_map_c(map)?),
            },
        })
    }
}

impl<I: Display, F: Display, E: Display, C: Display> Display for PathFormula<I, F, E, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PathFormula::Until { before, after } => {
                write!(f, "{before} U {after}")
            }
            PathFormula::Eventually { condition } => {
                write!(f, "F {condition}")
            }
            PathFormula::BoundedUntil {
                before,
                after,
                bound,
            } => {
                write!(f, "{before} U{bound} {after}")
            }
            PathFormula::BoundedEventually { condition, bound } => {
                write!(f, "F{bound} {condition}")
            }
            PathFormula::Generally { condition } => {
                write!(f, "G {condition}")
            }
        }
    }
}

/// A rewards formula, describing a value computed over a rewards structure of a model.
#[derive(Clone, PartialEq, Eq)]
pub enum RewardFormula<I, F, E, C> {
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
        states: StateFormula<I, F, E, C>,
    },

    /// The long-run average reward
    LongRunAverage,
}

impl<I, F, E, C> RewardFormula<I, F, E, C> {
    /// Returns a reward formula where the integers, floats, expressions and coalitions are stored as
    /// mutable references.
    pub fn as_mut(&mut self) -> RewardFormula<&mut I, &mut F, &mut E, C::Output<'_>>
    where
        C: CoalitionAsMut,
    {
        match self {
            RewardFormula::Instantaneous { k } => RewardFormula::Instantaneous { k },
            RewardFormula::Cumulative { k } => RewardFormula::Cumulative { k },
            RewardFormula::Finally { states: state } => RewardFormula::Finally {
                states: state.as_mut(),
            },
            RewardFormula::LongRunAverage => RewardFormula::LongRunAverage,
        }
    }

    /// Returns a reward formula where the integers, floats, expressions and coalitions are stored as
    /// references.
    pub fn as_ref(&self) -> RewardFormula<&I, &F, &E, C::Output<'_>>
    where
        C: CoalitionAsRef,
    {
        match self {
            RewardFormula::Instantaneous { k } => RewardFormula::Instantaneous { k },
            RewardFormula::Cumulative { k } => RewardFormula::Cumulative { k },
            RewardFormula::Finally { states: state } => RewardFormula::Finally {
                states: state.as_ref(),
            },
            RewardFormula::LongRunAverage => RewardFormula::LongRunAverage,
        }
    }

    /// Maps the integer values of the reward formula according to the given mapping function.
    pub fn map_i<I2, M: FnMut(I) -> I2>(self, map: &mut M) -> RewardFormula<I2, F, E, C> {
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
    ) -> Result<RewardFormula<I2, F, E, C>, Er> {
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
    pub fn map_f<F2, M: FnMut(F) -> F2>(self, map: &mut M) -> RewardFormula<I, F2, E, C> {
        self.try_map_f(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the float values of the reward formula according to the given fallible mapping function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_f()` also returns `Err(...)`.
    pub fn try_map_f<Er, F2, M: FnMut(F) -> Result<F2, Er>>(
        self,
        map: &mut M,
    ) -> Result<RewardFormula<I, F2, E, C>, Er> {
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
    pub fn map_e<E2, M: FnMut(E) -> E2>(self, map: &mut M) -> RewardFormula<I, F, E2, C> {
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
    ) -> Result<RewardFormula<I, F, E2, C>, Er> {
        Ok(match self {
            RewardFormula::Instantaneous { k } => RewardFormula::Instantaneous { k },
            RewardFormula::Cumulative { k } => RewardFormula::Cumulative { k },
            RewardFormula::Finally { states: state } => RewardFormula::Finally {
                states: state.try_map_e(map)?,
            },
            RewardFormula::LongRunAverage => RewardFormula::LongRunAverage,
        })
    }

    /// Maps the coalitions of the reward formula according to the given mapping function.
    pub fn map_c<C2, M: FnMut(C) -> C2>(self, map: &mut M) -> RewardFormula<I, F, E, C2> {
        self.try_map_c(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    /// Maps the coalitions of the reward formula according to the given fallible mapping
    /// function.
    ///
    /// If the mapping function returns `Err(...)`, then `try_map_c()` also returns `Err(...)`.
    pub fn try_map_c<Er, C2, M: FnMut(C) -> Result<C2, Er>>(
        self,
        map: &mut M,
    ) -> Result<RewardFormula<I, F, E, C2>, Er> {
        Ok(match self {
            RewardFormula::Instantaneous { k } => RewardFormula::Instantaneous { k },
            RewardFormula::Cumulative { k } => RewardFormula::Cumulative { k },
            RewardFormula::Finally { states: state } => RewardFormula::Finally {
                states: state.try_map_c(map)?,
            },
            RewardFormula::LongRunAverage => RewardFormula::LongRunAverage,
        })
    }
}

impl<I: Display, F: Display, E: Display, C: Display> Display for RewardFormula<I, F, E, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RewardFormula::Instantaneous { k } => write!(f, "I={k}"),
            RewardFormula::Cumulative { k } => write!(f, "C<={k}"),
            RewardFormula::Finally { states } => write!(f, "F {}", states),
            RewardFormula::LongRunAverage => write!(f, "LRA"),
        }
    }
}

/// A bound over the given numeric type `V`. Corresponds to syntax `< value`, `<= value`, `> value`
/// or `>= value`.
#[derive(Clone, PartialEq, Eq)]
pub struct Bound<V> {
    /// The comparison operator of the bound
    pub operator: BoundOperator,

    /// The value of the bound
    pub value: V,
}

impl<V> Bound<V> {
    /// Constructs a bound of form `> V`.
    pub fn greater_than(value: V) -> Self {
        Self {
            operator: BoundOperator::GreaterThan,
            value,
        }
    }
    /// Constructs a bound of form `>= V`.
    pub fn greater_or_equal(value: V) -> Self {
        Self {
            operator: BoundOperator::GreaterOrEqual,
            value,
        }
    }
    /// Constructs a bound of form `< V`.
    pub fn less_than(value: V) -> Self {
        Self {
            operator: BoundOperator::LessThan,
            value,
        }
    }
    /// Constructs a bound of form `<= V`.
    pub fn less_or_equal(value: V) -> Self {
        Self {
            operator: BoundOperator::LessOrEqual,
            value,
        }
    }

    /// Returns a bound where the value is stored as a mutable reference.
    pub fn as_mut(&mut self) -> Bound<&mut V> {
        Bound {
            operator: self.operator,
            value: &mut self.value,
        }
    }

    /// Returns a bound where the value is stored as a reference.
    pub fn as_ref(&self) -> Bound<&V> {
        Bound {
            operator: self.operator,
            value: &self.value,
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

    /// Returns the bound that is the logical negation of `self`.
    #[must_use]
    pub fn invert(self) -> Self {
        Self {
            operator: self.operator.invert(),
            value: self.value,
        }
    }

    /// Returns `true` if the given `value` adheres to the bound, otherwise false.
    ///
    /// # Examples
    ///
    /// ```
    /// # use probabilistic_properties::Bound;
    /// let bound = Bound::greater_or_equal(0.3);
    /// assert!(bound.accepts(0.5)); // because 0.5 >= 0.3
    /// assert!(bound.accepts(0.3)); // because 0.3 >= 0.3
    /// assert!(!bound.accepts(0.1)); // because not 0.1 >= 0.3
    /// ```
    pub fn accepts<VRef: Borrow<V>>(&self, value: VRef) -> bool
    where
        V: PartialOrd,
    {
        let Some(ord) = value.borrow().partial_cmp(&self.value) else {
            return false;
        };
        match self.operator {
            BoundOperator::LessThan => ord.is_lt(),
            BoundOperator::LessOrEqual => ord.is_le(),
            BoundOperator::GreaterThan => ord.is_gt(),
            BoundOperator::GreaterOrEqual => ord.is_ge(),
        }
    }
}

impl<V: Display> Display for Bound<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.operator, self.value)
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

impl BoundOperator {
    /// Returns the bounds operator that is the negation of the current operator.
    ///
    /// It transforms `<` into `>=`, `<=` into `>`, `>` into `<=` and `>=` into `<`.
    #[must_use]
    pub fn invert(&self) -> BoundOperator {
        match self {
            BoundOperator::LessThan => BoundOperator::GreaterOrEqual,
            BoundOperator::LessOrEqual => BoundOperator::GreaterThan,
            BoundOperator::GreaterThan => BoundOperator::LessOrEqual,
            BoundOperator::GreaterOrEqual => BoundOperator::LessThan,
        }
    }
}

impl Display for BoundOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BoundOperator::LessThan => write!(f, "<"),
            BoundOperator::LessOrEqual => write!(f, "<="),
            BoundOperator::GreaterThan => write!(f, ">"),
            BoundOperator::GreaterOrEqual => write!(f, ">="),
        }
    }
}

/// How non-determinism should be resolved
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum NonDeterminismKind {
    /// Choose actions that maximise the value
    Maximise,
    /// Choose actions that minimise the value
    Minimise,
}

trait DisplayNonDeterminism {
    fn to_str(&self) -> &'static str;
}

impl DisplayNonDeterminism for Option<NonDeterminismKind> {
    fn to_str(&self) -> &'static str {
        match self {
            None => "",
            Some(NonDeterminismKind::Minimise) => "min",
            Some(NonDeterminismKind::Maximise) => "max",
        }
    }
}

trait DisplayRewardsName {
    fn display(&self) -> String;
}

impl DisplayRewardsName for Option<String> {
    fn display(&self) -> String {
        match self {
            Some(name) => format!("{{\"{name}\"}}"),
            None => "".to_string(),
        }
    }
}
