use crate::expressions::UnknownVariableError;
use crate::module::RenameRules;
use crate::spans::{FullSpan, Span};
use crate::{Displayable, Expression, Identifier};
use std::fmt::Formatter;

/// A [`VariableManager`] using [`Identifier`] to refer to variables in expressions, instead of the
/// default of [`VariableReference`].
pub type VariableManagerNamedVars<S: Span = FullSpan> =
    VariableManager<S, Expression<Identifier<S>, S>>;

/// A collection of constants and variables.
///
/// Each model has one variable manager. The variable manager tracks all constants and variables of
/// the model, both global and local. In the rest of the model, variables can be referenced either
/// by their name ([`Identifier`]) or by reference ([`VariableReference`]).
///
/// # Example
///
/// Let `variables` be an empty variable manager:
/// ```
/// # use prism_model::{Identifier, VariableInfo, VariableManager, VariableRange, Expression};
/// let mut variables: VariableManager = VariableManager::new();
/// ```
///
/// Add global variable `x` with range `[-5..12]`:
///
/// ```
/// # use prism_model::{Identifier, VariableInfo, VariableManager, VariableRange, Expression};
/// # let mut variables: VariableManager = VariableManager::new();
/// #
/// let x_name = Identifier::new("x").unwrap();
/// let x_range = VariableRange::bounded_int(Expression::int(-5), Expression::int(12));
/// let x_ref = variables.add_variable(VariableInfo::global_var(x_name, x_range))
///     .expect("Failed to add variable");
/// ```
///
/// Given reference `x_ref`, we can access the [`VariableInfo`] of `x` using
/// [`VariableManager::get()`]:
///
/// ```
/// # use prism_model::{Identifier, VariableInfo, VariableManager, VariableRange, Expression};
/// # let mut variables: VariableManager = VariableManager::new();
/// #
/// # let x_name = Identifier::new("x").unwrap();
/// # let x_range = VariableRange::bounded_int(Expression::int(-5), Expression::int(12));
/// # let x_ref = variables.add_variable(VariableInfo::global_var(x_name, x_range))
/// #     .expect("Failed to add variable");
/// assert_eq!(variables.get(&x_ref).unwrap().is_constant, false);
/// ```
///
/// To add a local, boolean variable, we need the index of the module the variable is in. In
/// practice, [`ModuleManager::get_index_by_name`](crate::ModuleManager::get_index_by_name) is used
/// to obtain the index.
///
/// ```
/// # use prism_model::{Identifier, VariableInfo, VariableManager, VariableRange, Expression};
/// # let mut variables: VariableManager = VariableManager::new();
/// #
/// let y_name = Identifier::new("y").unwrap();
/// let y_range = VariableRange::bool();
/// let y_module = 7;
/// let y_ref = variables.add_variable(VariableInfo::local_var(y_name, y_range, y_module))
///     .expect("Failed to add variable");
/// ```
///
/// To obtain a variable reference given a name, use [`VariableManager::get_reference_by_str()`].
///
/// ```
/// # use prism_model::{Identifier, VariableInfo, VariableManager, VariableRange, Expression};
/// # let mut variables: VariableManager = VariableManager::new();
/// #
/// # let y_name = Identifier::new("y").unwrap();
/// # let y_range = VariableRange::bool();
/// # let y_module = 7;
/// # let y_ref = variables.add_variable(VariableInfo::local_var(y_name, y_range, y_module))
/// #     .expect("Failed to add variable");
/// assert_eq!(variables.get_reference_by_str("y"), Some(y_ref));
/// ```
#[derive(PartialEq, Clone, Debug)]
pub struct VariableManager<S: Span = FullSpan, E = Expression<VariableReference, S>> {
    /// The list of variables managed by this variable manager.
    ///
    /// Do not add variables directly to this. Instead, use [`VariableManager::add_variable()`].
    pub variables: Vec<VariableInfo<S, E>>,
}

