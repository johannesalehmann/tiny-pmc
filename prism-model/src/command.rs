use crate::expressions::Expression;
use crate::module::RenameRules;
use crate::spans::{FullSpan, Span};
use crate::{Displayable, Identifier, VariableReference};
use std::fmt::{Display, Formatter};

pub type CommandNamedVars<S: Span = FullSpan, A = Identifier<S>> =
    Command<Identifier<S>, S, Expression<Identifier<S>>, A>;

pub struct Command<
    V = VariableReference,
    S: Span = FullSpan,
    E = Expression<V, S>,
    A = Identifier<S>,
> {
    pub action: Option<A>,
    pub action_span: S,
    pub guard: E,
    pub updates: Vec<Update<V, S, E>>,
    pub span: S,
}

impl<V, S: Span, E, A> Command<V, S, E, A> {
    pub fn new(action: Option<A>, guard: E) -> Self {
        Self::new_spanned(action, S::empty(), guard, S::empty())
    }
    pub fn new_spanned(action: Option<A>, action_span: S, guard: E, span: S) -> Self {
        Self {
            action,
            action_span,
            guard,
            updates: Vec::new(),
            span,
        }
    }

    pub fn with_updates(action: Option<A>, guard: E, updates: Vec<Update<V, S, E>>) -> Self {
        Self::with_updates_spanned(action, S::empty(), guard, updates, S::empty())
    }

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
}

impl<V, S: Span, A> Command<V, S, Expression<V, S>, A> {
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

pub type UpdateNamedVars<S: Span = FullSpan> =
    Update<Identifier<S>, S, Expression<Identifier<S>, S>>;
pub struct Update<V = VariableReference, S: Span = FullSpan, E = Expression<V, S>> {
    pub probability: E,
    pub assignments: Vec<Assignment<V, S, E>>,
    pub span: S,
}

impl<V, S: Span, E> Update<V, S, E> {
    pub fn new(probability: E) -> Self {
        Self::new_spanned(probability, S::empty())
    }
    pub fn new_spanned(probability: E, span: S) -> Self {
        Self {
            probability,
            assignments: Vec::new(),
            span,
        }
    }
    pub fn with_assignments(probability: E, assignments: Vec<Assignment<V, S, E>>) -> Self {
        Self::with_assignments_spanned(probability, assignments, S::empty())
    }
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
}
impl<V, S: Span> Update<V, S, Expression<V, S>> {
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> Update<V, S2, Expression<V, S2>> {
        let mut update = Update::new_spanned(self.probability.map_span(map), map(self.span));
        for assignment in self.assignments {
            update.assignments.push(assignment.map_span(map));
        }
        update
    }
}

impl<S: Span> Update<Identifier<S>, S, Expression<Identifier<S>, S>> {
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

pub type AssignmentNamedVars<S: Span = FullSpan> =
    Assignment<Identifier<S>, S, Expression<Identifier<S>, S>>;
pub struct Assignment<V = VariableReference, S: Span = FullSpan, E = Expression<V, S>> {
    pub target: V,
    pub value: E,
    pub target_span: S,
    pub span: S,
}

impl<V, S: Span, E> Assignment<V, S, E> {
    pub fn new(target: V, value: E) -> Self {
        Self::new_spanned(target, value, S::empty(), S::empty())
    }
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
