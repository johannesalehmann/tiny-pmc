use crate::spans::{FullSpan, Span};
use crate::{Displayable, Expression, Identifier, VariableReference};
use std::fmt::Formatter;

/// A [`LabelManager`] using [`Identifier`] to refer to variables in expressions, instead of the
/// default of [`VariableReference`].
pub type LabelManagerNamedVars<S: Span = FullSpan> = LabelManager<S, Expression<Identifier<S>>>;

pub struct LabelManager<S: Span = FullSpan, E = Expression<VariableReference, S>> {
    pub labels: Vec<Label<S, E>>,
}

impl<S: Span, E> LabelManager<S, E> {
    pub fn new() -> Self {
        Self { labels: Vec::new() }
    }

    pub fn with_labels(mut labels: Vec<Label<S, E>>) -> Result<Self, AddLabelError> {
        let mut res = Self::new();

        for label in labels.drain(..) {
            res.add_label(label)?;
        }
        Ok(res)
    }

    pub fn add_label(&mut self, label: Label<S, E>) -> Result<(), AddLabelError> {
        for (index, other_label) in self.labels.iter().enumerate() {
            if other_label.name == label.name {
                return Err(AddLabelError::LabelExists { index });
            }
        }
        self.labels.push(label);
        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<&Label<S, E>> {
        self.labels.get(index)
    }

    pub fn index_of_name(&self, name: &str) -> Option<usize> {
        for (i, label) in self.labels.iter().enumerate() {
            if label.name.name == name {
                return Some(i);
            }
        }
        None
    }

    pub fn by_name(&self, name: &str) -> Option<&Label<S, E>> {
        for label in &self.labels {
            if label.name.name == name {
                return Some(label);
            }
        }
        None
    }
}

impl<V, S: Span> LabelManager<S, Expression<V, S>> {
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> LabelManager<S2, Expression<V, S2>> {
        LabelManager {
            labels: self.labels.into_iter().map(|l| l.map_span(map)).collect(),
        }
    }
}

impl<S: Span, E> crate::private::Sealed for LabelManager<S, E> {}
impl<Ctx, E: Displayable<Ctx>, S: Span> Displayable<Ctx> for LabelManager<S, E> {
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result {
        for formula in &self.labels {
            writeln!(f, "{}", formula.displayable(context))?;
        }
        if self.labels.len() > 0 {
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum AddLabelError {
    LabelExists { index: usize },
}

/// A [`Label`] using [`Identifier`] to refer to variables in expressions, instead of the default of
/// [`VariableReference`].
pub type LabelNamedVars<S: Span = FullSpan> = Label<S, Expression<Identifier<S>, S>>;
pub struct Label<S: Span = FullSpan, E = Expression<VariableReference, S>> {
    pub name: Identifier<S>,
    pub condition: E,
    pub span: S,
}

impl<S: Span, E> Label<S, E> {
    pub fn new(name: Identifier<S>, condition: E) -> Self {
        Self::new_spanned(name, condition, S::empty())
    }
    pub fn new_spanned(name: Identifier<S>, condition: E, span: S) -> Self {
        Self {
            name,
            condition,
            span,
        }
    }
}
impl<V, S: Span> Label<S, Expression<V, S>> {
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> Label<S2, Expression<V, S2>> {
        Label {
            name: self.name.map_span(map),
            condition: self.condition.map_span(map),
            span: map(self.span),
        }
    }
}

impl<S: Span, E> crate::private::Sealed for Label<S, E> {}
impl<Ctx, E: Displayable<Ctx>, S: Span> Displayable<Ctx> for Label<S, E> {
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result {
        writeln!(
            f,
            "label \"{}\" = {};",
            self.name,
            self.condition.displayable(context)
        )
    }
}