impl<S: Span, E> VariableManager<S, E> {
    /// Constructs an empty variable manager.
    ///
    /// Use [`VariableManager::add_variable()`] to add variables, or
    /// [`VariableManager::with_variables()`] to construct a manager from a pre-built list.
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
        }
    }

    /// Constructs a variable manager from an existing list of variables.
    ///
    /// Returns [`VariableAddError::VariableExists`] if any two variables share the same name.
    ///
    /// ```
    /// # use prism_model::*;
    /// let variables = vec![
    ///     VariableInfo::global_var(Identifier::new("x").unwrap(), VariableRange::bool()),
    ///     VariableInfo::global_var(Identifier::new("y").unwrap(), VariableRange::bool()),
    /// ];
    /// let manager: VariableManager = VariableManager::with_variables(variables).unwrap();
    /// ```
    pub fn with_variables(variables: Vec<VariableInfo<S, E>>) -> Result<Self, VariableAddError> {
        let mut manager = Self::new();
        for variable in variables {
            manager.add_variable(variable)?;
        }
        Ok(manager)
    }

    /// Adds a variable to the variable manager.
    ///
    /// If a variable with the same name already exists (regardless of scope),
    /// [`VariableAddError::VariableExists`] is returned with information on the existing variable.
    ///
    /// See [`VariableManager`] for detailed examples.
    // TODO: This should verify that the variable info is a legal combination (i.e. scope matches
    //  range and type is legal for variable kind
    pub fn add_variable(
        &mut self,
        variable_info: VariableInfo<S, E>,
    ) -> Result<VariableReference, VariableAddError> {
        if let Some(existing_variable) = self.get_reference(&variable_info.name) {
            Err(VariableAddError::VariableExists {
                reference: existing_variable,
            })
        } else {
            let index = VariableReference::new(self.variables.len());
            self.variables.push(variable_info);
            Ok(index)
        }
    }

    /// Returns a reference to the variable with the given `name` or `None` if no such variable
    /// exists.
    ///
    /// [`Identifier`]`s` are equal if their name is equal – their spans do not need to match.
    ///
    /// If the name is only available as a string, use [`VariableManager::get_reference_by_str()`].
    pub fn get_reference(&self, name: &Identifier<S>) -> Option<VariableReference> {
        for (index, var) in self.variables.iter().enumerate() {
            if &var.name == name {
                return Some(VariableReference::new(index));
            }
        }
        None
    }

    /// Returns a reference to the variable with the given `name` or `None` if no such variable
    /// exists.
    ///
    /// If the name is available as [`Identifier`], use [`VariableManager::get_reference()`].
    pub fn get_reference_by_str(&self, name: &str) -> Option<VariableReference> {
        for (index, var) in self.variables.iter().enumerate() {
            if &var.name.name == name {
                return Some(VariableReference::new(index));
            }
        }
        None
    }

    /// Returns details on the variable given a variable reference.
    ///
    /// To obtain a variable reference, either store the result of
    /// [`VariableManager::add_variable()`] or look up a variable by name using
    /// [`VariableManager::get_reference()`] or [`VariableManager::get_reference_by_str()`].
    ///
    /// Given a variable name, [`VariableManager::get_by_name()`] and
    /// [`VariableManager::get_by_str`] return the corresponding variable details.
    pub fn get(&self, reference: &VariableReference) -> Option<&VariableInfo<S, E>> {
        self.variables.get(reference.index)
    }

    /// Returns details on the variable given a variable identifier.
    ///
    /// If the identifier is available as `&str`, use [`VariableManager::get_by_str()`] instead.
    pub fn get_by_name(&self, name: &Identifier<S>) -> Option<&VariableInfo<S, E>> {
        self.get(&self.get_reference(name)?)
    }

    /// Returns details on the variable given a variable identifier.
    ///
    /// If the identifier is available as [`Identifier`], use [`VariableManager::get_by_name()`]
    /// instead.
    pub fn get_by_str(&self, name: &str) -> Option<&VariableInfo<S, E>> {
        self.get(&self.get_reference_by_str(name)?)
    }
}

impl<S: Span, E> crate::private::Sealed for VariableManager<S, E> {}
impl<Ctx, E: Displayable<Ctx>, S: Span> Displayable<(VariablePrintingStyle, &Ctx)>
    for VariableManager<S, E>
{
    fn fmt_internal(
        &self,
        f: &mut Formatter<'_>,
        (printing_style, context): &(VariablePrintingStyle, &Ctx),
    ) -> std::fmt::Result {
        for variable in &self.variables {
            if !printing_style.accepts(variable) {
                continue;
            }
            if printing_style == &VariablePrintingStyle::Const {
                write!(
                    f,
                    "const {} {}",
                    variable.range.displayable(context),
                    variable.name
                )?;
                if let Some(initial) = &variable.initial_value {
                    write!(f, " = {}", initial.displayable(context))?;
                }
            } else {
                if printing_style == &VariablePrintingStyle::GlobalVar {
                    write!(f, "global ")?;
                } else {
                    write!(f, "    ")?;
                }
                write!(
                    f,
                    "{} : {}",
                    variable.name,
                    variable.range.displayable(context)
                )?;
                if let Some(initial) = &variable.initial_value {
                    write!(f, " init {}", initial.displayable(context))?;
                }
            }
            writeln!(f, ";")?;
        }
        if self.variables.len() > 0 {
            writeln!(f, "")?;
        }
        Ok(())
    }
}

