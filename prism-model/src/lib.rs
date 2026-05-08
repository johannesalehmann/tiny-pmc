#![allow(type_alias_bounds)]

mod command;
pub use command::{
    Assignment, AssignmentNamedVars, Command, CommandNamedVars, Update, UpdateNamedVars,
};

mod expressions;
pub use expressions::{
    DefaultMapExpression, Expression, ExpressionNamedVars, GlobalVariableReference,
    IdentityMapExpression, MapExpression, UnknownVariableError, VariableScope,
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
