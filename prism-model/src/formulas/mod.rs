mod formula_dependencies;

pub use formula_dependencies::*;
use std::fmt::Formatter;

use crate::spans::{FullSpan, Span};
use crate::{Displayable, Expression, Identifier, VariableReference};

/// A [`FormulaManager`] using [`Identifier`] to refer to variables in expressions, instead of the
/// default of [`VariableReference`].
pub type FormulaManagerNamedVars<S: Span = FullSpan> =
    FormulaManager<S, Expression<Identifier<S>, S>>;

/// A collection of [`Formula`]`s`.
///
/// Each formula has a name and value expression. A `FormulaManager` collects formulas and ensures
/// there are no duplicates.
///
/// # Example
///
/// Create a `FormulaManager` using [`FormulaManager::new()`] or
/// [`FormulaManager::with_formulas()`]:
/// ```
/// # use prism_model::*;
/// let mut formulas: FormulaManagerNamedVars = FormulaManager::new();
/// ```
///
/// Formulas can then be added using [`FormulaManager::add_formula()`], which returns
/// [`AddFormulaError`] if the formula exists.
///
/// ```
/// # use prism_model::*;
/// # let mut formulas: FormulaManagerNamedVars  = FormulaManager::new();
/// let result_1
///     = formulas.add_formula(Formula::new(Identifier::new("five").unwrap(), Expression::int(5)));
/// assert_eq!(result_1, Ok(()));
///
/// let result_2
///     = formulas.add_formula(Formula::new(Identifier::new("five").unwrap(), Expression::float(5.0)));
/// assert_eq!(result_2, Err(AddFormulaError::FormulaExists{ index: 0}));
/// ```
///
/// For an example using nested formulas, see [`Expression::substitute_formulas()`].
///
/// # Formulas in expressions
///
/// In expressions, formulas are modelled as [`Expression::VarOrConst(name)`](Expression::VarOrConst),
/// i.e., they are not distinguished from variables and constants. Before evaluating an expression
/// that contains a
/// formula, it is recommended to expand all formulas. For details, see
/// [`Expression::substitute_formulas()`].
///
/// To expand formulas in the entire model, use
/// [`Model::substitute_formulas()`](crate::Model::substitute_formulas()).
#[derive(PartialEq, Clone, Debug)]
pub struct FormulaManager<S: Span = FullSpan, E = Expression<VariableReference, S>> {
    // TODO: Make this private and instead provide suitable access methods; in particular, an
    //  iterator for `FormulaManager`
    /// The collection of [`Formula`]s by this [`FormulaManager`].
    ///
    /// Do not add [`Formula`]`s` to this directly. Instead, use [`FormulaManager::add_formula()`],
    /// which ensures no duplicate names are added.
    pub formulas: Vec<Formula<S, E>>,
}

impl<S: Span, E> FormulaManager<S, E> {
    /// Constructs an empty [`FormulaManager`].
    ///
    /// To construct a non-empty [`FormulaManager`], use [`FormulaManager::with_formulas()`]
    /// instead.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::FormulaManager;
    /// let formulas: FormulaManager = FormulaManager::new();
    /// assert_eq!(formulas.formulas.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            formulas: Vec::new(),
        }
    }

    /// Constructs a [`FormulaManager`] with the given set of formulas.
    ///
    /// To construct a non-empty [`FormulaManager`], use [`FormulaManager::with_formulas()`]
    /// instead.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::*;
    /// let formulas: Result<FormulaManager, AddFormulaError> = FormulaManager::with_formulas(vec![
    ///     Formula::new(Identifier::new("f1").unwrap(), Expression::int(5)),
    ///     Formula::new(Identifier::new("f2").unwrap(), Expression::bool(false)),
    /// ]);
    /// assert!(formulas.is_ok());
    /// assert_eq!(formulas.unwrap().formulas.len(), 2);
    /// ```
    ///
    /// If the set of formulas contains duplicates, [`AddFormulaError`] is returned. `index: 1`
    /// indicates that the first occurrence of the duplicate formula was at index `1`.
    ///
    /// ```
    /// # use prism_model::*;
    /// let formulas: Result<FormulaManager, AddFormulaError> = FormulaManager::with_formulas(vec![
    ///     Formula::new(Identifier::new("e").unwrap(), Expression::float(2.72)),
    ///     Formula::new(Identifier::new("circle_constant").unwrap(), Expression::float(3.14)),
    ///     Formula::new(Identifier::new("circle_constant").unwrap(), Expression::float(6.28)),
    /// ]);
    /// assert_eq!(formulas.err(), Some(AddFormulaError::FormulaExists { index: 1 }));
    /// ```
    pub fn with_formulas(mut formulas: Vec<Formula<S, E>>) -> Result<Self, AddFormulaError> {
        let mut res = Self::new();

        for formula in formulas.drain(..) {
            res.add_formula(formula)?;
        }
        Ok(res)
    }

    /// Returns the formula with the given `index`.
    ///
    /// If no formula with this index exists, returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::{Expression, Formula, FormulaManager, Identifier};
    /// let formulas: FormulaManager = FormulaManager::with_formulas(vec![
    ///     Formula::new(Identifier::new("five").unwrap(), Expression::int(5)),
    ///     Formula::new(Identifier::new("six").unwrap(), Expression::int(6)),
    ///     Formula::new(Identifier::new("seven").unwrap(), Expression::int(7)),
    /// ]).unwrap();
    ///
    /// assert!(formulas.get(2).is_some());
    /// let identifier = &formulas.get(2).unwrap().name;
    /// assert_eq!(identifier.name, "seven");
    ///
    /// assert!(formulas.get(3).is_none());
    /// ```
    pub fn get(&self, index: usize) -> Option<&Formula<S, E>> {
        self.formulas.get(index)
    }

    /// Adds a formula to the [`FormulaManager`].
    ///
    /// If a formula with this name already exists, returns [`AddFormulaError`].
    ///
    /// # Example
    ///
    /// Add the first formula named `circle_constant` is ok:
    /// ```
    /// # use prism_model::*;
    /// let mut formulas: FormulaManager = FormulaManager::new();
    ///
    /// let res = formulas.add_formula(
    ///     Formula::new(Identifier::new("circle_constant").unwrap(), Expression::float(3.14))
    /// );
    /// assert_eq!(res, Ok(()));
    /// ```
    ///
    /// Adding a second formula of the same name produces an error. The error contains the index
    /// of the existing formula with the same name. This can be used to provide diagnostics to the
    /// user.
    ///
    /// ```
    /// # use prism_model::*;
    /// # let mut formulas: FormulaManager = FormulaManager::new();
    /// #
    /// # let res = formulas.add_formula(
    /// #     Formula::new(Identifier::new("circle_constant").unwrap(), Expression::float(3.14))
    /// # );
    /// # assert_eq!(res, Ok(()));
    /// #
    /// let res = formulas.add_formula(
    ///     Formula::new(Identifier::new("circle_constant").unwrap(), Expression::float(6.28))
    /// );
    /// assert_eq!(res, Err(AddFormulaError::FormulaExists {index: 0}));
    /// ```
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
    /// Maps the [`Span`] of every [`Formula`] in this `FormulaManager` according to mapping
    /// function `map`.
    ///
    /// The new value is of type `S2`, which may be different from the original span type `S`. Usage
    /// is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
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

