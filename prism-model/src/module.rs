use crate::spans::{FullSpan, Span};
use crate::{
    Command, Displayable, Expression, Identifier, VariableManager, VariablePrintingStyle,
    VariableReference,
};
use std::fmt::{Display, Formatter};

/// A [`ModuleManager`] using [`Identifier`] to refer to variables in expressions, instead of the
/// default of [`VariableReference`].
pub type ModuleManagerNamedVars<S: Span = FullSpan, A = Identifier<S>> =
    ModuleManager<Identifier<S>, S, Expression<Identifier<S>, S>, A>;

/// A collection of modules.
///
/// # Example
///
// TODO: Better example! (Perhaps at `_mut` variants to the get functions of module manager?
/// ```
/// # use prism_model::{ModuleManager, Module, Identifier};
/// let mut module_manager: ModuleManager = ModuleManager::new();
/// let name = Identifier::new("module_1").unwrap();
/// module_manager.add(Module::new(name.clone())).unwrap();
/// assert!(module_manager.get_by_name(&name).unwrap().commands.is_empty());
/// ```
///
/// # Renamed modules
///
/// Renamed modules are modelled by [`RenamedModule`] and stored separately in
/// [`Model::renamed_modules`](crate::Model::renamed_modules)
#[derive(PartialEq, Clone, Debug)]
pub struct ModuleManager<
    V = VariableReference,
    S: Span = FullSpan,
    E = Expression<V, S>,
    A = Identifier<S>,
> {
    /// The list of modules stored in this [`ModuleManager`].
    ///
    /// Do not add modules directly to this `Vec`. Instead, call [`ModuleManager::add()`], which
    /// ensures there are no duplicates.
    pub modules: Vec<Module<V, S, E, A>>,
}

impl<V, S: Span, E, A> ModuleManager<V, S, E, A> {
    /// Creates an empty `ModuleManager`.
    ///
    /// To construct a `ModuleManager` from a list of modules, use [`ModuleManager::with_modules()`].
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }
    /// Creates a `ModuleManager` with the given set of modules.
    ///
    /// To construct an empty `ModuleManager`, use [`ModuleManager::new()`] and add modules with
    /// [`ModuleManager::add()`].
    // TODO: Check for duplicates
    pub fn with_modules(modules: Vec<Module<V, S, E, A>>) -> Self {
        Self { modules }
    }

    /// Returns the module with the given index or `None`, if the index is out of bounds.
    ///
    /// To get a module by name, use [`ModuleManager::get_by_name()`]. To get a module's index given
    /// its name, use [`ModuleManager::get_index_by_name()`].
    pub fn get(&self, index: usize) -> Option<&Module<V, S, E, A>> {
        self.modules.get(index)
    }

    /// Returns the index of the module with the given name or `None`, if no module with the given
    /// name exists.
    ///
    /// If the name is available as a `&str`, use [`Self::get_index_by_str()`].
    ///
    /// To get a reference to a module given its name, consider using
    /// [`ModuleManager::get_by_name()`] instead.
    pub fn get_index_by_name(&self, name: &Identifier<S>) -> Option<usize> {
        self.get_index_by_str(&name.name)
    }

    /// Returns the index of the module with the given name or `None`, if no module with the given
    /// name exists.
    ///
    /// If the name is available as an [`Identifier`], use [`Self::get_index_by_name()`].
    ///
    /// To get a reference to a module given its name, consider using
    /// [`ModuleManager::get_by_str()`] instead.
    pub fn get_index_by_str(&self, name: &str) -> Option<usize> {
        self.modules
            .iter()
            .enumerate()
            .find(|(_, m)| &m.name.name == name)
            .map(|(i, _)| i)
    }

    /// Returns the module with the given name or `None`, if no module with the given name exists.
    ///
    /// If the name is available as an `&str`, use [`Self::get_by_str()`].
    pub fn get_by_name(&self, name: &Identifier<S>) -> Option<&Module<V, S, E, A>> {
        self.get_by_str(&name.name)
    }

    /// Returns the module with the given name or `None`, if no module with the given name exists.
    ///
    /// If the name is available as an [`Identifier`], use [`Self::get_by_name()`].
    pub fn get_by_str(&self, name: &str) -> Option<&Module<V, S, E, A>> {
        self.modules.iter().find(|m| &m.name.name == name)
    }

    /// Adds a new module to the module manager and returns its index.
    ///
    /// If a module with the same name already exists, [`AddModuleError::ModuleExists`] is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::{AddModuleError, Identifier, Module, ModuleManager};
    /// let mut module_manager: ModuleManager = ModuleManager::new();
    ///
    /// assert_eq!(
    ///     module_manager.add(Module::new(Identifier::new("module_1").unwrap())),
    ///     Ok(0)
    /// );
    ///
    /// assert_eq!(
    ///     module_manager.add(Module::new(Identifier::new("module_2").unwrap())),
    ///     Ok(1)
    /// );
    ///
    /// assert_eq!(
    ///     module_manager.add(Module::new(Identifier::new("module_2").unwrap())),
    ///     Err(AddModuleError::ModuleExists {index: 1})
    /// );
    /// ```
    pub fn add(&mut self, module: Module<V, S, E, A>) -> Result<usize, AddModuleError> {
        for (index, other_module) in self.modules.iter().enumerate() {
            if other_module.name == module.name {
                return Err(AddModuleError::ModuleExists { index });
            }
        }
        let index = self.modules.len();
        self.modules.push(module);
        Ok(index)
    }
}
impl<V, S: Span, A> ModuleManager<V, S, Expression<V, S>, A> {
    /// Maps the [`Span`]s of every [`Module`] in this `ModuleManager` according to mapping function
    /// `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> ModuleManager<V, S2, Expression<V, S2>, A> {
        let mut module_manager = ModuleManager {
            modules: Vec::with_capacity(self.modules.len()),
        };
        for module in self.modules {
            module_manager.modules.push(module.map_span(map))
        }

