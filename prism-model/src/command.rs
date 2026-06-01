use crate::expressions::Expression;
use crate::module::RenameRules;
use crate::spans::{FullSpan, Span};
use crate::{Displayable, Identifier, VariableReference};
use std::fmt::{Display, Formatter};

/// A [`Command`] using [`Identifier`] to refer to variables in expressions, instead of the default of
/// [`VariableReference`].
pub type CommandNamedVars<S: Span = FullSpan, A = Identifier<S>> =
    Command<Identifier<S>, S, Expression<Identifier<S>>, A>;

/// A PRISM command.
///
/// This corresponds to a PRISM entry `[`[`action`](Command::action)`] (`[`guard`](Command::guard)`) -> p1: u1 + ... + pn: un;`, where
/// `p1: u1 + ... pn: un` is a list of [`updates`](Command::updates).
///
/// # Semantics
///
/// A command can only be executed when [`guard`](Command::guard) evaluates to `true`. If this is
/// the case, then an update from [`updates`](Command::updates) is chosen according to the
/// probabilities stored in [`Update::probability`] and the [`assignments`](Update::assignments) of
/// the update are applied to the current valuations.
///
/// If [`updates`](Command::updates) is empty, this indicates the *empty update*, which is
/// equivalent to `1: ()`, i.e. doing nothing with probability 1. Call
/// [`Model::replace_empty_updates_with_identity_update()`](crate::Model::replace_empty_updates_with_identity_update)
/// to replace all empty updates with the equivalent empty update.
///
/// # Synchronisation
///
/// If `action` is `Some(name)`, we call the command *synchronising*. A synchronising command can
/// only be executed together with equally-named synchronising commands from other modules. For
/// example, suppose the following PRISM program fragment:
///
/// ```prism
/// module M1
///     [alpha] guard1 -> 0.5: (x'=x+1) + 0.5: (x'=x-1);
///     [alpha] guard2 -> 0.4: (x'=x+1) + 0.6: (x'=x-1);
/// endmodule
///
/// module M2
///     [beta] guard3 -> true;
/// endmodule
///
/// module M3
///     [alpha] guard4 -> 0.5: (y'=y+1) + 0.5: (y'=y-1);
///     [alpha] guard5 -> 0.4: (y'=y+1) + 0.6: (y'=y-1);
/// endmodule
///
/// module M4
///     [alpha] guard6 -> 0.5: (z'=z+1) + 0.5: (z'=z-1);
/// endmodule
/// ```
///
/// Then `M1` can only execute one of its `alpha` commands if `M3` and `M4` can also execute on of
/// their `alpha` commands. In particular, the following combinations are possible:
///
/// * `guard1`, `guard4` and `guard6`
/// * `guard2`, `guard4` and `guard6`
/// * `guard1`, `guard5` and `guard6`
/// * `guard2`, `guard5` and `guard6`
///
/// These synchronising commands are executed in a single step, i.e. `x`, `y` and `z` are updated
/// in the same step.
#[derive(PartialEq, Clone, Debug)]
pub struct Command<
    V = VariableReference,
    S: Span = FullSpan,
    E = Expression<V, S>,
    A = Identifier<S>,
> {
    /// The name of the synchronising action, or `None` if the command does not have a synchronising
    /// action. The PRISM command `[alpha] guard -> updates` corresponds to
    /// `action = Some(Identifier::new(action).unwrap())`, the PRISM command `[] guard -> updates`
    /// corresponds to `action = None`.
    pub action: Option<A>,

    /// The [`Span`] of the action of the action component of the PRISM command definition.
    ///
    /// # Example
    ///
    /// Here, `^` indicates characters covered by `action_span`
    ///
    /// ```prism
    /// [alpha] (x<20) -> 1.0: (x'=10);
    /// ^^^^^^^
    ///
    /// [] (x=10) -> 0.5: (x'=0) & 0.5: (x'=1);
    /// ^^
    /// ```
    ///
    /// `[` and `]` are covered as well. For synchronising actions, if `A = `[`Identifier`], the
    /// name itself (without `[` and `]`) is available at
    /// [`action`](Command::action)`.expect("Expected synchronising action").`[`span`](Identifier::span).
    pub action_span: S,

    /// The guard of the command. A command can only be executed when its guard evaluates to `true`.
    pub guard: E,

    /// The list of updates of the command.
    ///
    /// The probabilities must sum to `1`, unless the vector is empty (cf.
    /// [`Model::replace_empty_updates_with_identity_update()`](crate::Model::replace_empty_updates_with_identity_update)).
    pub updates: Vec<Update<V, S, E>>,

    /// The [`Span`] of the command.
    ///
    /// This span covers the entire command, starting at `[` of the action name and ending at `;`.
    pub span: S,
}

