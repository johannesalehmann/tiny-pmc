mod formula_substitution;

mod map_expression_cloned;
mod map_expression_into;
mod map_expressions;
mod renamed_module_expansion;
mod variable_indexing;

pub use formula_substitution::FormulaSubstitutionVisitor;
pub use renamed_module_expansion::ModuleExpansionError;

use crate::formulas::FormulaManager;
use crate::module::RenamedModule;
use crate::rewards::RewardsManager;
use crate::spans::{FullSpan, Span};
use crate::{
    Displayable, Expression, Identifier, LabelManager, ModuleManager, VariableInfo,
    VariableManager, VariablePrintingStyle, VariableRange, VariableReference,
};
use std::fmt::{Display, Formatter};

/// A [`Module`](crate::Module) using [`Identifier`] to refer to variables in expressions, instead of the default
/// of [`VariableReference`].
pub type ModelNamedVars<S: Span = FullSpan, A = Identifier<S>> =
    Model<Identifier<S>, S, Expression<Identifier<S>, S>, A>;

/// A PRISM model.
///
/// # Generics
///
/// `Model` and most of its components use the following generic parameters:
/// - `V` represents variables. The default is [`VariableReference`]. [`Identifier`] is also widely
///   used and there are types aliases with suffix `NamedVars`(e.g. `ModelNamedVars`) to set
///   `V = Identifier`. [See below](#varref_details) for a discussion of details.
/// - `S` stores [spans](Span), which store source code locations. The default is [`FullSpan`].
/// - `E` represents expressions. The default is [`Expression`].
/// - `A` stores action names. The default is [`Identifier`].
///
/// # Constructing a model
///
/// ```
/// # use prism_model::*;
/// // Use ModelNamedVars to refer to variables by name instead of reference.
/// let mut model: ModelNamedVars = Model::new(ModelType::mdp());
/// let x_name = Identifier::new("x").unwrap();
///
/// // label "goal" = x=10;
/// model.labels.add_label(Label::new(
///     Identifier::new("goal").unwrap(),
///     Expression::var_or_const(x_name.clone()).equals_to(Expression::int(10))
/// ));
///
/// let mut main = Module::new(Identifier::new("main").unwrap());
///
/// // [alpha] x < 10 -> 0.5: (x'=x+1) + 0.5 ();
/// main.commands.push(Command::with_updates(
///     Some(Identifier::new("alpha").unwrap()),
///     Expression::var_or_const(x_name.clone()).less_than(Expression::int(10)),
///     vec![
///         // 0.5: (x'=x+1)
///         Update::with_assignments(
///             Expression::float(0.5),
///             vec![ Assignment::new(
///                 x_name.clone(),
///                 Expression::var_or_const(x_name.clone()).plus(Expression::int(1))
///             ) ]
///         ),
///         // 0.5: ()
///         Update::with_assignments(
///             Expression::float(0.5),
///             Vec::new()
///         )
///     ]
/// ));
///
/// // [] x=10 -> true;
/// main.commands.push(Command::new(
///     None,
///     Expression::var_or_const(x_name.clone()).equals_to(Expression::int(10)))
/// );
///
/// let main_index = model.modules.add(main).expect("Failed to add module");
///
/// // All variables are stored in `variable_manager`, with the module's index for local variables:
/// model.variable_manager.add_variable(VariableInfo::local_var(
///     x_name,
///     VariableRange::bounded_int(Expression::int(0), Expression::int(10)),
///     main_index)
/// );
///
/// // This now replaces the variable names with corresponding references:
/// let model_with_references: Model = model.replace_identifiers_by_variable_indices()
///     .expect("Could not find variable(s):");
/// ```
///
/// `Model` and its components can track references to source code in [`Span`]`s`. Most constructors
/// have a `_spanned` variant (e.g. [`Model::new_spanned()`]) to add a span.
// TODO: Add reference to PRISM Parser once ready
///
/// # Normalising a model
///
/// The PRISM language contains several features that simplify modelling, but require special care
/// when analysing the model. The following functions transform some special features into more
/// standard representations.
///
/// - [`substitute_formulas()`](Model::substitute_formulas) replaces every formula reference with
///   the formula's condition.
/// - [`expand_renamed_modules()`](Model::expand_renamed_modules) expands each renamed module
///   block into a full module.
/// - [`name_unnamed_actions()`](Model::name_unnamed_actions) assigns a unique name to every unnamed
///   action.
/// - [`replace_empty_updates_with_identity_update()`](Model::replace_empty_updates_with_identity_update)
///   Replaces each command of form `[alpha] guard -> true;` with one of form
///   `[alpha] guard -> 1: ();`.
/// - [`add_missing_init_statements()`](Model::add_missing_init_statements) makes implicit init
///   statements of variables explicit.
///
/// # Transforming a model
///
/// - [`map_expressions()`](Model::map_expressions),
///   [`map_expressions_cloned()`](Model::map_expressions_cloned) and
///   [`map_expressions_into()`](Model::map_expressions_into)
///   map the expressions of the model.
///
/// - [`map_span()`](Model::map_span) maps the [`Span`]`s` of the model.
///
/// - [`init_statements_to_init_block()`](Model::init_statements_to_init_block) replaces the init
///   statements of variables with an init constraint.
///
/// - [`replace_identifiers_by_variable_indices()`](Model::replace_identifiers_by_variable_indices)
///   transforms a model that uses [`Identifier`]`s` for variables into one that uses
///   [`VariableReference`]`s`. [See below](#varref_details) for details.
///
/// # Variables and references
///
/// PRISM models contain variables of types [booleans](`VariableRange::Boolean`),
/// [bounded](`VariableRange::BoundedInt`) and [unbounded](`VariableRange::UnboundedInt`) integers
/// and [floats](`VariableRange::Float`). Variables can be global or defined within a module. Global
/// variables can be marked as constants.
///
/// All variables are stored in [`Model::variable_manager`], even if they are defined in a module.
/// The variable manager stores variable names, types, scope and an optional initial value.
///
/// <a name="varref_details"></a>
/// Within expressions, variables can either be represented by an [`Identifier`] or by a
/// [`VariableReference`]. The former uses a [`String`] internally, the latter an index.
/// [`Model::replace_identifiers_by_variable_indices()`] transforms a model using [`Identifier`] into
/// one using [`VariableReference`]. Eponymous functions are available in most model components.
///
/// When dealing with models that use [`Identifier`], one can use the type alias [`ModelNamedVars`]
/// instead of [`Model<VariableReference>`]. Type aliases of the form `...NamedVars` are available
#[derive(PartialEq, Clone, Debug)]
pub struct Model<V = VariableReference, S: Span = FullSpan, E = Expression<V, S>, A = Identifier<S>>
{
    /// The type of model.
    ///
    /// Currently, this is mandatory (unlike in PRISM itself, where the model type can be omitted)
    // TODO: Make model type optional or devise some other way of handling models with missing model
    //  type.
    pub model_type: ModelType<S>,

