// The `NamedVars` versions of every component use a type alias variable bound, which are currently
// not checked. Let's suppress these warnings.
#![allow(type_alias_bounds)]

//! A library to represent a model written in the [PRISM modelling language](https://www.prismmodelchecker.org/manual/ThePRISMLanguage/Introduction).
//!
//! The PRISM modelling language is used to model
//! - Markov decision processes
//! - Markov chains (discrete- and continuous-time)
//! - probabilistic timed automata
//! - partially observable Markov decision processes
//! - partially observable probabilistic timed automata
//!
//! This library models the subset of the PRISM modelling language for Markov decision processes and
//! discrete-time Markov chains. The remaining model types are partially supported.
//!
//! A PRISM model is represented by [`Model`].
//!
//! # Example
//!
//! ```
//! use prism_model::*;
//! let mut model: Model = Model::new(ModelType::mdp());
//!
//! // Create global variable `var1` with bounds `[-3..5]`
//! let var_info = VariableInfo::global_var(
//!     Identifier::new("var1").unwrap(),
//!     VariableRange::bounded_int(Expression::int(-3), Expression::int(5)),
//! );
//! let var = model.variable_manager.add_variable(var_info).unwrap();
//!
//! // Create module with name `mod1`
//! let mut module = Module::new(Identifier::new("mod1").unwrap());
//!
//! // Add command `[alpha] (var1<5) -> 1.0: (var1'=var1+1);` to `mod1`
//! let action = Some(Identifier::new("alpha").unwrap());
//! let guard = Expression::var_or_const(var).less_than(Expression::int(5));
//! let probability = Expression::float(1.0);
//! let assignment = Assignment::new(var, Expression::var_or_const(var).plus(Expression::int(1)));
//! let update = Update::with_assignments(probability, vec![assignment]);
//! module.commands.push(Command::with_updates(action, guard, vec![update]));
//!
//! // Add `mod1` to the model
//! let module_index = model.modules.add(module).unwrap();
//! ```
//! # Generics
//!
//! `prism-model` is generic over the types used to represent
//! - actions (the default is [`Identifier`]),
//! - expressions (the default is [`Expression`]),
//! - variables (the default is [`VariableReference`], see also below)
//! - and spans, which store source code locations (the default is [`FullSpan`])
//!
//! ## Variables and references
//!
//! PRISM models contain variables of types [booleans](`VariableRange::Boolean`),
//! [bounded](`VariableRange::BoundedInt`) and [unbounded](`VariableRange::UnboundedInt`) integers
//! and [floats](`VariableRange::Float`). Variables can be global or defined within a module. Global
//! variables can be marked as constants.
//!
//! All variables are stored in [`Model::variable_manager`], even if they are defined in a module.
//! The variable manager stores variable names, types, ranges, scope and an optional initial value.
//!
//! Within expressions, variables can either be represented by an [`Identifier`] or by a
//! [`VariableReference`]. The former uses a [`String`] internally, the latter an index.
//! [`Model::replace_identifier_by_variable_indices`] transforms a model using [`Identifier`] into
//! one using [`VariableReference`]. Eponymous functions are available in most model components.
//!
//! When dealing with models that use [`Identifier`], one can use the type alias [`ModelNamedVars`]
//! instead of [`Model<VariableReference>`]. Type aliases of the form `...NamedVars` are available
//! for most model components.
//!
//! # Labels and functions
//!
//! TODO
//!
//! # Maps
//!
//! TODO
//!
//! # Printing a model
//!
//! TODO
//!
//! # Future work
//!
//! TODO

mod command;
pub use command::{
    Assignment, AssignmentNamedVars, Command, CommandNamedVars, Update, UpdateNamedVars,
};

mod expressions;
pub use expressions::{
    DefaultMapExpression, Expression, ExpressionNamedVars, IdentityMapExpression, MapExpression,
    UnknownVariableError,
};

mod module;
pub use module::{
    AddModuleError, Module, ModuleManager, ModuleManagerNamedVars, ModuleNamedVars, RenameRule,
    RenameRules, RenamedModule,
};

mod formulas;
pub use formulas::{
    AddFormulaError, CyclicDependency, CyclicDependencyEntry, Formula, FormulaManager,
    FormulaManagerNamedVars, FormulaNamedVars,
};

mod labels;
pub use labels::{AddLabelError, Label, LabelManager, LabelManagerNamedVars, LabelNamedVars};

mod model;
pub use model::{Model, ModelNamedVars, ModelType, ModuleExpansionError};

mod rewards;
pub use rewards::{
    AddRewardsError, Rewards, RewardsElement, RewardsElementNamedVars, RewardsManager,
    RewardsManagerNamedVars, RewardsNamedVars, RewardsTarget,
};

mod variables;
pub use variables::{
    VariableAddError, VariableInfo, VariableInfoNamedVars, VariableManager,
    VariableManagerNamedVars, VariablePrintingStyle, VariableRange, VariableRangeNamedVars,
    VariableReference,
};

mod identifier;
pub use identifier::{Identifier, InvalidName};

mod displayable;
pub(crate) use displayable::private;
pub use displayable::{Displayable, DisplayableWithContext};

mod spans;
pub use spans::{FullSpan, Span};