impl<V, S: Span, E, A> Command<V, S, E, A> {
    /// Constructs a new [`Command`] with empty list of updates and empty [`Span`].
    ///
    /// * `action`: If this is `Some(name)`, then a command with synchronising action `name` is
    /// created. If it is `None`, then the command is not synchronising.
    /// * `guard`: The guard of command.
    ///
    /// To construct a command with span, use [`Command::new_spanned()`].
    ///
    /// To add updates to the command, use [`Command::add_update()`] or construct the command with
    /// [`Command::with_updates()`].
    pub fn new(action: Option<A>, guard: E) -> Self {
        Self::new_spanned(action, S::empty(), guard, S::empty())
    }

    /// Constructs a new [`Command`] with empty list of updates and the given [`Span`].
    ///
    /// * `action`: The name of the action or `None` if the command is not synchronising.
    /// * `action_span`: The span covering the action (see also [`Command::action_span`])
    /// * `guard`: The guard of the command
    /// * `span`: The span of the entire command
    ///
    /// To construct a command with empty span, use [`Command::new()`].
    ///
    /// To add updates to the command, use [`Command::add_update()`] or construct the command with
    /// [`Command::with_updates_spanned()`].
    pub fn new_spanned(action: Option<A>, action_span: S, guard: E, span: S) -> Self {
        Self {
            action,
            action_span,
            guard,
            updates: Vec::new(),
            span,
        }
    }

    /// Constructs a new [`Command`] with the given list of updates and empty span.
    ///
    /// * `action`: The name of the action or `None` if the command is not synchronising.
    /// * `guard`: The guard of the command
    /// * `updates`: The list of updates
    ///
    /// To construct a command with span, use [`Command::with_updates_spanned()`].
    ///
    /// To construct a command without updates, use [`Command::new()`] and add updates with
    /// [`Command::add_update()`].
    pub fn with_updates(action: Option<A>, guard: E, updates: Vec<Update<V, S, E>>) -> Self {
        Self::with_updates_spanned(action, S::empty(), guard, updates, S::empty())
    }

    /// Constructs a new [`Command`] with the given list of updates and the given [`Span`].
    ///
    /// * `action`: The name of the action or `None` if the command is not synchronising.
    /// * `action_span`: The span covering the action (see also [`Command::action_span`])
    /// * `guard`: The guard of the command
    /// * `updates`: The list of updates
    /// * `span`: The span of the entire command
    ///
    /// To construct a command with empty span, use [`Command::with_updates()`].
    ///
    /// To construct a command without updates, use [`Command::new_spanned()`] and add updates with
    /// [`Command::add_update()`].
    pub fn with_updates_spanned(
        action: Option<A>,
        action_span: S,
        guard: E,
        updates: Vec<Update<V, S, E>>,
        span: S,
    ) -> Self {
        Self {
            action,
            action_span,
            guard,
            updates,
            span,
        }
    }

    /// Adds the given update to the command.
    pub fn add_update(&mut self, update: Update<V, S, E>) {
        self.updates.push(update);
    }
}

impl<V, S: Span, A> Command<V, S, Expression<V, S>, A> {
    /// Maps every [`Span`] of this `Command` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to [`action_span`](Self::action_span), [`guard`](Self::guard),
    /// every [`Update`] in [`updates`](Self::updates) and [`span`](Self::span).
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> Command<V, S2, Expression<V, S2>, A> {
        Command {
            action: self.action,
            action_span: map(self.action_span),
            guard: self.guard.map_span(map),
            updates: self.updates.into_iter().map(|u| u.map_span(map)).collect(),
            span: map(self.span),
        }
    }
}