    /// The variables and constants of the model, including local variables.
    pub variable_manager: VariableManager<S, E>,

    /// The formulas of the model.
    ///
    /// To expand formulas, see [`Model::substitute_formulas()`]. This replaces every reference to
    /// a formula in the model with that formula's value.
    pub formulas: FormulaManager<S, E>,

    /// The modules of the model.
    ///
    /// Renamed modules are stored separately in [`Model::renamed_modules`].
    pub modules: ModuleManager<V, S, E, A>,

    /// The renamed modules, corresponding to PRISM syntax `module M2 = M1 [ x=y, y=x ] endmodule`.
    ///
    /// To expand renamed modules (i.e. turn them into normal modules stored in [`Model::modules`],
    /// use [`Model::expand_renamed_modules()`].
    pub renamed_modules: Vec<RenamedModule<S>>,

    /// The initial states constraint. If present, all states that satisfy this constraint are
    /// initial state.
    ///
    /// This option is mutually exclusive with initial values for variables.
    pub init_constraint: Option<E>,

    /// The labels of the model.
    pub labels: LabelManager<S, E>,

    /// The collection of reward structures of the model.
    pub rewards: RewardsManager<S, E, A>,

    /// The [`Span`] of the entire model. Most model components also have individual spans, which
    /// can be used to add context to error messages.
    pub span: S,
}

impl<V, S: Span, E, A> Model<V, S, E, A> {
    /// Constructs an empty model with the given model type.
    ///
    /// The model will have no variables, no modules, no labels and formulas and no rewards.
    ///
    /// The model's [`Span`] is empty. Use [`Model::new_spanned()`] to use a custom span.
    pub fn new(model_type: ModelType<S>) -> Self {
        Self::new_spanned(model_type, S::empty())
    }

