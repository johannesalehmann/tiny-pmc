use crate::spans::{FullSpan, Span};
use crate::{Displayable, Expression, Identifier, VariableReference};
use std::fmt::{Display, Formatter};

/// A [`RewardsManager`] using [`Identifier`] to refer to variables in expressions, instead of the
/// default of [`VariableReference`].
pub type RewardsManagerNamedVars<S: Span = FullSpan, A = Identifier<S>> =
    RewardsManager<S, Expression<Identifier<S>, S>, A>;
pub struct RewardsManager<
    S: Span = FullSpan,
    E = Expression<VariableReference, S>,
    A = Identifier<S>,
> {
    pub rewards: Vec<Rewards<S, E, A>>,
}

impl<S: Span, E, A> RewardsManager<S, E, A> {
    pub fn new() -> Self {
        Self {
            rewards: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Rewards<S, E, A>> {
        self.rewards.get(index)
    }

    pub fn add(&mut self, rewards: Rewards<S, E, A>) -> Result<(), AddRewardsError> {
        for (index, other_rewards) in self.rewards.iter().enumerate() {
            if other_rewards.name == rewards.name {
                return Err(AddRewardsError::RewardsExist { index });
            }
        }
        self.rewards.push(rewards);
        Ok(())
    }
}

impl<V, S: Span, A> RewardsManager<S, Expression<V, S>, A> {
    /// Maps the [`Span`] of every [`Reward`] in this `RewardsElement` according to mapping function
    /// `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> RewardsManager<S2, Expression<V, S2>, A> {
        RewardsManager {
            rewards: self.rewards.into_iter().map(|r| r.map_span(map)).collect(),
        }
    }
}

#[derive(Debug)]
pub enum AddRewardsError {
    RewardsExist { index: usize },
}

/// A [`Rewards`] using [`Identifier`] to refer to variables in expressions, instead of the default
/// of [`VariableReference`].
pub type RewardsNamedVars<S: Span = FullSpan, A = Identifier<S>> =
    Rewards<S, Expression<Identifier<S>, S>, A>;

pub struct Rewards<S: Span = FullSpan, E = Expression<VariableReference, S>, A = Identifier<S>> {
    pub name: Option<Identifier<S>>,
    pub entries: Vec<RewardsElement<S, E, A>>,
    pub span: S,
}

impl<S: Span, E, A> Rewards<S, E, A> {
    pub fn new(name: Option<Identifier<S>>, span: S) -> Self {
        Self {
            name: name.into(),
            entries: Vec::new(),
            span,
        }
    }
    pub fn with_entries(
        name: Option<Identifier<S>>,
        entries: Vec<RewardsElement<S, E, A>>,
        span: S,
    ) -> Self {
        Self {
            name,
            entries,
            span,
        }
    }
}
impl<V, S: Span, A> Rewards<S, Expression<V, S>, A> {
    /// Maps every [`Span`] of this `Rewards` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to [`name`](Self::name), every [`RewardsElement`] in
    /// [`entries`](Self::entries) and [`span`](Self::span).
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> Rewards<S2, Expression<V, S2>, A> {
        Rewards {
            name: self.name.map(|i| i.map_span(map)),
            entries: self.entries.into_iter().map(|e| e.map_span(map)).collect(),
            span: map(self.span),
        }
    }
}

impl<S: Span, E, A> crate::private::Sealed for Rewards<S, E, A> {}
impl<Ctx, S: Span, E: Displayable<Ctx>, A: Display> Displayable<Ctx> for Rewards<S, E, A> {
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result {
        write!(f, "rewards")?;
        if let Some(name) = &self.name {
            write!(f, " \"{}\"", name)?;
        }
        writeln!(f)?;
        for element in &self.entries {
            writeln!(f, "    {}", element.displayable(context))?;
        }
        writeln!(f, "endrewards")
    }
}

#[derive(Clone)]
pub enum RewardsTarget<A = Identifier<FullSpan>> {
    State,
    Action(Option<A>),
}
impl<A: Display> Display for RewardsTarget<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RewardsTarget::State => Ok(()),
            RewardsTarget::Action(Some(a)) => {
                write!(f, "[{}] ", a)
            }
            RewardsTarget::Action(None) => {
                write!(f, "[] ")
            }
        }
    }
}

/// A [`RewardsElement`] using [`Identifier`] to refer to variables in expressions, instead of the
/// default of [`VariableReference`].
pub type RewardsElementNamedVars<S: Span = FullSpan, A = Identifier<S>> =
    RewardsElement<S, Expression<Identifier<S>, S>, A>;

pub struct RewardsElement<
    S: Span = FullSpan,
    E = Expression<VariableReference, S>,
    A = Identifier<S>,
> {
    pub condition: E,
    pub value: E,
    pub target: RewardsTarget<A>,
    pub span: S,
}

impl<S: Span, E, A> RewardsElement<S, E, A> {
    pub fn new(condition: E, value: E, span: S) -> Self {
        Self {
            condition,
            value,
            target: RewardsTarget::State,
            span,
        }
    }
    pub fn with_action(condition: E, value: E, action: Option<A>, span: S) -> Self {
        Self {
            condition,
            value,
            target: RewardsTarget::Action(action),
            span,
        }
    }
    pub fn with_target(condition: E, value: E, target: RewardsTarget<A>, span: S) -> Self {
        Self {
            condition,
            value,
            target,
            span,
        }
    }
}
impl<V, S: Span, A> RewardsElement<S, Expression<V, S>, A> {
    /// Maps every [`Span`] of this `RewardsElement` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to [`range`](Self::range), [`name`](Self::name),
    /// [`initial_value`](Self::initial_value) and [`span`](Self::span).
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> RewardsElement<S2, Expression<V, S2>, A> {
        RewardsElement {
            condition: self.condition.map_span(map),
            value: self.value.map_span(map),
            target: self.target,
            span: map(self.span),
        }
    }
}

impl<S: Span, E, A> crate::private::Sealed for RewardsElement<S, E, A> {}
impl<Ctx, A: Display, E: Displayable<Ctx>, S: Span> Displayable<Ctx> for RewardsElement<S, E, A> {
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result {
        write!(
            f,
            "{}{}:{};",
            self.target,
            self.condition.displayable(context),
            self.value.displayable(context)
        )
    }
}