impl<S: Span> Command<Identifier<S>, S, Expression<Identifier<S>, S>, Identifier<S>> {
    /// Constructs a new command by applying `rename_rules` to `self`.
    ///
    /// # Example
    ///
    /// Let `command` model PRISM command `[alpha] a -> 1.0: (x'=5)`.
    ///
    /// ```
    /// # use prism_model::{Command, CommandNamedVars, Expression, Identifier, Update, Assignment};
    /// let command: CommandNamedVars = Command::with_updates(
    ///     Some(Identifier::new("alpha").unwrap()),
    ///     Expression::var_or_const(Identifier::new("a").unwrap()),
    ///     vec![
    ///         Update::with_assignments(
    ///             Expression::float(1.0),
    ///             vec![Assignment::new(Identifier::new("x").unwrap(), Expression::int(5))]
    ///         )
    ///     ],
    /// );
    /// ```
    ///
    /// Let `rename_rules` express the rules `alpha` -> `beta`, `a` -> `b`, `x` -> `y`:
    ///
    /// ```
    /// # use prism_model::{Identifier, RenameRule, RenameRules};
    /// let rename_rules: RenameRules = RenameRules::with_rules(vec![
    ///     RenameRule::new(Identifier::new("alpha").unwrap(), Identifier::new("beta").unwrap()),
    ///     RenameRule::new(Identifier::new("a").unwrap(), Identifier::new("b").unwrap()),
    ///     RenameRule::new(Identifier::new("x").unwrap(), Identifier::new("y").unwrap()),
    /// ]);
    /// ```
    ///
    /// Then applying `rename_rules` to `command` yields the PRISM command
    /// `[beta] b -> 1.0: (y'=5)`.
    ///
    /// ```
    /// # use prism_model::{Command, CommandNamedVars, Expression, Identifier, Update, Assignment, RenameRule, RenameRules};
    /// # let command: CommandNamedVars = Command::with_updates(
    /// #     Some(Identifier::new("alpha").unwrap()),
    /// #     Expression::var_or_const(Identifier::new("a").unwrap()),
    /// #     vec![
    /// #         Update::with_assignments(
    /// #             Expression::float(1.0),
    /// #             vec![Assignment::new(Identifier::new("x").unwrap(), Expression::int(5))]
    /// #         )
    /// #     ],
    /// # );
    /// #
    /// # let rename_rules: RenameRules = RenameRules::with_rules(vec![
    /// #     RenameRule::new(Identifier::new("alpha").unwrap(), Identifier::new("beta").unwrap()),
    /// #     RenameRule::new(Identifier::new("a").unwrap(), Identifier::new("b").unwrap()),
    /// #     RenameRule::new(Identifier::new("x").unwrap(), Identifier::new("y").unwrap()),
    /// # ]);
    /// #
    /// assert_eq!(
    ///     command.renamed(&rename_rules),
    ///     Command::with_updates(
    ///         Some(Identifier::new("beta").unwrap()),
    ///         Expression::var_or_const(Identifier::new("b").unwrap()),
    ///         vec![
    ///             Update::with_assignments(
    ///                 Expression::float(1.0),
    ///                 vec![Assignment::new(Identifier::new("y").unwrap(), Expression::int(5))]
    ///             )
    ///         ],
    ///     )
    /// );
    /// ```
    pub fn renamed(&self, rename_rules: &RenameRules<S>) -> Self {
        Self {
            action: self.action.as_ref().map(|a| a.renamed(rename_rules)),
            action_span: self.action_span.clone(),
            guard: self.guard.renamed(rename_rules),
            updates: self
                .updates
                .iter()
                .map(|u| u.renamed(rename_rules))
                .collect(),
            span: self.span.clone(),
        }
    }
}

impl<V, S: Span, E, A> crate::private::Sealed for Command<V, S, E, A> {}
impl<Ctx, V: Displayable<Ctx>, S: Span, E: Displayable<Ctx>, A: Display> Displayable<Ctx>
    for Command<V, S, E, A>
{
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result {
        write!(f, "[")?;
        if let Some(action) = &self.action {
            write!(f, "{}", action)?;
        }
        write!(f, "] ")?;
        write!(f, "{} -> ", self.guard.displayable(context))?;
        if self.updates.len() == 0 {
            write!(f, "true")?;
        } else {
            for (i, update) in self.updates.iter().enumerate() {
                if i > 0 {
                    write!(f, " + ")?;
                }
                write!(f, "{}", update.displayable(context))?;
            }
        }

        write!(f, ";")
    }
}