    /// Constructs an empty model with the given model type and [`Span`].
    ///
    /// The model will have no variables, no modules, no labels and formulas and no rewards.
    ///
    /// To construct a model with empty span, use [`Model::new()`].
    pub fn new_spanned(model_type: ModelType<S>, span: S) -> Self {
        Self {
            model_type,
            variable_manager: VariableManager::new(),
            formulas: FormulaManager::new(),
            modules: ModuleManager::new(),
            renamed_modules: Vec::new(),
            init_constraint: None,
            labels: LabelManager::new(),
            rewards: RewardsManager::new(),
            span,
        }
    }

    /// Constructs a model with the given parameters.
    ///
    /// See [`Model`] for details on the parameters.
    pub fn from_components(
        model_type: ModelType<S>,
        variable_manager: VariableManager<S, E>,
        formulas: FormulaManager<S, E>,
        modules: ModuleManager<V, S, E, A>,
        renamed_modules: Vec<RenamedModule<S>>,
        init_constraint: Option<E>,
        labels: LabelManager<S, E>,
        rewards: RewardsManager<S, E, A>,
        span: S,
    ) -> Self {
        Self {
            model_type,
            variable_manager,
            formulas,
            modules,
            renamed_modules,
            init_constraint,
            labels,
            rewards,
            span,
        }
    }
}
impl<V, S: Span, E> Model<V, S, E, Identifier<S>> {
    /// Assigns a unique name (of the form `unnamed_action_i` for `i >= 0`) to every command with unnamed action
    /// (i.e. every command where [`Command::action`](crate::Command::action)` = None`).
    ///
    /// To use a different naming scheme, use [`Model::name_unnamed_actions_with_custom_name`].
    ///
    /// # Issues
    ///
    /// If the model already contains an action of form
    pub fn name_unnamed_actions(&mut self) {
        // TODO: Make sure the model does not have any actions with these names
        self.name_unnamed_actions_with_custom_name(|i, _| format!("unnamed_action_{i}"))
    }

    /// Assigns a name to every command with unnamed action using the naming function `f`. This
    /// applies to every command with [`Command::action`](crate::Command::action)` = None`.
    ///
    /// `f` is called with two parameters:
    /// - `usize` is a running count of unnamed commands
    /// - `S` is the span covering the unnamed action, i.e.
    ///   [`Command::action_span`](crate::Command::action_span)`.
    ///
    /// # Issues
    ///
    /// The parameters of `f` will be changed in a future version to be more ergonomic. On the one
    /// hand, `f` should be able to construct a custom span. Additionally, providing the index is
    /// superfluous and `f` can simply do the counting internally.
    pub fn name_unnamed_actions_with_custom_name<F: FnMut(usize, &S) -> String>(
        &mut self,
        mut name_function: F,
    ) {
        // TODO: Support custom spans
        let mut counter = 0;
        for module in &mut self.modules.modules {
            for command in &mut module.commands {
                if command.action.is_none() {
                    command.action = Some(
                        Identifier::new_potentially_reserved_spanned(
                            name_function(counter, &command.action_span),
                            command.action_span.clone(),
                        )
                        .unwrap(),
                    );
                    counter += 1;
                }
            }
        }
    }

    /// Returns the names of all actions that are actually synchronising, i.e. that occur in at
    /// least two modules.
    ///
    /// A command with unnamed action (i.e. with [`Command::action`](crate::Command::action)` = None`)
    /// is never synchronising, but even if a command has a named action, it is only synchronising
    /// if the same action occurs in two or more modules.
    pub fn actually_synchronising_actions(&self) -> std::collections::HashSet<String> {
        use std::collections::HashSet;
        let mut seen_before = HashSet::new();
        let mut actually_synchronising = HashSet::new();
        for module in &self.modules.modules {
            let mut module_actions = HashSet::new();
            for command in &module.commands {
                if let Some(command) = &command.action {
                    if !module_actions.contains(&command.name) {
                        module_actions.insert(command.name.clone());
                    }
                }
            }

            for action in module_actions {
                if seen_before.contains(&action) {
                    actually_synchronising.insert(action);
                } else {
                    seen_before.insert(action);
                }
            }
        }

        actually_synchronising
    }
}

// TODO: This trait is only used to enable init_statements_to_init_block to work both when `V` is
//  `Identifier` and when `V` is `VariableReference`. Perhaps we can use some more general mechanism
//  or expose this trait more broadly?
pub trait VariableIdentifierProvider<S: Span, E> {
    fn get_variable_identifier(info: &VariableInfo<S, E>, index: usize) -> Self;
}