impl<V, S: Span> VariableManager<S, Expression<V, S>> {
    /// Maps the [`Span`]s of every [`VariableInfo`] in this `VariableManager` according to mapping
    /// function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> VariableManager<S2, Expression<V, S2>> {
        VariableManager {
            variables: self
                .variables
                .into_iter()
                .map(|v| v.map_span(map))
                .collect(),
        }
    }
}
impl<S: Span> VariableManager<S, Expression<Identifier<S>, S>> {
    /// Copies the variable declarations with scope `old_module_index` to a new scope
    /// `new_module_index` and applies the renaming rules to their names and details.
    ///
    /// If a variable with scope `old_module_index` is not present in `rename_rules`,
    /// [`MissingVariableRenaming`] is returned.
    ///
    /// This function is used during renamed module expansion. See
    /// [`Model::expand_renamed_modules`](crate::Model::expand_renamed_modules) for details and a
    /// usage example.
    pub fn add_renamed(
        &mut self,
        old_module_index: usize,
        new_module_index: usize,
        rename_rules: &RenameRules<S>,
    ) -> Result<VariableManager<S, Expression<Identifier<S>, S>>, MissingVariableRenaming<S>> {
        let variables = Vec::with_capacity(self.variables.len());
        for i in 0..self.variables.len() {
            let variable = &self.variables[i];
            if variable.is_constant || variable.scope != Some(old_module_index) {
                continue;
            }
            match rename_rules.get_renaming(&variable.name) {
                None => {
                    return Err(MissingVariableRenaming {
                        variable_name: variable.name.clone(),
                        original_definition_span: variable.span.clone(),
                    });
                }
                Some(renaming) => {
                    let new_var = VariableInfo {
                        range: variable.range.renamed(rename_rules),
                        name: renaming,
                        initial_value: variable
                            .initial_value
                            .as_ref()
                            .map(|i| i.renamed(rename_rules)),
                        span: variable.span.clone(),
                        is_constant: false,
                        scope: Some(new_module_index),
                    };
                    self.variables.push(new_var)
                }
            }
        }

        Ok(VariableManager { variables })
    }
}

/// An error produced while trying to add a variable or constant to a [`VariableManager`].
pub enum VariableAddError {
    /// A variable or constant with the same name already exists (in any scope).
    VariableExists {
        /// Reference to the existing variable with the same name
        reference: VariableReference,
    },
}

/// An error produced by [`VariableManager::add_renamed()`], indicating that a variable in the
/// scope to be copied is not present in the renaming rules.
pub struct MissingVariableRenaming<S: Span> {
    /// The name of the variable that is missing in the renaming rules.
    pub variable_name: Identifier<S>,

    /// The [`Span`] of the entire definition of the original variable.
    pub original_definition_span: S,
}

impl std::fmt::Debug for VariableAddError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableAddError::VariableExists { reference } => {
                write!(
                    f,
                    "Variable already exists in this manager (index {})",
                    reference.index
                )
            }
        }
    }
}

/// A [`VariableInfo`] using [`Identifier`] to refer to variables in expressions, instead of the
/// default of [`VariableReference`].
pub type VariableInfoNamedVars<S> = VariableInfo<S, Expression<Identifier<S>, S>>;

/// Information about a (local or global) variable or constant, corresponding to a PRISM variable
/// declaration.
///
/// # Example
///
/// As a prerequisite for this example, we need a [`ModuleManager`](crate::ModuleManager) with
/// module `main`:
///
/// ```
/// # use prism_model::{Identifier, ModuleManager, Module};
/// let mut module_manager: ModuleManager = ModuleManager::with_modules(
///     vec![ Module::new(Identifier::new("main").unwrap()) ]
/// );
/// ```
///
/// The declaration of local variable `x: [3..12] init 5;` in module `main` corresponds to the following variable
/// info:
///
/// ```
/// # use prism_model::{Expression, Identifier, VariableInfo, VariableRange, ModuleManager, Module, FullSpan, Span};
/// # let mut module_manager: ModuleManager = ModuleManager::with_modules(
/// #     vec![ Module::new(Identifier::new("main").unwrap()) ]
/// # );
/// let main_index = module_manager.get_index_by_str("main").unwrap();
/// let x_name = Identifier::new("x").unwrap();
/// let x_range = VariableRange::bounded_int(Expression::int(3), Expression::int(12));
/// let info: VariableInfo = VariableInfo::local_var(x_name.clone(), x_range.clone(), main_index)
///     .initial_value(Expression::int(5));
///
/// assert_eq!(
///     info,
///     VariableInfo {
///         is_constant: false,
///         scope: Some(main_index),
///         range: x_range,
///         name: x_name,
///         initial_value: Some(Expression::int(5)),
///         span: FullSpan::empty(),
///     }
/// );
/// ```
///
/// To construct global variables, use [`VariableInfo::global_var()`]. For constants, use
/// [`VariableInfo::global_const()`].
#[derive(PartialEq, Clone, Debug)]
pub struct VariableInfo<S: Span = FullSpan, E = Expression<VariableReference, S>> {
    // TODO: Merge is_constant and scope into one enum to make local constants unrepresentable
    /// If `true`, the variable is a constant. Otherwise, it is a (global or local) variable.
    pub is_constant: bool,

    /// If `None`, the variable is global (and may be variable or constant). If this is
    /// `Some(idx)`, the variable belongs to the module with index `idx`.
    pub scope: Option<usize>,

    /// The domain of the variable, e.g. boolean, float or (bounded and unbounded) integer.
    pub range: VariableRange<S, E>,

    /// The name of the variable
    pub name: Identifier<S>,