        module_manager
    }
}

/// An error caused while adding a [`Module`] to a [`ModuleManager`].
// TODO: Turn into simple struct
#[derive(Debug, Clone, PartialEq)]
pub enum AddModuleError {
    /// A module with the same [`name`](Module::name) already exists.
    ModuleExists {
        /// The index of the first module with the same name in the corresponding [`ModuleManager`].
        index: usize,
    },
}

/// A [`Module`] using [`Identifier`] to refer to variables in expressions, instead of the default
/// of [`VariableReference`].
pub type ModuleNamedVars<S: Span = FullSpan, A = Identifier<S>> =
    Module<Identifier<S>, S, Expression<Identifier<S>, S>, A>;

/// A PRISM module.
///
/// Each module has a name, a list of commands and a [`Span`]. Module variables are stored in
/// [`Model::variable_manager`](crate::Model::variable_manager), not in the module.
#[derive(PartialEq, Clone, Debug)]
pub struct Module<
    V = VariableReference,
    S: Span = FullSpan,
    E = Expression<V, S>,
    A = Identifier<S>,
> {
    /// The name of the module
    pub name: Identifier<S>,

    /// The commands of the module
    pub commands: Vec<Command<V, S, E, A>>,

    /// The span of the module
    pub span: S,
}

impl<V, S: Span, E, A> Module<V, S, E, A> {
    /// Constructs a module with given name, empty list of commands and empty [`Span`].
    ///
    /// To construct a module with given span, use [`Module::new_spanned()`].
    pub fn new(name: Identifier<S>) -> Self {
        Self::new_spanned(name, S::empty())
    }

    /// Constructs a module with given name, empty list of commands and given [`Span`].
    ///
    /// To construct a module with empty span, use [`Module::new()`].
    pub fn new_spanned(name: Identifier<S>, span: S) -> Self {
        Self {
            name,
            commands: Vec::new(),
            span,
        }
    }
}
impl<V, S: Span, A> Module<V, S, Expression<V, S>, A> {
    /// Maps every [`Span`] of this `Module` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to [`name`](Self::name), every [`Command`] in
    /// [`commands`](Self::commands) and [`span`](Self::span).
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> Module<V, S2, Expression<V, S2>, A> {
        Module {
            name: self.name.map_span(map),
            commands: self.commands.into_iter().map(|c| c.map_span(map)).collect(),
            span: map(self.span),
        }
    }
}