impl<S: Span, E> VariableIdentifierProvider<S, E> for VariableReference {
    fn get_variable_identifier(info: &VariableInfo<S, E>, index: usize) -> Self {
        let _ = info;
        VariableReference::new(index)
    }
}

impl<S: Span, E> VariableIdentifierProvider<S, E> for Identifier<S> {
    fn get_variable_identifier(info: &VariableInfo<S, E>, index: usize) -> Self {
        let _ = index;
        info.name.clone()
    }
}

impl<V, S: Span, A> Model<V, S, Expression<V, S>, A> {
    /// Maps every [`Span`] of this `Model` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to
    /// [`model_type`](Self::model_type),
    /// [`variable_manager`](Self::variable_manager),
    /// [`formulas`](Self::formulas),
    /// [`modules`](Self::modules),
    /// [`renamed_modules`](Self::renamed_modules),
    /// [`init_constraint`](Self::init_constraint),
    /// [`labels`](Self::labels),
    /// [`rewards`](Self::rewards) and
    /// [`span`](Self::span).
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> Model<V, S2, Expression<V, S2>, A> {
        Model {
            model_type: self.model_type.map_span(map),
            variable_manager: self.variable_manager.map_span(map),
            formulas: self.formulas.map_span(map),
            modules: self.modules.map_span(map),
            renamed_modules: self
                .renamed_modules
                .into_iter()
                .map(|m| m.map_span(map))
                .collect(),
            init_constraint: self.init_constraint.map(|i| i.map_span(map)),
            labels: self.labels.map_span(map),
            rewards: self.rewards.map_span(map),
            span: map(self.span),
        }
    }

    /// Adds an identity update (i.e. one with probability `1` and no assignments) to every command
    /// that has no updates.
    ///
    /// Commands with no updates are produced by the syntax `[alpha] guard -> true;`. To avoid
    /// handling this special case, this function adds an identity update, which is semantically
    /// equivalent.
    ///
    /// # Example
    ///
    /// Consider the following model:
    ///
    /// ```prism
    /// mdp
    /// module main
    ///     x: [0..10];
    ///     [] (x < 10) -> 1: (x'=x+1);
    ///     [] (x = 10) -> true;
    /// endmodule
    /// ```
    ///
    /// Calling `replace_empty_updates_with_identity_update()` results in the following equivalent
    /// model:
    ///
    /// ```prism
    /// mdp
    /// module main
    ///     x: [0..10];
    ///     [] (x < 10) -> 1: (x'=x+1);
    ///     [] (x = 10) -> 1: ();
    /// endmodule
    /// ```
    pub fn replace_empty_updates_with_identity_update(&mut self) {
        for module in &mut self.modules.modules {
            for command in &mut module.commands {
                if command.updates.len() == 0 {
                    command.updates.push(crate::Update::new_spanned(
                        Expression::Float(1.0, command.span.clone()),
                        command.span.clone(),
                    )); // TODO: The expression's and update's span should only cover the `true` token, but its span is currently not tracked
                }
            }
        }
    }

    /// Adds init statements to variables with missing init statements; making their implicit
    /// initial values explicit.
    ///
    /// This applies the following transformations to global and local variables.
    ///
    /// - `x: [min..max];` is transformed into `x: [min..max] init min;`
    /// - `y: bool` is transformed into `y: bool init false`
    ///
    /// These transformations preserve model semantics.
    ///
    /// Constants are not modified, as they have no implicit initial value.
    ///
    /// # Panics
    ///
    /// The function panics in the following cases:
    ///
    /// - If the model has an [init constraint](Model::init_constraint), because this is mutually
    ///   exclusive with init statements on variables
    /// - If an [unbounded integer](VariableRange::UnboundedInt) or [float](VariableRange::Float)
    ///   variable has no init statement, as these have no implicit init value.
    // TODO: Only constants may have type float, so the float case can never occur. After revamping
    //  the VariableRange types, rewrite this part of the documentation.
    ///
    /// # Example
    ///
    /// ```prism
    /// mdp
    ///
    /// module main
    ///     x: [0..10];
    ///     y: bool;
    /// endmodule
    /// ```
    pub fn add_missing_init_statements(&mut self)
    where
        V: Clone,
    {
        if self.init_constraint.is_some() {
            panic!(
                "Cannot add missing init statements because the model uses an init constraint instead of init statements"
            );
        }

        for variable in &mut self.variable_manager.variables {
            if !variable.is_constant {
                if variable.initial_value.is_none() {
                    variable.initial_value = Some(match &variable.range {
                        VariableRange::BoundedInt { min, .. } => min.clone(),
                        VariableRange::UnboundedInt { .. } => {
                            panic!("Unbounded integers must have an initial value.")
                        }
                        VariableRange::Boolean { .. } => {
                            Expression::Bool(false, variable.range.span().clone())
                        }
                        VariableRange::Float { .. } => {
                            panic!("Unbounded integers must have an initial value.")
                        }
                    });
                }
            }
        }
    }