/// An [`Update`] using [`Identifier`] to refer to variables in expressions, instead of the default
/// of [`VariableReference`].
pub type UpdateNamedVars<S: Span = FullSpan> =
    Update<Identifier<S>, S, Expression<Identifier<S>, S>>;

/// An update, i.e. a probability and a list of variable assignments.
///
/// An update corresponds to the prism syntax `probability: (assignment_1) & ... & (assignment_n)`.
/// A [`Command`] contains several updates, with their total probability equal to `1`.
#[derive(PartialEq, Clone, Debug)]
pub struct Update<V = VariableReference, S: Span = FullSpan, E = Expression<V, S>> {
    /// The probability of the update.
    ///
    /// This needs to evaluate to float, so the expression must be of type float or int.
    pub probability: E,

    /// A (potentially empty) list of assignments.
    ///
    /// Each assignment assigns a new value to a variable. The assignments are executed in parallel:
    /// Given assignments `(x'=5)` and `(y'=x+y)` for old values `x = 2` and `y=10`, the new values
    /// are `x = 5` and `y = 12`.
    pub assignments: Vec<Assignment<V, S, E>>,

    /// The [`Span`] of the update.
    pub span: S,
}

impl<V, S: Span, E> Update<V, S, E> {
    /// Constructs an update with the given probability, empty [`Span`] and empty list of assignments.
    ///
    /// To construct an update with given span, use [`Update::new_spanned()`]. To add assignments,
    /// use [`Update::add_assignment()`] or construct the update with
    /// [`Update::with_assignments()`].
    pub fn new(probability: E) -> Self {
        Self::new_spanned(probability, S::empty())
    }

    /// Constructs an update with the given probability, given [`Span`] and empty list of assignments.
    ///
    /// To construct an update with empty span, use [`Update::new()`]. To add assignments, use
    /// [`Update::add_assignment()`] or construct the update with
    /// [`Update::with_assignments_spanned()`].
    pub fn new_spanned(probability: E, span: S) -> Self {
        Self {
            probability,
            assignments: Vec::new(),
            span,
        }
    }

    /// Constructs an update with the given probability, empty [`Span`] and given list of
    /// assignments.
    ///
    /// To construct an update with given span, use [`Update::with_assignments_spanned()`].
    /// To construct an update without assignments, use [`Update::new()`] and updates with
    /// [`Update::add_assignment()`].
    pub fn with_assignments(probability: E, assignments: Vec<Assignment<V, S, E>>) -> Self {
        Self::with_assignments_spanned(probability, assignments, S::empty())
    }

    /// Constructs an update with the given probability, given [`Span`] and given list of
    /// assignments.
    ///
    /// To construct an update with empty span, use [`Update::with_assignments()`].
    /// To construct an update without assignments, use [`Update::new_spanned()`] and updates with
    /// [`Update::add_assignment()`]
    pub fn with_assignments_spanned(
        probability: E,
        assignments: Vec<Assignment<V, S, E>>,
        span: S,
    ) -> Self {
        Self {
            probability,
            assignments,
            span,
        }
    }

    /// Adds a new assignment to the update.
    ///
    /// To construct an update with given set of assignments, use [`Update::with_assignments()`]
    /// or [`Update::with_assignments_spanned()`].
    pub fn add_assignment(&mut self, assignment: Assignment<V, S, E>) {
        self.assignments.push(assignment);
    }
}
impl<V, S: Span> Update<V, S, Expression<V, S>> {
    /// Maps every [`Span`] of this `Update` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to [`probability`](Self::probability), every [`Assignment`] in
    /// [`assignments`](Self::assignments) and [`span`](Self::span).
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> Update<V, S2, Expression<V, S2>> {
        let mut update = Update::new_spanned(self.probability.map_span(map), map(self.span));
        for assignment in self.assignments {
            update.assignments.push(assignment.map_span(map));
        }
        update
    }
}

impl<S: Span> Update<Identifier<S>, S, Expression<Identifier<S>, S>> {
    /// Constructs a new update by applying `rename_rules` to `self`.
    ///
    /// See [`Command::renamed()`] for a detailed example.
    pub fn renamed(&self, rename_rules: &RenameRules<S>) -> Self {
        Self {
            probability: self.probability.renamed(rename_rules),
            assignments: self
                .assignments
                .iter()
                .map(|a| a.renamed(rename_rules))
                .collect(),
            span: self.span.clone(),
        }
    }
}