impl<V, S: Span, E, A> crate::private::Sealed for Module<V, S, E, A> {}
impl<'a, 'b, Ctx, V: Displayable<Ctx>, S: Span, E: Displayable<Ctx>, A: Display>
    Displayable<(usize, &VariableManager<S, E>, &Ctx)> for Module<V, S, E, A>
{
    fn fmt_internal(
        &self,
        f: &mut Formatter<'_>,
        (own_index, variable_manager, context): &(usize, &VariableManager<S, E>, &Ctx),
    ) -> std::fmt::Result {
        writeln!(f, "module {}", self.name)?;
        write!(
            f,
            "{}",
            variable_manager.displayable(&(
                VariablePrintingStyle::LocalVar {
                    module_index: *own_index
                },
                context
            ))
        )?;
        for command in &self.commands {
            writeln!(f, "    {}", command.displayable(context))?;
        }
        writeln!(f, "endmodule")
    }
}

/// A renamed module.
///
/// This corresponds to the PRISM syntax
///
/// ```prism
/// module new_name = old_name [ x=y, y=x ] endmodule
/// ```
///
/// and indicates that the module is similar to [`old_name`](RenamedModule::old_name), but with
/// name [`new_name`](RenamedModule::new_name) and the actions and variables in declarations and
/// commands renamed according to the [`rename_rules`](RenamedModule::rename_rules).
///
/// Renamed modules are stored in [`Model::renamed_modules`](crate::Model::renamed_modules).
///
/// Use [Model::expand_renamed_modules()](crate::Model::expand_renamed_modules()) to transform the
/// renamed modules of a model into normal modules.
#[derive(PartialEq, Clone, Debug)]
pub struct RenamedModule<S: Span = FullSpan> {
    /// The name of the module that will be copied during renaming.
    pub old_name: Identifier<S>,

    /// The name of the new module.
    pub new_name: Identifier<S>,

    /// The renaming rules applied to variables and actions during the renaming. They are applied
    /// simultaneously, i.e. you can use this to swap two variables by adding two rules `x` -> `y`
    /// and `y` -> `x`.
    pub rename_rules: RenameRules<S>,

    /// The [`Span`] covering the entire renamed module.
    pub span: S,
}

impl<S: Span> RenamedModule<S> {
    /// Constructs a `RenamedModule` with given properties and empty [`Span`]. Refer to
    /// [`RenamedModule`] for documentation of the parameters.
    ///
    /// To construct a `RenamedModule` with given span, use [`RenamedModule::new_spanned()`].
    pub fn new(
        old_name: Identifier<S>,
        new_name: Identifier<S>,
        rename_rules: RenameRules<S>,
    ) -> Self {
        Self::new_spanned(old_name, new_name, rename_rules, Span::empty())
    }

    /// Constructs a `RenamedModule` with given properties and given [`Span`]. Refer to
    /// [`RenamedModule`] for documentation of the parameters.
    ///
    /// To construct a `RenamedModule` with empty span, use [`RenamedModule::new()`].
    pub fn new_spanned(
        old_name: Identifier<S>,
        new_name: Identifier<S>,
        rename_rules: RenameRules<S>,
        span: S,
    ) -> Self {
        Self {
            old_name,
            new_name,
            rename_rules,
            span,
        }
    }

    /// Maps every [`Span`] of this `RenamedModule` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to [`old_name`](Self::old_name), [`new_name`](Self::new_name), every
    /// [`RenameRule`] in [`rename_rules`](Self::rename_rules) and [`span`](Self::span).
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> RenamedModule<S2> {
        RenamedModule {
            old_name: self.old_name.map_span(map),
            new_name: self.new_name.map_span(map),
            rename_rules: RenameRules {
                rules: self
                    .rename_rules
                    .rules
                    .into_iter()
                    .map(|i| i.map_span(map))
                    .collect(),
            },
            span: map(self.span),
        }
    }
}