    /// Transforms a model without [init constraint](Model::init_constraint) into an equivalent model
    /// with init constraint. This removes the init statements of all variables.
    ///
    ///
    /// # Panics
    ///
    /// If the model already contains an init constraint, this function panics.
    ///
    /// This function calls [`Model::add_missing_init_statements()`], which panics if an unbounded
    /// integer or float variable without init statement is present in the model.
    // TODO: See the TODO in Model::add_missing_init_statements() on float variables.
    ///
    /// # Example
    ///
    /// Consider the following model:
    ///
    /// ```prism
    /// mdp
    ///
    /// module A
    ///     x: int init -2;
    ///     y: bool init true;
    /// endmodule
    ///
    /// module B
    ///     z: [-1..7];
    ///     w: bool;
    /// endmodule B
    /// ```
    ///
    /// Calling `init_statements_to_init_block` yields the following model:
    ///
    /// ```prism
    /// mdp
    ///
    /// init
    ///     x=-2 & y=true & z=-1 & w=false
    /// endinit
    ///
    /// module A
    ///     x: int;
    ///     y: bool;
    /// endmodule
    ///
    /// module B
    ///     z: [-1..7];
    ///     w: bool;
    /// endmodule B
    /// ```
    pub fn init_statements_to_init_block(&mut self)
    where
        V: Clone + VariableIdentifierProvider<S, Expression<V, S>>,
    {
        // TODO: Fix how new spans are created
        if self.init_constraint.is_some() {
            panic!(
                "Cannot transform init statements to init block because the model already uses an init block"
            );
        }

        let mut init_constraint: Option<Expression<V, S>> = None;

        self.add_missing_init_statements();

        for (variable_index, variable) in self.variable_manager.variables.iter_mut().enumerate() {
            if !variable.is_constant {
                match std::mem::replace(&mut variable.initial_value, None) {
                    Some(value) => {
                        let identifier = V::get_variable_identifier(variable, variable_index);
                        let variable_constraint = Expression::Equals(
                            Box::new(Expression::VarOrConst(identifier, self.span.clone())),
                            Box::new(value),
                            self.span.clone(),
                        );
                        if let Some(prev_init) = init_constraint.take() {
                            let span = prev_init.span().clone();
                            init_constraint = Some(Expression::Conjunction(
                                Box::new(prev_init),
                                Box::new(variable_constraint),
                                span,
                            ));
                        } else {
                            init_constraint = Some(variable_constraint);
                        }
                        variable.initial_value = None;
                    }
                    None => {
                        panic!("Variable {} does not have initial value.", variable.name)
                    }
                }
            }
        }

        self.init_constraint = init_constraint;
    }
}

/// The type of model. Refer to the [PRISM manual](https://www.prismmodelchecker.org/manual/ThePRISMLanguage/ModelType)
/// for details.
///
/// Model types can be constructed using constructors, e.g. [`ModelType::dtmc()`] and
/// [`ModelType::mdp_spanned()`].
///
/// Each model type stores a [`Span`], which can be accessed by [`ModelType::span()`].
///
/// # Outlook
///
/// A future version will support the remaining model types supported by PRISM.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ModelType<S: Span = FullSpan> {
    /// Discrete-time Markov chain
    ///
    /// This is a model with probabilistic behaviour, but no non-deterministic choices.
    ///
    /// Construct this type using [`ModelType::dtmc()`] and [`ModelType::dtmc_spanned()`].
    Dtmc(S),

    /// Continuous-time Markov chain
    ///
    /// This is a model with transition rates and no non-deterministic choices.
    ///
    /// Construct this type using [`ModelType::ctmc()`] and [`ModelType::ctmc_spanned()`].
    Ctmc(S),

    /// Markov decision process
    ///
    /// This is a model with probabilistic behaviour and non-deterministic choices.
    ///
    /// Construct this type using [`ModelType::mdp()`] and [`ModelType::mdp_spanned()`].
    Mdp(S),
}

