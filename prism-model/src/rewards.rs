use crate::spans::{FullSpan, Span};
use crate::{Displayable, Expression, Identifier, VariableReference};
use std::fmt::{Display, Formatter};

/// A [`RewardsManager`] using [`Identifier`] to refer to variables in expressions, instead of the
/// default of [`VariableReference`].
pub type RewardsManagerNamedVars<S: Span = FullSpan, A = Identifier<S>> =
    RewardsManager<S, Expression<Identifier<S>, S>, A>;

/// A collection of rewards. Each reward has a name and assigns a value to model states or actions.
#[derive(PartialEq, Clone, Debug)]
pub struct RewardsManager<
    S: Span = FullSpan,
    E = Expression<VariableReference, S>,
    A = Identifier<S>,
> {
    /// The set of rewards.
    ///
    /// Instead of adding rewards to this directly, use [`RewardsManager::add()`], which ensures
    /// there are no duplicates.
    pub rewards: Vec<Rewards<S, E, A>>,
}

impl<S: Span, E, A> RewardsManager<S, E, A> {
    /// Creates an empty `RewardsManager`.
    ///
    /// To add rewards, use [`RewardsManager::add()`] or create the `RewardsManager` with
    /// [`RewardsManager::with_rewards()`].
    pub fn new() -> Self {
        Self {
            rewards: Vec::new(),
        }
    }

    /// Creates a `RewardsManager` with the given rewards.
    ///
    /// To create an empty `RewardsManager`, use [`RewardsManager::new()`] and add rewards with
    /// [`RewardsManager::add()`].
    pub fn with_rewards(rewards: Vec<Rewards<S, E, A>>) -> Self {
        Self { rewards }
    }

    /// Returns the reward with the given index or `None` if the index is out of bounds.
    pub fn get(&self, index: usize) -> Option<&Rewards<S, E, A>> {
        self.rewards.get(index)
    }

    /// Adds a new reward to this `RewardsManager`.
    ///
    /// If a reward with the same name already exists, returns [`AddRewardsError::RewardsExist`].
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
    /// Maps the [`Span`] of every [`Rewards`] in this `RewardsManager` according to mapping
    /// function `map`.
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

/// Error produced while adding a reward to a [`RewardsManager`]
#[derive(Debug)]
pub enum AddRewardsError {
    /// A reward with this name already exists
    RewardsExist {
        /// The index of the existing reward with the same name.
        ///
        /// Use [`RewardsManager::get(index)`](RewardsManager::get()) to retrieve details about the
        /// existing reward.
        index: usize,
    },
}

/// A [`Rewards`] using [`Identifier`] to refer to variables in expressions, instead of the default
/// of [`VariableReference`].
pub type RewardsNamedVars<S: Span = FullSpan, A = Identifier<S>> =
    Rewards<S, Expression<Identifier<S>, S>, A>;

/// A rewards structure, corresponding to a single `rewards "name" ... endrewards` block of a PRISM
/// model.
#[derive(PartialEq, Clone, Debug)]
pub struct Rewards<S: Span = FullSpan, E = Expression<VariableReference, S>, A = Identifier<S>> {
    /// The name of the rewards structure. Naming reward structures is optional in PRISM models.
    pub name: Option<Identifier<S>>,

    /// The entries of the rewards structure.
    ///
    /// Each entry corresponds to an entry in the `rewards "name" ... endrewards` block, i.e. either
    /// `condition: value;` or `[action] condition: value;`.
    pub entries: Vec<RewardsElement<S, E, A>>,

    /// The [`Span`] of the rewards structure.
    pub span: S,
}

impl<S: Span, E, A> Rewards<S, E, A> {
    /// Constructs a rewards structure with given name, no entries and empty [`Span`].
    ///
    /// To add entries, use [`Rewards::add()`] or construct the rewards structure with
    /// [`Rewards::with_entries()`].
    ///
    /// To construct a rewards structure with given span, use [`Rewards::new_spanned()`].
    pub fn new(name: Option<Identifier<S>>) -> Self {
        Self::new_spanned(name, S::empty())
    }