    /// If this is `Some(expr)`, the variable has initial value `expr`, otherwise, the variable has
    /// no explicit initial value.
    ///
    /// `expr` must be evaluatable at this stage:
    /// * If `is_constant = false`, then `expr` must not refer to other (non-constant) variables
    /// * If `is_constant = true`, then `expr` must not refer to (non-constant) variables and
    ///   dependencies on constants' initial values must be acyclic.
    ///
    /// If no initial value is present, the variable may still have an implicit value:
    /// * If the model uses an init expression, then all values satisfying the init expression are
    ///   initial values.
    /// * Otherwise, initial values are determined as follows:
    ///     * bounded integers: the minimal value of the bounds
    ///     * booleans: false
    ///     * floats and unbounded integers: these must have an initial value
    pub initial_value: Option<E>,

    /// The [`Span`] of the variable declaration
    pub span: S,
}

impl<S: Span, E> VariableInfo<S, E> {
    /// Creates a global variable with given name and range, no initial value and empty [`Span`].
    ///
    /// To add an initial value, use [`VariableInfo::initial_value()`].
    ///
    /// To use a custom span, use [`VariableInfo::global_var_spanned()`].
    pub fn global_var(name: Identifier<S>, range: VariableRange<S, E>) -> Self {
        Self::global_var_spanned(name, range, S::empty())
    }

    /// Creates a global variable with given name, range and [`Span`] and no initial value.
    ///
    /// To add an initial value, use [`VariableInfo::initial_value()`].
    ///
    /// To use an empty span, use [`VariableInfo::global_var()`].
    pub fn global_var_spanned(name: Identifier<S>, range: VariableRange<S, E>, span: S) -> Self {
        Self::new_spanned(name, range, false, None, span)
    }

    /// Creates a global constant with given name and range, no initial value and empty [`Span`].
    ///
    /// To add an initial value, use [`VariableInfo::initial_value()`].
    ///
    /// To use a custom span, use [`VariableInfo::global_const_spanned()`].
    pub fn global_const(name: Identifier<S>, range: VariableRange<S, E>) -> Self {
        Self::global_const_spanned(name, range, S::empty())
    }

    /// Creates a global constant with given name, range and [`Span`] and no initial value.
    ///
    /// To add an initial value, use [`VariableInfo::initial_value()`].
    ///
    /// To use an empty span, use [`VariableInfo::global_const()`].
    pub fn global_const_spanned(name: Identifier<S>, range: VariableRange<S, E>, span: S) -> Self {
        Self::new_spanned(name, range, true, None, span)
    }

    /// Creates a local variable in module `module` with given name and range, no initial value and
    /// empty [`Span`].
    ///
    /// To add an initial value, use [`VariableInfo::initial_value()`].
    ///
    /// To use a custom span, use [`VariableInfo::local_var_spanned()`].
    pub fn local_var(name: Identifier<S>, range: VariableRange<S, E>, module: usize) -> Self {
        Self::local_var_spanned(name, range, module, S::empty())
    }

    /// Creates a local variable in module `module` with given name, range and [`Span`] and no
    /// initial value.
    ///
    /// To add an initial value, use [`VariableInfo::initial_value()`].
    ///
    /// To use an empty span, use [`VariableInfo::local_var()`].
    pub fn local_var_spanned(
        name: Identifier<S>,
        range: VariableRange<S, E>,
        module: usize,
        span: S,
    ) -> Self {
        Self::new_spanned(name, range, false, Some(module), span)
    }

    /// Creates a variable with the given parameters and no initial value.
    ///
    /// Refer to [`VariableInfo`] for documentation of the parameters.
    ///
    /// There are also specialised constructor ([`VariableInfo::global_var()`],
    /// [`VariableInfo::global_const()`] and [`VariableInfo::local_var()`] for the different
    /// variable kinds.
    ///
    /// To use a custom [`Span`], use [`VariableInfo::new_spanned()`].
    ///
    /// To add an initial value, use [`VariableInfo::with_initial_value()`] or
    /// [`VariableInfo::initial_value()`].
    pub fn new(
        name: Identifier<S>,
        range: VariableRange<S, E>,
        is_constant: bool,
        scope: Option<usize>,
    ) -> Self {
        Self::new_spanned(name, range, is_constant, scope, S::empty())
    }

    /// Creates a variable with the given parameters and no initial value.
    ///
    /// Refer to [`VariableInfo`] for documentation of the parameters.
    ///
    /// There are also specialised constructor ([`VariableInfo::global_var_spanned()`],
    /// [`VariableInfo::global_const_spanned()`] and [`VariableInfo::local_var_spanned()`] for the
    /// different variable kinds.
    ///
    /// To use an empty [`Span`], use [`VariableInfo::new()`].
    ///
    /// To add an initial value, use [`VariableInfo::with_initial_value_spanned()`] or
    /// [`VariableInfo::initial_value()`].
    pub fn new_spanned(
        name: Identifier<S>,
        range: VariableRange<S, E>,
        is_constant: bool,
        scope: Option<usize>,
        span: S,
    ) -> Self {
        Self {
            name,
            range,
            initial_value: None,
            span,
            is_constant,
            scope,
        }
    }

