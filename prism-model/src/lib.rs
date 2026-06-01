// The `NamedVars` versions of every component use a type alias variable bound, which are currently
// not checked. Let's suppress these warnings.
#![allow(type_alias_bounds)]
#![warn(missing_docs)]

//! A library to represent a model written in the [PRISM modelling language](https://www.prismmodelchecker.org/manual/ThePRISMLanguage/Introduction).
//!
//! # Features
//!
//! - Markov decision processes (MDPs) and discrete-time Markov chains [^other_types].
//! - [model normalisation](Model#normalising-a-model) (e.g. expanding formulas or renamed modules)
//! - [model transformations](Model#transforming-a-model) (e.g. mapping expressions or adding an
//!   init constraint)
//! - [variable references by name or by index](Model#variables-and-references)
//! - [spans](`Span`) to link model components to the corresponding source code
//! - [generic types for variables, expressions and spans](Model#generics)
//!
//! # Example
//!
//! A PRISM model is represented by [`Model`]. Refer to its documentation for more details.
//!
//! ```
//! use prism_model::*;
//! let mut model: Model = Model::new(ModelType::mdp());
//!
//! // Create global variable `var1` with bounds `[-3..5]`
//! let var = model.variable_manager.add_variable(VariableInfo::global_var(
//!     Identifier::new("var1").unwrap(),
//!     VariableRange::bounded_int(Expression::int(-3), Expression::int(5)),
//! )).expect("Error adding variable:");
//!
//! // Create module with name `mod1`
//! let mut module = Module::new(Identifier::new("mod1").unwrap());
//!
//! // Add command `[alpha] (var1<5) -> 1.0: (var1'=var1+1);` to `mod1`
//! let action = Some(Identifier::new("alpha").unwrap());
//! let guard = Expression::var_or_const(var).less_than(Expression::int(5));
//! let update = Update::with_assignments(
//!     Expression::float(1.0), // <- probability of update
//!     vec![Assignment::new(var, Expression::var_or_const(var).plus(Expression::int(1)))]);
//! module.commands.push(Command::with_updates(action, guard, vec![update]));
//!
//! let module_index = model.modules.add(module).unwrap();
//! ```
//!
//! # Sister crates
//!
//! These sister crates are available at <https://github.com/johannesalehmann/tiny-pmc>. Once they
//! reach maturity, they will be published on crates.io.
//!
//! - `prism-parser`: Parses a PRISM model and returns [`prism_model::Model`](Model).
//! - `prism-model-builder`: Builds a state-based model from a given [`prism_model::Model`](Model).
//!
//! The following sister crates are still in early stages of development and should be used only
//! experimentally.
//! - `probabilistic-models`: Types for state-based model representations
//! - `probabilistic-model-algorithms`: Algorithms for verifying state-based probabilistic models,
//!   e.g. value iteration for MDPs and stochastic games.
//! - `tiny-pmc` and `tiny-pmc-cli`: High-level interface for parsing, building and checking
//!   PRISM models.
//!
//! [^other_types]: Continuous-time Markov chains should work as well by treating
//!                 [`Update::probability`] as the rate.

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
    FormulaManagerNamedVars, FormulaNamedVars, SpannedDependency,
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
    MissingVariableRenaming, VariableAddError, VariableInfo, VariableInfoNamedVars,
    VariableManager, VariableManagerNamedVars, VariablePrintingStyle, VariableRange,
    VariableRangeNamedVars, VariableReference,
};

mod identifier;
pub use identifier::{Identifier, InvalidName};

mod displayable;
pub(crate) use displayable::private;
pub use displayable::{Displayable, DisplayableWithContext};

mod spans;
pub use spans::{FullSpan, Span};