    /// Constructs a rewards structure with given name, no entries and given [`Span`].
    ///
    /// To add entries, use [`Rewards::add()`] or construct the rewards structure with
    /// [`Rewards::with_entries()`].
    ///
    /// To construct a rewards structure with empty span, use [`Rewards::new()`].
    pub fn new_spanned(name: Option<Identifier<S>>, span: S) -> Self {
        Self {
            name: name.into(),
            entries: Vec::new(),
            span,
        }
    }

    /// Constructs a rewards structure with given name, given entries and empty [`Span`].
    ///
    /// To construct an empty rewards structure, use [`Rewards::new()`] and add entries with
    /// [`Rewards::add()`].
    ///
    /// To construct a rewards structure with given span, use [`Rewards::with_entries_spanned()`].
    pub fn with_entries(
        name: Option<Identifier<S>>,
        entries: Vec<RewardsElement<S, E, A>>,
    ) -> Self {
        Self::with_entries_spanned(name, entries, S::empty())
    }

    /// Constructs a rewards structure with given name, given entries and given [`Span`].
    ///
    /// To construct an empty rewards structure, use [`Rewards::new()`] and add entries with
    /// [`Rewards::add()`].
    ///
    /// To construct a rewards structure with empty span, use [`Rewards::with_entries()`].
    pub fn with_entries_spanned(
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

    /// Adds a new rewards element to the rewards structure. A rewards element corresponds to
    /// the PRISM syntax `condition: value;` or `[action] condition: value`
    pub fn add(&mut self, entry: RewardsElement<S, E, A>) {
        self.entries.push(entry);
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

/// The target of a [`RewardsElement`], either a state or an action.
///
/// * [`RewardsTarget::State`] corresponds to PRISM syntax `condition: value;`.
/// * [`RewardsTarget::Action`] corresponds to PRISM syntax `[action] condition: value`
#[derive(PartialEq, Clone, Debug)]
pub enum RewardsTarget<A = Identifier<FullSpan>> {
    /// A [`RewardsElement`] targeting a state, corresponding to PRISM syntax `condition: value;`.
    ///
    /// This indicates that the reward is given in every state that fulfils
    /// [`RewardsElement::condition`].
    State,

    /// A [`RewardsElement`] targeting a transition.
    ///
    /// * `RewardsElement::Action(None)` corresponds to PRISM syntax `[] condition: value;`.
    ///
    ///   This indicates that the reward is assigned to every unlabelled transition from a state
    ///   where [`RewardsElement::condition`] holds.
    ///
    /// * `RewardsElement::Action(Some(id))` corresponds to PRISM syntax `[id] condition: value;`.
    ///
    ///   This indicates that the reward is assigned to every transition labelled with `id` from a
    ///   state where [`RewardsElement::condition`] holds.
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

/// An entry in a [rewards structure](Rewards), corresponding to PRISM syntax `condition: value;`
/// or `[action] condition: value;`.
///
/// Each `RewardsElement` has a [`value`](RewardsElement::value) that is assigned to states in which
/// [`condition`](RewardsElement::condition) holds. If [`target`](RewardsElement::target) is
/// [`RewardsTarget::Action(name)`](RewardsTarget::Action), the reward is instead assigned to
/// transitions of the state labelled with action `name`. See [`RewardsTarget`] for details.
#[derive(PartialEq, Clone, Debug)]
pub struct RewardsElement<
    S: Span = FullSpan,
    E = Expression<VariableReference, S>,
    A = Identifier<S>,
> {
    /// The condition of the reward.
    ///
    /// The reward's [`value`](RewardsElement::value) is assigned when
    /// this evaluates to `true`.
    pub condition: E,

    /// The value of the reward.
    ///
    /// If [`condition`](RewardsElement::condition) evaluates to `true`, this expression is
    /// evaluated and assigned to the corresponding states or transitions(depending on
    /// [`target`](RewardsElement::target)).
    pub value: E,

    /// The target of the reward.
    ///
    /// This decides whether the reward is assigned to states or their
    /// outgoing transitions. See [`RewardsTarget`] for details.
    pub target: RewardsTarget<A>,

    /// The [`Span`] of the reward.
    pub span: S,
}

impl<S: Span, E, A> RewardsElement<S, E, A> {
    /// Constructs a `RewardsElement` with given condition and value, targeting states.
    ///
    /// This `RewardsElement` has target [`RewardsTarget::State`]. Use
    /// [`RewardsElement::with_action()`] or [`RewardsElement::with_target()`] for other targets.
    ///
    /// The [`Span`] is empty. To construct a `RewardsElement` with a given span, use
    /// [`RewardsElement::new_spanned()`].
    pub fn new(condition: E, value: E) -> Self {
        Self::new_spanned(condition, value, S::empty())
    }

    /// Constructs a `RewardsElement` with given condition, value and [`Span`], targeting states.
    ///
    /// This `RewardsElement` has target [`RewardsTarget::State`]. Use
    /// [`RewardsElement::with_action_spanned()`] or [`RewardsElement::with_target_spanned()`] for
    /// other targets.
    ///
    /// To construct a `RewardsElement` with an empty span, use [`RewardsElement::new()`].
    pub fn new_spanned(condition: E, value: E, span: S) -> Self {
        Self {
            condition,
            value,
            target: RewardsTarget::State,
            span,
        }
    }

    /// Constructs a `RewardsElement` with given condition and value, targeting the given action.
    ///
    /// This `RewardsElement` has target [`RewardsTarget::Action(action)`](RewardsTarget::Action).
    /// Use [`RewardsElement::new()`] or [`RewardsElement::with_target()`] for other targets.
    ///
    /// The [`Span`] is empty. To construct a `RewardsElement` with a given span, use
    /// [`RewardsElement::with_action_spanned()`].
    pub fn with_action(condition: E, value: E, action: Option<A>) -> Self {
        Self::with_action_spanned(condition, value, action, S::empty())
    }

    /// Constructs a `RewardsElement` with given condition and value and [`Span`], targeting the
    /// given action.
    ///
    /// This `RewardsElement` has target [`RewardsTarget::Action(action)`](RewardsTarget::Action).
    /// Use [`RewardsElement::new_spanned()`] or [`RewardsElement::with_target_spanned()`] for other
    /// targets.
    ///
    /// To construct a `RewardsElement` with an empty span, use [`RewardsElement::with_action()`].
    pub fn with_action_spanned(condition: E, value: E, action: Option<A>, span: S) -> Self {
        Self {
            condition,
            value,
            target: RewardsTarget::Action(action),
            span,
        }
    }

    /// Constructs a `RewardsElement` with given condition, value and target.
    ///
    /// For specific targets, use [`RewardsElement::new()`] or [`RewardsElement::with_action()`].
    ///
    /// The [`Span`] is empty. To construct a `RewardsElement` with a given span, use
    /// [`RewardsElement::with_target_spanned()`].
    pub fn with_target(condition: E, value: E, target: RewardsTarget<A>) -> Self {
        Self::with_target_spanned(condition, value, target, S::empty())
    }

    /// Constructs a `RewardsElement` with given condition, value, target and [`Span`].
    ///
    /// For specific targets, use [`RewardsElement::new_spanned()`] or
    /// [`RewardsElement::with_action_spanned()`].
    ///
    /// To construct a `RewardsElement` with an empty span, use [`RewardsElement::with_target()`].
    pub fn with_target_spanned(condition: E, value: E, target: RewardsTarget<A>, span: S) -> Self {
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
    /// `map` is applied to [`condition`](Self::condition), [`value`](Self::value), and
    /// [`span`](Self::span).
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