    /// Creates a variable with the given parameters and initial value.
    ///
    /// Refer to [`VariableInfo`] for documentation of the parameters.
    ///
    /// There are also specialised constructor ([`VariableInfo::global_var()`],
    /// [`VariableInfo::global_const()`] and [`VariableInfo::local_var()`] for the different
    /// variable kinds. Use [`VariableInfo::initial_value()`] to add an initial value to these.
    ///
    /// To use a custom [`Span`], use [`VariableInfo::with_initial_value_spanned()`].
    pub fn with_initial_value(
        name: Identifier<S>,
        range: VariableRange<S, E>,
        is_constant: bool,
        scope: Option<usize>,
        initial_value: E,
    ) -> Self {
        Self::with_initial_value_spanned(name, range, is_constant, scope, initial_value, S::empty())
    }

    /// Creates a variable with the given parameters and initial value.
    ///
    /// Refer to [`VariableInfo`] for documentation of the parameters.
    ///
    /// There are also specialised constructor ([`VariableInfo::global_var()`],
    /// [`VariableInfo::global_const()`] and [`VariableInfo::local_var()`] for the different
    /// variable kinds. Use [`VariableInfo::initial_value()`] to add an initial value to these.
    ///
    /// To use an empty [`Span`], use [`VariableInfo::with_initial_value()`].
    pub fn with_initial_value_spanned(
        name: Identifier<S>,
        range: VariableRange<S, E>,
        is_constant: bool,
        scope: Option<usize>,
        initial_value: E,
        span: S,
    ) -> Self {
        Self {
            name,
            range,
            initial_value: Some(initial_value),
            span,
            is_constant,
            scope,
        }
    }

    /// Creates a variable with the given parameters.
    ///
    /// Refer to [`VariableInfo`] for documentation of the parameters.
    pub fn with_optional_initial_value(
        name: Identifier<S>,
        range: VariableRange<S, E>,
        is_constant: bool,
        scope: Option<usize>,
        initial_value: Option<E>,
        span: S,
    ) -> Self {
        Self {
            name,
            range,
            initial_value,
            span,
            is_constant,
            scope,
        }
    }

    /// Adds an initial value to an existing variable.
    #[must_use]
    pub fn initial_value(mut self, initial_value: E) -> Self {
        self.initial_value = Some(initial_value);
        self
    }
}
impl<V, S: Span> VariableInfo<S, Expression<V, S>> {
    /// Maps every [`Span`] of this variable according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to the [`range`](Self::range), [`name`](Self::name),
    /// [`initial_value`](Self::initial_value) and [`span`](Self::span).
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> VariableInfo<S2, Expression<V, S2>> {
        VariableInfo {
            is_constant: self.is_constant,
            scope: self.scope,
            range: self.range.map_span(map),
            name: self.name.map_span(map),
            initial_value: self.initial_value.map(|i| i.map_span(map)),
            span: map(self.span),
        }
    }
}

/// A [`VariableRange`] using [`Identifier`] to refer to variables in expressions, instead of the
/// default of [`VariableReference`].
pub type VariableRangeNamedVars<S: Span = FullSpan> =
    VariableRange<S, Expression<Identifier<S>, S>>;

/// The domain of a variable. Corresponds to a variable's type, with additional information to
/// support bounded integers.
///
/// A variable range can be a [bounded int](VariableRange::BoundedInt),
/// [unbounded int](VariableRange::UnboundedInt), a [boolean](VariableRange::Boolean) or a
/// [float](VariableRange::Float).
///
/// Bounded integers are only legal for variables. Floats are only legal for constants. Use
/// [`VariableRange::is_legal_for_variable()`] and [`VariableRange::is_legal_for_constant()`] to
/// check whether types are suitable.
///
/// [`VariableRange`] stores a [`Span`] that is accessible using [`VariableRange::span()`]
#[derive(Debug, PartialEq, Clone)]
pub enum VariableRange<S: Span = FullSpan, E = Expression<VariableReference, S>> {
    /// A bounded integer, corresponding to PRISM syntax `x: [min..max];`
    ///
    /// Both `min` and `max` are inclusive: `x: [-2, 1]` can take values `-2`, `-1`, `0` and `1`.
    ///
    /// Use [`VariableRange::bounded_int()`] and [`VariableRange::bounded_int_spanned()`] to
    /// construct this variant.
    ///
    /// # Example
    ///
    /// `min` and `max` can be integers:
    ///
    /// ```
    /// # use prism_model::{Expression, VariableRange};
    /// let r: VariableRange = VariableRange::bounded_int(Expression::int(-3), Expression::int(5));
    /// ```
    ///
    /// You can also use complex expressions for `min` and `max`; incorporating constants and
    /// mathematical operations
    ///
    /// ```
    /// # use prism_model::{Expression, Identifier, VariableRange, VariableRangeNamedVars};
    /// let r: VariableRangeNamedVars = VariableRange::bounded_int(
    ///     Expression::int(0),
    ///     Expression::var_or_const(Identifier::new("N").unwrap()).plus(Expression::int(1))
    /// );
    /// ```
    BoundedInt {
        /// The minimal value of this bounded integer variable. The value is inclusive.
        min: E,

        /// The maximal value of this bounded integer variable. The value is inclusive.
        max: E,

        /// The span of the bounded integer definition.
        ///
        /// Here, `^` denotes characters covered by the span:
        ///
        /// ```prism
        /// x: [-3..N*2+1] init 0;
        ///    ^^^^^^^^^^^
        /// ```
        span: S,
    },