// TODO: Consider turning this into a simple struct `FormulaExists`. After all, it is unlikely that
//  further error kinds will be added to this error.
/// Error caused when trying to add a [`Formula`] to a [`FormulaManager`].
///
/// The only variant is [`FormulaExists`](AddFormulaError::FormulaExists), which indicates that the
/// formula manager already contains a formula with the same name.
#[derive(Debug, PartialEq, Eq)]
pub enum AddFormulaError {
    /// Indicates that a [`Formula`] with the same [`name`](Formula::name) already exists in the
    /// [`FormulaManager`].
    ///
    ///
    FormulaExists {
        /// The index of the first [`Formula`] with the same name in the [`FormulaManager`].
        ///
        /// When [`FormulaManager::with_formulas()`] is used, this corresponds to the first
        /// occurrence of the duplicate formula in argument `formulas`. (In this case, it is not
        /// possible to obtain the index of the second occurrence of the duplicate formula. Call
        /// [`FormulaManager::add_formula()`] repeatedly if this is required.)
        index: usize,
    },
}

/// A [`Formula`] using [`Identifier`] to refer to variables in expressions, instead of the default
/// of [`VariableReference`].
pub type FormulaNamedVars<S: Span> = Formula<S, Expression<Identifier<S>, S>>;

/// A formula, consisting of a name and a condition expression.
///
/// Formulas are used to encapsulate repeatedly-used expressions in PRISM models. They are usually
/// stored in a [`FormulaManager`] (cf. [`Model::formulas`](crate::Model::formulas)). Before
/// evaluating an expression, formulas should be replaced by calling
/// [`Expression::substitute_formulas`].
///
/// For examples, see [`FormulaManager`].
#[derive(PartialEq, Clone, Debug)]
pub struct Formula<S: Span, E = Expression<VariableReference, S>> {
    /// The name of the formula
    pub name: Identifier<S>,

    /// The condition of the formula.
    pub condition: E,

    /// The [`Span`] of the formula.
    pub span: S,
}

impl<S: Span, E> Formula<S, E> {
    /// Creates a new `Formula` with given name and condition.
    ///
    /// The [`Span`] is empty. To construct a formula with non-empty span, use
    /// [`Formula::new_spanned()`].
    pub fn new(name: Identifier<S>, condition: E) -> Self {
        Self::new_spanned(name, condition, S::empty())
    }

    /// Creates a new `Formula` with given name, condition and [`Span`].
    ///
    /// To construct a formula with empty span, use [`Formula::new()`].
    pub fn new_spanned(name: Identifier<S>, condition: E, span: S) -> Self {
        Self {
            name,
            condition,
            span,
        }
    }
}

impl<S: Span, V> Formula<S, Expression<V, S>> {
    /// Maps every [`Span`] of this `Formula` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to [`name`](Self::name), [`condition`](Self::condition) and
    /// [`span`](Self::span).
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
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
