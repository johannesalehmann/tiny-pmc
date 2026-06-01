use crate::spans::{FullSpan, Span};
use crate::{Displayable, Expression, Identifier, VariableReference};
use std::fmt::Formatter;

/// A [`LabelManager`] using [`Identifier`] to refer to variables in expressions, instead of the
/// default of [`VariableReference`].
pub type LabelManagerNamedVars<S: Span = FullSpan> = LabelManager<S, Expression<Identifier<S>>>;

/// A collection of [`Label`]`s`.
///
/// Each label has a name and condition expression. A `LabelManager` collects labels and ensures
/// there are no duplicates.
///
/// # Example
///
/// Create a `LabelManager` using [`LabelManager::new()`] or [`LabelManager::with_labels()`]:
/// ```
/// # use prism_model::*;
/// let mut labels: LabelManagerNamedVars = LabelManager::new();
/// ```
///
/// Labels can then be added using [`LabelManager::add_label()`], which returns
/// [`AddLabelError`] if the label exists.
///
/// ```
/// # use prism_model::*;
/// # let mut labels: LabelManagerNamedVars = LabelManager::new();
/// let result_1
///     = labels.add_label(Label::new(Identifier::new("done").unwrap(), Expression::bool(true)));
/// assert!(result_1.is_ok());
///
/// let result_2
///     = labels.add_label(Label::new(Identifier::new("done").unwrap(), Expression::bool(false)));
/// assert!(matches!(result_2, Err(AddLabelError::LabelExists { index: 0 })));
/// ```
///
/// Labels can be looked up by index using [`LabelManager::get()`], by name string using
/// [`LabelManager::by_name()`] or the index of a named label can be retrieved using
/// [`LabelManager::index_of_name()`].
#[derive(PartialEq, Clone, Debug)]
pub struct LabelManager<S: Span = FullSpan, E = Expression<VariableReference, S>> {
    /// The collection of [`Label`]s held by this [`LabelManager`].
    ///
    /// Do not add [`Label`]`s` to this directly. Instead, use [`LabelManager::add_label()`],
    /// which ensures no duplicate names are added.
    pub labels: Vec<Label<S, E>>,
}

impl<S: Span, E> LabelManager<S, E> {
    /// Constructs an empty [`LabelManager`].
    ///
    /// To construct a non-empty [`LabelManager`], use [`LabelManager::with_labels()`] instead.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::LabelManager;
    /// let labels: LabelManager = LabelManager::new();
    /// assert_eq!(labels.labels.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self { labels: Vec::new() }
    }

    /// Constructs a [`LabelManager`] with the given set of labels.
    ///
    /// To construct an empty [`LabelManager`], use [`LabelManager::new()`] instead.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::*;
    /// let labels: Result<LabelManager, AddLabelError> = LabelManager::with_labels(vec![
    ///     Label::new(Identifier::new("l1").unwrap(), Expression::bool(true)),
    ///     Label::new(Identifier::new("l2").unwrap(), Expression::bool(false)),
    /// ]);
    /// assert!(labels.is_ok());
    /// assert_eq!(labels.unwrap().labels.len(), 2);
    /// ```
    ///
    /// If the set of labels contains duplicates, [`AddLabelError`] is returned. `index: 1`
    /// indicates that the first occurrence of the duplicate label was at index `1`.
    ///
    /// ```
    /// # use prism_model::*;
    /// let labels: Result<LabelManager, AddLabelError> = LabelManager::with_labels(vec![
    ///     Label::new(Identifier::new("ready").unwrap(), Expression::bool(true)),
    ///     Label::new(Identifier::new("done").unwrap(), Expression::bool(false)),
    ///     Label::new(Identifier::new("done").unwrap(), Expression::bool(true)),
    /// ]);
    /// assert!(matches!(labels.err(), Some(AddLabelError::LabelExists { index: 1 })));
    /// ```
    pub fn with_labels(mut labels: Vec<Label<S, E>>) -> Result<Self, AddLabelError> {
        let mut res = Self::new();

        for label in labels.drain(..) {
            res.add_label(label)?;
        }
        Ok(res)
    }

    /// Adds a label to the [`LabelManager`].
    ///
    /// If a label with this name already exists, returns [`AddLabelError`].
    ///
    /// # Example
    ///
    /// Adding the first label named `done` is ok:
    /// ```
    /// # use prism_model::*;
    /// let mut labels: LabelManager = LabelManager::new();
    ///
    /// let res = labels.add_label(
    ///     Label::new(Identifier::new("done").unwrap(), Expression::bool(true))
    /// );
    /// assert!(res.is_ok());
    /// ```
    ///
    /// Adding a second label of the same name produces an error. The error contains the index
    /// of the existing label with the same name. This can be used to provide diagnostics to the
    /// user.
    ///
    /// ```
    /// # use prism_model::*;
    /// # let mut labels: LabelManager = LabelManager::new();
    /// #
    /// # let res = labels.add_label(
    /// #     Label::new(Identifier::new("done").unwrap(), Expression::bool(true))
    /// # );
    /// # assert!(res.is_ok());
    /// #
    /// let res = labels.add_label(
    ///     Label::new(Identifier::new("done").unwrap(), Expression::bool(false))
    /// );
    /// assert_eq!(res, Err(AddLabelError::LabelExists { index: 0 }));
    /// ```
    pub fn add_label(&mut self, label: Label<S, E>) -> Result<(), AddLabelError> {
        for (index, other_label) in self.labels.iter().enumerate() {
            if other_label.name == label.name {
                return Err(AddLabelError::LabelExists { index });
            }
        }
        self.labels.push(label);
        Ok(())
    }

    /// Returns the label with the given `index`.
    ///
    /// If no label with this index exists, returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::*;
    /// let labels: LabelManager = LabelManager::with_labels(vec![
    ///     Label::new(Identifier::new("ready").unwrap(), Expression::bool(true)),
    ///     Label::new(Identifier::new("running").unwrap(), Expression::bool(true)),
    ///     Label::new(Identifier::new("done").unwrap(), Expression::bool(false)),
    /// ]).unwrap();
    ///
    /// assert!(labels.get(2).is_some());
    /// let identifier = &labels.get(2).unwrap().name;
    /// assert_eq!(identifier.name, "done");
    ///
    /// assert!(labels.get(3).is_none());
    /// ```
    pub fn get(&self, index: usize) -> Option<&Label<S, E>> {
        self.labels.get(index)
    }

    /// Returns the index of the label with the given `name`, or `None` if no such label exists.
    ///
    /// To retrieve the label itself by name, use [`LabelManager::by_name()`].
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::*;
    /// let labels: LabelManager = LabelManager::with_labels(vec![
    ///     Label::new(Identifier::new("ready").unwrap(), Expression::bool(true)),
    ///     Label::new(Identifier::new("done").unwrap(), Expression::bool(false)),
    /// ]).unwrap();
    ///
    /// assert_eq!(labels.index_of_name("done"), Some(1));
    /// assert_eq!(labels.index_of_name("unknown"), None);
    /// ```
    pub fn index_of_name(&self, name: &str) -> Option<usize> {
        for (i, label) in self.labels.iter().enumerate() {
            if label.name.name == name {
                return Some(i);
            }
        }
        None
    }

    /// Returns the label with the given `name`, or `None` if no such label exists.
    ///
    /// To retrieve only the index of a named label, use [`LabelManager::index_of_name()`].
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::*;
    /// let labels: LabelManager = LabelManager::with_labels(vec![
    ///     Label::new(Identifier::new("ready").unwrap(), Expression::bool(true)),
    ///     Label::new(Identifier::new("done").unwrap(), Expression::bool(false)),
    /// ]).unwrap();
    ///
    /// assert!(labels.by_name("done").is_some());
    /// assert_eq!(labels.by_name("done").unwrap().name.name, "done");
    ///
    /// assert!(labels.by_name("unknown").is_none());
    /// ```
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
    /// Maps the [`Span`] of every [`Label`] in this `LabelManager` according to mapping function
    /// `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
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