impl<S: Span> Display for RenamedModule<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "module {} = {} [", self.new_name, self.old_name)?;
        let mut is_first = true;
        for rename_rule in &self.rename_rules.rules {
            if !is_first {
                write!(f, ", ")?;
            }
            is_first = false;
            write!(f, "{}", rename_rule)?;
        }
        write!(f, "] endmodule")
    }
}

/// A list of [`RenameRule`]s, expressing a list of variable renamings.
///
/// Each renaming replaces one variable identifier with another. The renamings are executed
/// simultaneously: For example, given renaming rules `x` -> `y`, `y` -> `z` and `z` -> `x`, the
/// expression `x + (y * z)` is transformed into `y + (z * x)`.
#[derive(PartialEq, Clone, Debug)]
pub struct RenameRules<S: Span = FullSpan> {
    /// The list of renaming rules
    pub rules: Vec<RenameRule<S>>,
}

impl<S: Span> RenameRules<S> {
    /// Constructs an empty set of renaming rules.
    ///
    /// Add rules with [`RenameRules::add_rule()`] or by using [`RenameRules::with_rules()`].
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Constructs a set of renaming rules from the given vector of rules.
    ///
    /// To construct an empty set of renaming rules, use [`RenameRules::new()`] and add rules with
    /// [`RenameRules::add_rule()`].
    pub fn with_rules<R: Into<Vec<RenameRule<S>>>>(rules: R) -> Self {
        Self {
            rules: rules.into(),
        }
    }

    /// Adds a rule to the list of renaming rules.
    ///
    /// Instead of adding rules individually, consider using [`RenameRules::with_rules()`].
    pub fn add_rule(&mut self, rule: RenameRule<S>) {
        self.rules.push(rule);
    }

    /// Looks up a renaming for the given old name.
    ///
    /// If a renaming exists, it returns the new name corresponding to the given old name. If no
    /// renaming exists, returns `None`.
    pub fn get_renaming(&self, old_name: &Identifier<S>) -> Option<Identifier<S>> {
        for rule in &self.rules {
            if &rule.old_name == old_name {
                return Some(rule.new_name.clone());
            }
        }
        None
    }
}

/// A renaming rule, expressing that `old_name` should be renamed to `new_name`.
#[derive(PartialEq, Clone, Debug)]
pub struct RenameRule<S: Span = FullSpan> {
    /// The old name which will be renamed by this rule
    pub old_name: Identifier<S>,

    /// The new name
    pub new_name: Identifier<S>,

    /// The [`Span`] covering the renaming rule.
    pub span: S,
}
impl<S: Span> RenameRule<S> {
    /// Constructs a renaming rule with the given parameters and empty span.
    ///
    /// Refer to [`RenameRule`] for details. To construct a `RenameRule` with non-empty span, use
    /// [`RenameRule::new_spanned()`].
    pub fn new(old_name: Identifier<S>, new_name: Identifier<S>) -> Self {
        Self::new_spanned(old_name, new_name, S::empty())
    }

    /// Constructs a renaming rule with the given parameters and span.
    ///
    /// Refer to [`RenameRule`] for details. To construct a `RenameRule` with empty span, use
    /// [`RenameRule::new()`].
    pub fn new_spanned(old_name: Identifier<S>, new_name: Identifier<S>, span: S) -> Self {
        Self {
            old_name,
            new_name,
            span,
        }
    }

    /// Maps every [`Span`] of this `RenameRule` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to [`old_name`](Self::old_name), [`new_name`](Self::new_name) and
    /// [`span`](Self::span).
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> RenameRule<S2> {
        RenameRule {
            old_name: self.old_name.map_span(map),
            new_name: self.new_name.map_span(map),
            span: map(self.span),
        }
    }
}
impl<S: Span> Display for RenameRule<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.old_name, self.new_name)
    }
}
