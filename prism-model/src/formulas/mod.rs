mod formula_dependencies;

pub use formula_dependencies::*;
use std::fmt::Formatter;

use crate::spans::{FullSpan, Span};
use crate::{Displayable, Expression, Identifier, VariableReference};

/// A [`FormulaManager`] using [`Identifier`] to refer to variables in expressions, instead of the
/// default of [`VariableReference`].
pub type FormulaManagerNamedVars<S: Span = FullSpan> =
    FormulaManager<S, Expression<Identifier<S>, S>>;

pub struct FormulaManager<S: Span = FullSpan, E = Expression<VariableReference, S>> {
    pub formulas: Vec<Formula<S, E>>,
}

impl<S: Span, E> FormulaManager<S, E> {
    pub fn new() -> Self {
        Self {
            formulas: Vec::new(),
        }
    }

    pub fn with_formulas(mut formulas: Vec<Formula<S, E>>) -> Result<Self, AddFormulaError> {
        let mut res = Self::new();

        for formula in formulas.drain(..) {
            res.add_formula(formula)?;
        }
        Ok(res)
    }

    pub fn get(&self, index: usize) -> Option<&Formula<S, E>> {
        self.formulas.get(index)
    }

    pub fn add_formula(&mut self, formula: Formula<S, E>) -> Result<(), AddFormulaError> {
        for (index, other_formula) in self.formulas.iter().enumerate() {
            if other_formula.name == formula.name {
                return Err(AddFormulaError::FormulaExists { index });
            }
        }
        self.formulas.push(formula);
        Ok(())
    }
}

impl<V, S: Span> FormulaManager<S, Expression<V, S>> {
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> FormulaManager<S2, Expression<V, S2>> {
        FormulaManager {
            formulas: self.formulas.into_iter().map(|f| f.map_span(map)).collect(),
        }
    }
}

impl<S: Span, E> crate::private::Sealed for FormulaManager<S, E> {}
impl<Ctx, E: Displayable<Ctx>, S: Span> Displayable<Ctx> for FormulaManager<S, E> {
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result {
        for formula in &self.formulas {
            writeln!(f, "{}", formula.displayable(context))?;
        }
        if self.formulas.len() > 0 {
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum AddFormulaError {
    FormulaExists { index: usize },
}

/// A [`Formula`] using [`Identifier`] to refer to variables in expressions, instead of the default
/// of [`VariableReference`].
pub type FormulaNamedVars<S: Span> = Formula<S, Expression<Identifier<S>, S>>;
pub struct Formula<S: Span, E = Expression<VariableReference, S>> {
    pub name: Identifier<S>,
    pub condition: E,
    pub span: S,
}

impl<S: Span, E> Formula<S, E> {
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

impl<S: Span, V> Formula<S, Expression<V, S>> {
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> Formula<S2, Expression<V, S2>> {
        Formula {
            name: self.name.map_span(map),
            condition: self.condition.map_span(&map),
            span: map(self.span),
        }
    }
}

impl<S: Span, E> crate::private::Sealed for Formula<S, E> {}
impl<Ctx, S: Span, E: Displayable<Ctx>> Displayable<Ctx> for Formula<S, E> {
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result {
        write!(
            f,
            "formula {} = {};",
            self.name,
            self.condition.displayable(context)
        )
    }
}