    /// An unbounded integer, internally modelled as a 64-bit signed integer.
    ///
    /// Use [`VariableRange::unbounded_int()`] and [`VariableRange::unbounded_int_spanned()`] to
    /// construct this variant.
    UnboundedInt {
        /// The span of the unbounded integer type.
        ///
        /// Here, `^` denotes characters covered by the span:
        ///
        /// ```prism
        /// y: int init -5;
        ///    ^^^
        /// ```
        span: S,
    },

    /// A boolean variable
    ///
    /// Use [`VariableRange::bool()`] and [`VariableRange::bool_spanned()`] to construct this
    /// variant.
    Boolean {
        /// The span of the boolean type.
        ///
        /// Here, `^` denotes characters covered by the span:
        ///
        /// ```prism
        /// z: bool init true;
        ///    ^^^^
        /// ```
        span: S,
    },

    /// A floating-point number
    ///
    /// Use [`VariableRange::float()`] and [`VariableRange::float_spanned()`] to construct this
    /// variant.
    Float {
        /// The span of the float type.
        ///
        /// Here, `^` denotes characters covered by the span:
        ///
        /// ```prism
        /// x: float init 2/7;
        ///    ^^^^^
        /// ```
        span: S,
    },
}

impl<S: Span, E> VariableRange<S, E> {
    /// Constructs a bounded integer type with given minimal and maximal value (both inclusive) and
    /// empty span.
    ///
    /// See [`VariableRange::BoundedInt`] for details.
    ///
    /// To use a custom span, use [`VariableRange::bounded_int_spanned()`].
    pub fn bounded_int(min: E, max: E) -> Self {
        Self::bounded_int_spanned(min, max, S::empty())
    }

    /// Constructs a bounded integer with given minimal and maximal value (both inclusive) and given
    /// span.
    ///
    /// See [`VariableRange::BoundedInt`] for details.
    ///
    /// To use an empty span, use [`VariableRange::bounded_int()`].
    pub fn bounded_int_spanned(min: E, max: E, span: S) -> Self {
        Self::BoundedInt { min, max, span }
    }

    /// Constructs an unbounded integer type with empty span.
    ///
    /// See [`VariableRange::UnboundedInt`] for details.
    ///
    /// To use a custom span, use [`VariableRange::unbounded_int_spanned()`].
    pub fn unbounded_int() -> Self {
        Self::unbounded_int_spanned(S::empty())
    }

    /// Constructs an unbounded integer type with given span.
    ///
    /// See [`VariableRange::UnboundedInt`] for details.
    ///
    /// To use an empty span, use [`VariableRange::unbounded_int()`].
    pub fn unbounded_int_spanned(span: S) -> Self {
        Self::UnboundedInt { span }
    }

    /// Constructs a boolean type with empty span.
    ///
    /// See [`VariableRange::Boolean`] for details.
    ///
    /// To use a custom span, use [`VariableRange::bool_spanned()`].
    pub fn bool() -> Self {
        Self::bool_spanned(S::empty())
    }

    /// Constructs a boolean type with given span.
    ///
    /// See [`VariableRange::Boolean`] for details.
    ///
    /// To use an empty span, use [`VariableRange::bool()`].
    pub fn bool_spanned(span: S) -> Self {
        Self::Boolean { span }
    }

    /// Constructs a floating-point type with empty span.
    ///
    /// See [`VariableRange::Float`] for details.
    ///
    /// To use a custom span, use [`VariableRange::float_spanned()`].
    pub fn float() -> Self {
        Self::float_spanned(S::empty())
    }

    /// Constructs a floating-point type with given span.
    ///
    /// See [`VariableRange::Float`] for details.
    ///
    /// To use an empty span, use [`VariableRange::float()`].
    pub fn float_spanned(span: S) -> Self {
        Self::Float { span }
    }

    /// Returns `true` if the type is legal for constants.
    ///
    /// Bounded integers are not legal for constants. All other types are legal for constants.
    ///
    /// Use [`VariableRange::is_legal_for_variable()`] to check whether a type is legal for
    /// variables.
    pub fn is_legal_for_constant(&self) -> bool {
        match self {
            VariableRange::BoundedInt { .. } => false,
            VariableRange::UnboundedInt { .. } => true,
            VariableRange::Boolean { .. } => true,
            VariableRange::Float { .. } => true,
        }
    }

    /// Returns `true` if the type is legal for (non-constant) variables.
    ///
    /// Floats are not legal for variables. All other types are legal for variables.
    ///
    /// Use [`VariableRange::is_legal_for_constant()`] to check whether a type is legal for
    /// constants.
    pub fn is_legal_for_variable(&self) -> bool {
        match self {
            VariableRange::BoundedInt { .. } => true,
            VariableRange::UnboundedInt { .. } => true,
            VariableRange::Boolean { .. } => true,
            VariableRange::Float { .. } => false,
        }
    }