/// Error caused when trying to add a [`Label`] to a [`LabelManager`].
///
/// The only variant is [`LabelExists`](AddLabelError::LabelExists), which indicates that the
/// label manager already contains a label with the same name.
#[derive(Debug, Clone, PartialEq)]
pub enum AddLabelError {
    /// Indicates that a [`Label`] with the same [`name`](Label::name) already exists in the
    /// [`LabelManager`].
    LabelExists {
        /// The index of the first [`Label`] with the same name in the [`LabelManager`].
        ///
        /// When [`LabelManager::with_labels()`] is used, this corresponds to the first
        /// occurrence of the duplicate label in argument `labels`. (In this case, it is not
        /// possible to obtain the index of the second occurrence of the duplicate label. Call
        /// [`LabelManager::add_label()`] repeatedly if this is required.)
        index: usize,
    },
}

/// A [`Label`] using [`Identifier`] to refer to variables in expressions, instead of the default of
/// [`VariableReference`].
pub type LabelNamedVars<S: Span = FullSpan> = Label<S, Expression<Identifier<S>, S>>;

/// A label, consisting of a name and a condition expression.
///
/// Labels are used to identify sets of states in PRISM models. A label marks all states in which
/// its condition expression evaluates to `true`. They are usually stored in a [`LabelManager`]
/// (cf. [`Model::labels`](crate::Model::labels)).
///
/// For examples, see [`LabelManager`].
#[derive(PartialEq, Clone, Debug)]
pub struct Label<S: Span = FullSpan, E = Expression<VariableReference, S>> {
    /// The name of the label.
    pub name: Identifier<S>,

    /// The condition of the label.
    pub condition: E,

    /// The [`Span`] of the label.
    pub span: S,
}

impl<S: Span, E> Label<S, E> {
    /// Creates a new `Label` with given name and condition.
    ///
    /// The [`Span`] is empty. To construct a label with non-empty span, use
    /// [`Label::new_spanned()`].
    pub fn new(name: Identifier<S>, condition: E) -> Self {
        Self::new_spanned(name, condition, S::empty())
    }

    /// Creates a new `Label` with given name, condition and [`Span`].
    ///
    /// To construct a label with empty span, use [`Label::new()`].
    pub fn new_spanned(name: Identifier<S>, condition: E, span: S) -> Self {
        Self {
            name,
            condition,
            span,
        }
    }
}
impl<V, S: Span> Label<S, Expression<V, S>> {
    /// Maps every [`Span`] of this `Label` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to [`name`](Self::name), [`condition`](Self::condition),
    /// and [`span`](Self::span).
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
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