impl<V, S: Span, E> crate::private::Sealed for Update<V, S, E> {}
impl<Ctx, V: Displayable<Ctx>, S: Span, E: Displayable<Ctx>> Displayable<Ctx> for Update<V, S, E> {
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result {
        // This would produce nicer output, but require E to provide some way of inspecting its
        // value, e.g. by requiring a separate IsOne trait.
        // match &self.probability {
        //     Expression::Int(1, _) => {}
        //     e => {
        //         write!(f, "{} : ", e)?;
        //     }
        // }
        write!(f, "{} : ", self.probability.displayable(context))?;

        let mut is_first = true;
        for assignment in &self.assignments {
            if !is_first {
                write!(f, " & ")?;
            }
            is_first = false;
            write!(f, "({})", assignment.displayable(context))?;
        }

        Ok(())
    }
}

/// A [`Assignment`] using [`Identifier`] to refer to variables in expressions, instead of the
/// default of [`VariableReference`].
pub type AssignmentNamedVars<S: Span = FullSpan> =
    Assignment<Identifier<S>, S, Expression<Identifier<S>, S>>;

/// An assignment, consisting of a variable target and a new value for the variable.
///
/// This corresponds to the PRISM fragment `(target'=value)`.
///
/// Assignments are usually grouped in an [`Update`].
#[derive(PartialEq, Clone, Debug)]
pub struct Assignment<V = VariableReference, S: Span = FullSpan, E = Expression<V, S>> {
    /// The variable that updated in this assignment.
    pub target: V,

    /// The new value for [`target`](Assignment::target).
    pub value: E,

    /// The [`Span`] of `target`.
    ///
    /// If `V = `[`Identifier`], this is equal to `target.`[`span`](Identifier::span). If
    /// `V = `[`VariableReference`], `target_span`  is useful because `VariableReference` does not
    /// store a span.
    pub target_span: S,

    /// The [`Span`] of the assignment.
    pub span: S,
}

impl<V, S: Span, E> Assignment<V, S, E> {
    /// Constructs a new assignment with given target variable and value.
    ///
    /// * `target`: The variable that is updated in this assignment
    /// * `value`: The new value for variable `target`
    pub fn new(target: V, value: E) -> Self {
        Self::new_spanned(target, value, S::empty(), S::empty())
    }

    /// Constructs a new assignment with given target variable, value and [`Span`].
    ///
    /// * `target`: The variable that is updated in this assignment
    /// * `value`: The new value for variable `target`
    /// * `target_span`: The [`Span`] covering `target`. See
    ///     [`Self::target_span`](Assignment::target_span) for details
    /// * `span`: The [`Span`] covering the entire assignment
    pub fn new_spanned(target: V, value: E, target_span: S, span: S) -> Self {
        Self {
            target,
            value,
            target_span,
            span,
        }
    }
}
impl<V, S: Span> Assignment<V, S, Expression<V, S>> {
    /// Maps every [`Span`] of this `Assignment` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to [`value`](Self::value), [`target_span`](Self::target_span) and
    /// [`span`](Self::span).
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> Assignment<V, S2, Expression<V, S2>> {
        Assignment {
            target: self.target,
            value: self.value.map_span(map),
            target_span: map(self.target_span),
            span: map(self.span),
        }
    }
}
impl<S: Span> Assignment<Identifier<S>, S, Expression<Identifier<S>, S>> {
    /// Constructs a new assignment by applying `rename_rules` to `self`.
    ///
    /// See [`Command::renamed()`] for a detailed example.
    pub fn renamed(&self, rename_rules: &RenameRules<S>) -> Self {
        Self {
            target: self.target.renamed(rename_rules),
            value: self.value.renamed(rename_rules),
            target_span: self.target_span.clone(),
            span: self.span.clone(),
        }
    }
}

impl<V, S: Span, E> crate::private::Sealed for Assignment<V, S, E> {}
impl<Ctx, V: Displayable<Ctx>, S: Span, E: Displayable<Ctx>> Displayable<Ctx>
    for Assignment<V, S, E>
{
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result {
        write!(
            f,
            "{}'={}",
            self.target.displayable(context),
            self.value.displayable(context)
        )
    }
}