    /// Returns a string description of the enum variant, e.g. `bounded int`, `bool` or `float`.
    ///
    /// This does not include the span or bounds of the bounded integer. Use
    /// `format!("{}", (`[`VariableRange::displayable()`]`)` to format the variable range
    /// like PRISM, including bounds (e.g. `[-3..5]`, `bool` or `float`).
    pub fn name(&self) -> &'static str {
        match self {
            VariableRange::BoundedInt { .. } => "bounded int",
            VariableRange::UnboundedInt { .. } => "int",
            VariableRange::Boolean { .. } => "bool",
            VariableRange::Float { .. } => "float",
        }
    }

    /// Returns the [`Span`] of the variable range.
    pub fn span(&self) -> &S {
        match self {
            VariableRange::BoundedInt { span, .. } => span,
            VariableRange::UnboundedInt { span } => span,
            VariableRange::Boolean { span } => span,
            VariableRange::Float { span } => span,
        }
    }
}

impl<V, S: Span> VariableRange<S, Expression<V, S>> {
    /// Maps every [`Span`] of this `VariableRange` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// For every variant of this enum, `map` is applied to `span`, and for
    /// [`VariableRange::BoundedInt`], it is also applied to `min` and `max`.
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> VariableRange<S2, Expression<V, S2>> {
        match self {
            VariableRange::BoundedInt { min, max, span } => VariableRange::BoundedInt {
                min: min.map_span(map),
                max: max.map_span(map),
                span: map(span),
            },
            VariableRange::UnboundedInt { span } => VariableRange::UnboundedInt { span: map(span) },
            VariableRange::Boolean { span } => VariableRange::Boolean { span: map(span) },
            VariableRange::Float { span } => VariableRange::Float { span: map(span) },
        }
    }
}

impl<S: Span, E> crate::private::Sealed for VariableRange<S, E> {}
impl<Ctx, E: Displayable<Ctx>, S: Span> Displayable<Ctx> for VariableRange<S, E> {
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result {
        match self {
            VariableRange::BoundedInt { min, max, .. } => {
                write!(
                    f,
                    "[{}..{}]",
                    min.displayable(context),
                    max.displayable(context)
                )
            }
            VariableRange::UnboundedInt { .. } => {
                write!(f, "int")
            }
            VariableRange::Boolean { .. } => {
                write!(f, "bool")
            }
            VariableRange::Float { .. } => {
                write!(f, "double")
            }
        }
    }
}
impl<S: Span> VariableRange<S, Expression<Identifier<S>, S>> {
    /// Applies the renaming rules to the expressions in the variable range.
    ///
    /// This only changes `min` and `max` of [`VariableRange::BoundedInt`] variants and returns a
    /// clone of the remaining variants.
    pub fn renamed(&self, rename_rules: &RenameRules<S>) -> Self {
        match self {
            VariableRange::BoundedInt { min, max, span } => VariableRange::BoundedInt {
                min: min.renamed(rename_rules),
                max: max.renamed(rename_rules),
                span: span.clone(),
            },
            VariableRange::UnboundedInt { span } => {
                VariableRange::UnboundedInt { span: span.clone() }
            }
            VariableRange::Boolean { span } => VariableRange::Boolean { span: span.clone() },
            VariableRange::Float { span } => VariableRange::Float { span: span.clone() },
        }
    }

    /// Replaces variables represented by [`Identifier`] by [`VariableReference`]`s`. See
    /// [`Expression::replace_identifiers_by_variable_indices()`] for details.
    pub fn replace_identifiers_by_variable_indices(
        &self,
        variable_manager: &VariableManager<S, Expression<Identifier<S>, S>>,
    ) -> Result<VariableRange<S, Expression<VariableReference, S>>, Vec<UnknownVariableError<S>>>
    {
        match self {
            VariableRange::BoundedInt { min, max, span } => {
                let mut errors = Vec::new();
                let min = min
                    .clone()
                    .replace_identifiers_by_variable_indices(variable_manager);
                let max = max
                    .clone()
                    .replace_identifiers_by_variable_indices(variable_manager);
                if let Err(err) = &min {
                    errors.extend_from_slice(&err[..]);
                }
                if let Err(err) = &max {
                    errors.extend_from_slice(&err[..]);
                }
                if let (Ok(min), Ok(max)) = (min, max) {
                    Ok(VariableRange::BoundedInt {
                        min,
                        max,
                        span: span.clone(),
                    })
                } else {
                    Err(errors)
                }
            }
            VariableRange::UnboundedInt { span } => {
                Ok(VariableRange::UnboundedInt { span: span.clone() })
            }
            VariableRange::Boolean { span } => Ok(VariableRange::Boolean { span: span.clone() }),
            VariableRange::Float { span } => Ok(VariableRange::Float { span: span.clone() }),
        }
    }
}