impl<S: Span> ModelType<S> {
    /// Constructs [`ModelType::Dtmc`] with empty [`Span`].
    ///
    /// Use [`ModelType::dtmc_spanned()`] to use a given span.
    pub fn dtmc() -> Self {
        Self::Dtmc(S::empty())
    }

    /// Constructs [`ModelType::Dtmc`] with given [`Span`].
    ///
    /// Use [`ModelType::dtmc()`] to use an empty span.
    pub fn dtmc_spanned(span: S) -> Self {
        Self::Dtmc(span)
    }

    /// Constructs [`ModelType::Ctmc`] with empty [`Span`].
    ///
    /// Use [`ModelType::ctmc_spanned()`] to use a given span.
    pub fn ctmc() -> Self {
        Self::Ctmc(S::empty())
    }

    /// Constructs [`ModelType::Ctmc`] with given [`Span`].
    ///
    /// Use [`ModelType::ctmc()`] to use an empty span.
    pub fn ctmc_spanned(span: S) -> Self {
        Self::Ctmc(span)
    }

    /// Constructs [`ModelType::Mdp`] with empty [`Span`].
    ///
    /// Use [`ModelType::mdp_spanned()`] to use a given span.
    pub fn mdp() -> Self {
        Self::Mdp(S::empty())
    }

    /// Constructs [`ModelType::Mdp`] with given [`Span`].
    ///
    /// Use [`ModelType::mdp()`] to use an empty span.
    pub fn mdp_spanned(span: S) -> Self {
        Self::Mdp(span)
    }

    /// Returns the [`Span`] of the model type.
    pub fn span(&self) -> &S {
        match self {
            ModelType::Dtmc(s) => s,
            ModelType::Ctmc(s) => s,
            ModelType::Mdp(s) => s,
        }
    }

    /// Maps the [`Span`] of the model type according to mapping function `map`.
    ///
    /// The new value is of type `S2`, which may be different from the original span type `S`. Usage
    /// is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> ModelType<S2> {
        match self {
            ModelType::Dtmc(span) => ModelType::Dtmc(map(span)),
            ModelType::Ctmc(span) => ModelType::Ctmc(map(span)),
            ModelType::Mdp(span) => ModelType::Mdp(map(span)),
        }
    }
}

impl<S: Span> Display for ModelType<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Dtmc(_) => {
                write!(f, "dtmc")
            }
            ModelType::Ctmc(_) => {
                write!(f, "ctmc")
            }
            ModelType::Mdp(_) => {
                write!(f, "mdp")
            }
        }
    }
}

impl<A, E, V, S: Span> crate::private::Sealed for Model<V, S, E, A> {}
impl<Ctx, A: Display, E: Displayable<Ctx>, V: Displayable<Ctx>, S: Span> Displayable<Ctx>
    for Model<V, S, E, A>
{
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result {
        writeln!(f, "{}", self.model_type)?;
        writeln!(f, "")?;
        write!(
            f,
            "{}",
            self.variable_manager
                .displayable(&(VariablePrintingStyle::Const, &context))
        )?;
        write!(
            f,
            "{}",
            self.variable_manager
                .displayable(&(VariablePrintingStyle::GlobalVar, &context))
        )?;
        write!(f, "{}", self.formulas.displayable(context))?;
        write!(f, "{}", self.labels.displayable(context))?;
        if let Some(init) = &self.init_constraint {
            writeln!(f, "init")?;
            writeln!(f, "    {}", init.displayable(context))?;
            writeln!(f, "endinit")?;
        }
        for (i, module) in self.modules.modules.iter().enumerate() {
            writeln!(
                f,
                "{}",
                module.displayable(&(i, &self.variable_manager, context))
            )?;
        }
        for renamed_module in &self.renamed_modules {
            writeln!(f, "{}", renamed_module)?;
        }
        for rewards in &self.rewards.rewards {
            writeln!(f, "{}", rewards.displayable(context))?;
        }

        Ok(())
    }
}

impl<S: Span, A: Display> Display
    for Model<VariableReference, S, Expression<VariableReference, S>, A>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.displayable(&self.variable_manager).fmt(f)
    }
}

impl<S: Span, A: Display> Display for Model<Identifier<S>, S, Expression<Identifier<S>, S>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.displayable(&()).fmt(f)
    }
}