/// A reference to a variable. Internally, this uses an index that corresponds to a [`VariableInfo`]
/// in a [`VariableManager`].
///
/// # Obtaining a variable reference:
///
/// [`VariableManager::add_variable()`] returns a reference if successful:
///
/// ```
/// # use prism_model::*;
/// let mut manager: VariableManager = VariableManager::new();
/// let info = VariableInfo::global_var(Identifier::new("x").unwrap(), VariableRange::bool());
/// let var: VariableReference = manager.add_variable(info).expect("Error adding variable");
/// ```
///
/// To get a reference given a variable name, use [`VariableManager::get_reference()`] or
/// [`VariableManager::get_reference_by_str()`]:
///
/// ```
/// # use prism_model::*;
/// # let mut manager: VariableManager = VariableManager::new();
/// # let info = VariableInfo::global_var(Identifier::new("x").unwrap(), VariableRange::bool());
/// # manager.add_variable(info).expect("Error adding variable");
/// let var: VariableReference = manager.get_reference_by_str("x").unwrap();
/// ```
///
/// A model that uses [`Identifier`]`s` to represent variables (i.e. `V = Identifier`) can be
/// converted into one using `VariableReferences` by calling
/// [`Model::replace_identifiers_by_variable_indices`](crate::Model::replace_identifiers_by_variable_indices).
///
/// ```
/// # use prism_model::*;
/// let mut model: ModelNamedVars = Model::new(ModelType::mdp());
/// // Add constant N:
/// let n_ref = model.variable_manager.add_variable(
///     VariableInfo::global_const(Identifier::new("N").unwrap(), VariableRange::unbounded_int()),
/// ).unwrap();
///
/// // Add global variable x. Its initial value refers to N by name.
/// let x_ref = model.variable_manager.add_variable(
///     VariableInfo::global_var(Identifier::new("x").unwrap(), VariableRange::unbounded_int())
///         .initial_value(Expression::var_or_const(Identifier::new("N").unwrap()))
/// );
///
/// let model: Model = model.replace_identifiers_by_variable_indices().expect("Unknown variable!");
///
/// // Now the initial value of x refers to N by reference:///
/// assert_eq!(
///     model.variable_manager.get_by_str("x").unwrap().initial_value,
///     Some(Expression::var_or_const(n_ref))
/// )
/// ```
///
/// # Using a variable reference
///
/// To access variable information such as name using a reference, use [`VariableManager::get()`].
///
/// ```
/// # use prism_model::*;
/// let mut manager: VariableManager = VariableManager::new();
/// let x_ref = manager.add_variable(
///     VariableInfo::global_var(Identifier::new("x").unwrap(), VariableRange::bool())
/// ).unwrap();
///
/// let info = manager.get(&x_ref).unwrap();
/// assert_eq!(info.name, Identifier::new("x").unwrap());
/// ```
///
/// Use the reference to construct expression `x + 5`:
///
/// ```
/// # use prism_model::*;
/// # let mut manager: VariableManager = VariableManager::new();
/// # let x_ref = manager.add_variable(
/// #     VariableInfo::global_var(Identifier::new("x").unwrap(), VariableRange::bool()),
/// # ).unwrap();
/// let expr: Expression = Expression::var_or_const(x_ref).plus(Expression::int(5));
/// ```
#[derive(PartialEq, Clone, Copy)]
pub struct VariableReference {
    /// The index of the variable in [`VariableManager::variables`].
    ///
    /// Use with caution -- usually, there are suitable wrapper functions, such as
    /// [`VariableManager::get()`].
    pub index: usize,
}

impl VariableReference {
    /// Constructs a variable reference with the given index.
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

impl std::fmt::Debug for VariableReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "var_{}", self.index)
    }
}

impl crate::private::Sealed for VariableReference {}
impl<S: Span> Displayable<VariableManager<S, Expression<VariableReference, S>>>
    for VariableReference
{
    fn fmt_internal(
        &self,
        f: &mut Formatter<'_>,
        context: &VariableManager<S, Expression<VariableReference, S>>,
    ) -> std::fmt::Result {
        let variable = context.get(&self).unwrap();
        write!(f, "{}", variable.name)
    }
}

/// Determines how a variable is printed when producing PRISM code (because constants, global
/// variables and local variables use different syntax).
///
/// This is used by [`VariableManager::displayable()`]. It determines which variables are printed
/// (e.g. `VariableManager::displayable(VariablePrintingStyle::Const)` only prints constants). It
/// also determines how the variables are formatted.
#[derive(PartialEq, Copy, Clone)]
pub enum VariablePrintingStyle {
    /// Prints the constants of a model, formatted as `const float pi = 3.1415;`.
    Const,

    /// Prints the global variables of a model, formatted as `global n: [0..12];`
    GlobalVar,

    /// Prints the local variables of the module with index `module_index`, formatted as
    /// `n: bool init true;`
    LocalVar {
        /// The index of the module whose variables should be printed.
        ///
        /// Use e.g. [`ModuleManager::get_index_by_str()`](crate::ModuleManager::get_index_by_str())
        /// to obtain the module index.
        module_index: usize,
    },
}

impl VariablePrintingStyle {
    fn accepts<S: Span, E>(&self, variable: &VariableInfo<S, E>) -> bool {
        match self {
            VariablePrintingStyle::Const => variable.is_constant,
            VariablePrintingStyle::GlobalVar => !variable.is_constant && variable.scope.is_none(),
            VariablePrintingStyle::LocalVar { module_index } => {
                !variable.is_constant && variable.scope == Some(*module_index)
            }
        }
    }
}
