mod label_substitution;
mod maps;

use crate::expressions::label_substitution::LabelSubstitutionVisitor;
use crate::expressions::map_variable::MapVariable;
use crate::module::RenameRules;
use crate::spans::{FullSpan, Span};
use crate::{
    CyclicDependency, Displayable, FormulaManager, Identifier, LabelManager, VariableManager,
    VariableReference,
};
pub use maps::*;
use std::fmt::{Display, Formatter};

/// An expression using [`Identifier`] to refer to variables, instead of the default of
/// [`VariableReference`].
///
/// Use [`Expression::replace_identifiers_by_variable_indices`] to convert
/// from the former to the latter.
pub type ExpressionNamedVars<S: Span = FullSpan> = Expression<Identifier<S>, S>;

// TODO: Add link to prism-model-builder crate once it is published

// TODO: Break up the big example into smaller examples

/// Represents an expression, consisting of mathematical and logical operations applied to constant
/// values and variables.
///
/// `Expression` is a recursive datatype and represents any valid PRISM expression [1].
///
/// # Example
///
/// ```
/// # use prism_model::*;
/// // -42 + 84.0
/// let addition: Expression = Expression::int(-42).plus(Expression::float(84.0));
///
/// // false | 3 < 5
/// let comparison: Expression = Expression::bool(false).or(Expression::int(3).less_than(Expression::int(5)));
///
/// // Associating expressions with a span to keep track of corresponding source code
/// let spanned_expression: Expression = Expression::bool_spanned(true, FullSpan::from_range(12..16));
/// assert_eq!(spanned_expression.span().end(), Some(16));
///
/// // Expressions referring to the variable with name `"x"` and index 5, respectively
/// let named_var: ExpressionNamedVars = Expression::var_or_const(Identifier::new("x").unwrap());
/// let indexed_var: Expression = Expression::var_or_const(VariableReference::new(5));
/// // `Variable::replace_identifiers_by_variable_indices(...)` transforms named_var into indexed_var
///
/// // min(3, -5)
/// let min: Expression = Expression::function(Identifier::new_potentially_reserved("min").unwrap(),
///     vec![Expression::int(3), Expression::int(-5)]
/// );
/// ```
///
/// # Constructing expressions
///
/// Each expression has a helper function to construct it, e.g. [`Expression::int`] or
/// [`Expression::equals_to`]. This creates an expression without a span. A similar helper function
/// with suffix `_spanned`, e.g. [`Expression::int_spanned`] constructs an expression with a span
/// in order to link the expression to the corresponding PRISM code.
///
/// # Evaluating expressions
///
/// This crate *does not* provide a method to evaluate expressions.
///
/// Crate `prism-model-builder` provides a `TreeWalkingEvaluator` to evaluate expressions, given
/// a suitable valuation. For better performance, `prism-model-builder` also provides a
/// `StackBasedExpression`. An expression can be transformed into a `StackBasedExpression` and then
/// evaluated, yielding performance gains.
///
/// # Transforming expressions
///
/// To map the span of an expression, use [`Expression::map_span()`]. To map variable references,
/// use [`Expression::map_variable()`]. An expression using [`Identifier`] to store variable names
/// can be transformed into one using [`VariableReference`] with
/// [`Expression::replace_identifiers_by_variable_indices`].
///
/// Expressions can be transformed arbitrarily by implementing trait [`MapExpression`] and calling
/// [`Expression::visit`].
///
/// # Formulas
///
/// References to formulas can be stored as a variable:
/// ```
/// use prism_model::{Expression, FullSpan, Identifier};
/// let formula: Expression<Identifier, FullSpan> = Expression::var_or_const(Identifier::new("formula_name").unwrap());
/// ```
///
/// As a formula does not have a variable index, they are generally replaced by the formula's value
/// before calling [`Expression::replace_identifiers_by_variable_indices`]. See
/// [`crate::Model::substitute_formulas()`] for details.
///
/// # Labels
///
/// Labels are represented by [`Expression::Label`]. This is intended for PRISM properties, as PRISM
/// models cannot use labels in their expressions.
///
/// [1]: https://prismmodelchecker.org/manual/ThePRISMLanguage/Expressions
#[derive(PartialEq, Clone)]
pub enum Expression<V = VariableReference, S: Span = FullSpan> {
    /// An integer literal.
    ///
    /// # Construction
    ///
    /// An integer literal can be constructed using [`Expression::int()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::int(-6);
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::int_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::int_spanned(4, FullSpan::from_range(8..9));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Int(inner, _)` evaluates to `inner`.
    ///
    /// # Types
    ///
    /// `Expression::Int(inner, span)` is of type integer. `span` stores the expression's [`Span`].
    Int(i64, S),

    /// A floating-point number literal.
    ///
    /// # Construction
    ///
    /// A float literal can be constructed using [`Expression::float()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::float(4.5);
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::float_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::float_spanned(8.9, FullSpan::from_range(6..9));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Float(inner, _)` evaluates to `inner`.
    ///
    /// # Types
    ///
    /// `Expression::Float(inner, span)` is of type float. `span` stores the expression's [`Span`].
    Float(f64, S),

    /// A boolean literal.
    ///
    /// # Construction
    ///
    /// A boolean literal can be constructed using [`Expression::bool()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::bool(true);
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::bool_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::bool_spanned(false, FullSpan::from_range(3..8));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Bool(inner, _)` evaluates to `inner`.
    ///
    /// # Types
    ///
    /// `Expression::Bool(inner, span)` is of type bool. `span` stores the expression's [`Span`].
    Bool(bool, S),

    /// A variable, constant, or formula reference.
    ///
    /// [`Expression`] is generic over the type used to represent variables. Usually, either
    /// [`Identifier`] is used (which stores a variable's name) or [`VariableReference`] (which
    /// stores a variable's index in the model's [`VariableManager`].
    ///
    /// Formulas can only be represented by [`Expression<Identifier>`], not by
    /// [`Expression<VariableReference>`]. Call [`Expression::substitute_formulas()`] before
    /// transforming an `Identifier`-based expression into a `VariableReference`-based one.
    ///
    /// # Constructing `Expression<Identifier>::VarOrConst`:
    ///
    /// A variable expression can be created using [`Expression::var_or_const()`]:
    /// ```
    /// # use prism_model::*;
    /// # let expr: Expression<Identifier> =
    /// Expression::var_or_const(Identifier::new("x").unwrap());
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::var_or_const_spanned()`]:
    ///
    /// ```
    /// # use prism_model::*;
    /// # let expr: Expression<Identifier> =
    /// Expression::var_or_const_spanned(Identifier::new("x").unwrap(), FullSpan::from_range(5..6));
    /// ```
    ///
    /// # Constructing `Expression<VariableReference>::VarOrConst`:
    ///
    /// Let `vm` be a [`VariableManager`]. Usually, this is the one stored in
    /// [`crate::Model::variable_manager`].
    /// ```
    /// # use prism_model::*;
    /// let mut vm: VariableManager = VariableManager::new();
    /// ```
    ///
    /// Now add a variable to the variable manager, obtaining a variable reference (or an error, if
    /// the variable already exists):
    ///
    /// ```
    /// # use prism_model::*;
    /// # fn main() -> Result<(), VariableAddError> {
    /// # let mut vm: VariableManager = VariableManager::new();
    /// let var_ref = vm.add_variable(VariableInfo::global_var(
    ///     Identifier::new("y").unwrap(),
    ///     VariableRange::bool())
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Alternatively, a reference for an existing variable can be obtained using
    /// [`VariableManager::get_reference`]:
    ///
    /// ```
    /// # use prism_model::*;
    /// # fn main() -> Result<(), VariableAddError> {
    /// # let mut vm: VariableManager = VariableManager::new();
    /// # let var_ref = vm.add_variable(VariableInfo::global_var(
    /// #     Identifier::new("y").unwrap(),
    /// #     VariableRange::bool())
    /// # )?;
    /// let var_ref_2 = vm.get_reference(&Identifier::new("y").unwrap())
    ///    .expect("Did not find variable with name `y`");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The variable expression can then be constructed:
    ///
    /// ```
    /// # use prism_model::*;
    /// # fn main() -> Result<(), VariableAddError> {
    /// # let mut vm: VariableManager = VariableManager::new();
    /// # let var_ref = vm.add_variable(VariableInfo::global_var(
    /// #     Identifier::new("y").unwrap(),
    /// #     VariableRange::bool())
    /// # )?;
    /// # let var_ref_2 = vm.get_reference(&Identifier::new("y").unwrap())
    /// #    .expect("Did not find variable with name `y`");
    /// let exp_with_ref: Expression = Expression::var_or_const(var_ref);
    /// let exp_with_ref_2: Expression = Expression::var_or_const(var_ref_2);
    ///
    /// assert_eq!(exp_with_ref, exp_with_ref_2);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Evaluation
    ///
    /// A variable expression evaluates to the value of the variable it represents. If it refers to
    /// a formula, it cannot be evaluated. Call [`Expression::substitute_formulas()`] to replace
    /// each formula reference with the formula's value.
    ///
    /// # Types
    ///
    /// In `Expression::VarOrConst(name, span)`, `name` stores the variable's name and `span` stores
    /// the expression's [`Span`].
    VarOrConst(V, S),

    // TODO: Consider using `Identifier` instead of V to represent labels. After all, it never makes
    //  sense to using `VariableReference` to refer to a label.
    /// A label.
    ///
    /// Labels can only meaningfully be represented by [`Expression<Identifier>`], not by
    /// [`Expression<VariableReference>`]. Before calling
    /// [`Expression::replace_identifiers_by_variable_indices`], use
    /// [`Expression::substitute_labels()`] to replace labels with their corresponding references.
    ///
    /// # Construction
    ///
    /// A label expression can be created using [`Expression::label()`]:
    /// ```
    /// # use prism_model::*;
    /// # let expr: Expression<Identifier> =
    /// Expression::label(Identifier::new("goal").unwrap());
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::label_spanned()`]:
    ///
    /// ```
    /// # use prism_model::*;
    /// # let expr: Expression<Identifier> =
    /// Expression::label_spanned(Identifier::new("goal").unwrap(), FullSpan::from_range(5..11));
    /// ```
    ///
    /// # Substitution
    ///
    /// Let `labels` be a [`LabelManager`] with label `"goal"` (with value `false`):
    ///
    /// ```
    /// # use prism_model::*;
    /// let mut labels: LabelManager<FullSpan, Expression<Identifier>> = LabelManager::new();
    /// ```
    ///
    /// Add a label `"goal"` (with condition `false`):
    ///
    /// ```
    /// # use prism_model::*;
    /// # let mut labels: LabelManager<FullSpan, Expression<Identifier>> = LabelManager::new();
    /// labels.add_label(Label::new(Identifier::new("goal").unwrap(), Expression::bool(false)));
    /// ```
    ///
    /// Now construct a label expression with name `"goal"`:
    ///
    /// ```
    /// # use prism_model::*;
    /// # let mut labels: LabelManager<FullSpan, Expression<Identifier>> = LabelManager::new();
    /// # labels.add_label(Label::new(Identifier::new("goal").unwrap(), Expression::bool(false)));
    ///
    /// let mut expr = Expression::label(Identifier::new("goal").unwrap());
    /// # expr.substitute_labels(&labels);
    /// # assert_eq!(expr, Expression::bool(false));
    /// ```
    ///
    /// Then calling [`Expression::substitute_labels()`] replaces every `Expression::Label` in
    /// `expr` by its value:
    ///
    /// ```
    /// # use prism_model::*;
    /// # let mut labels: LabelManager<FullSpan, Expression<Identifier>> = LabelManager::new();
    /// # labels.add_label(Label::new(Identifier::new("goal").unwrap(), Expression::bool(false)));
    ///
    /// # let mut expr = Expression::label(Identifier::new("goal").unwrap());
    /// expr.substitute_labels(&labels);
    /// assert_eq!(expr, Expression::bool(false));
    /// ```
    ///
    /// # Evaluation
    ///
    /// Label expressions cannot be evaluated. They must only occur in properties, not models, and
    /// should be expanded by calling [`Expression::substitute_labels()`] before evaluating the
    /// expression.
    ///
    /// # Types
    ///
    /// In `Expression::Label(name, span)`, `name` stores the label's name and `span` stores the
    /// expression's [`Span`].
    Label(V, S),

    /// Function call
    ///
    /// A function consists of a name (stored as [`Identifier`]) and a list of arguments, stored
    /// in a `Vec<`[`Expression`]`>`.
    /// PRISM only supports a pre-defined list of functions, but this type can express any function
    /// name. No check against the list of valid functions is performed at this stage.
    ///
    /// # Construction
    ///
    /// A function call can be constructed using [`Expression::function()`]. When constructing the
    /// identifier of the function name, use [`Identifier::new_potentially_reserved()`] because
    /// function names are reserved names.
    ///
    /// ```
    /// # use prism_model::*;
    /// # let ex: Expression =
    /// Expression::function(
    ///     Identifier::new_potentially_reserved("min").unwrap(),
    ///     vec![Expression::int(3), Expression::float(-5.3)]
    /// );
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::function_spanned()`]:
    ///
    /// ```
    /// # use prism_model::*;
    /// # let ex: Expression =
    /// Expression::function_spanned(
    ///     Identifier::new_potentially_reserved("min").unwrap(),
    ///     vec![Expression::int(3), Expression::float(-5.3)],
    ///     FullSpan::from_range(0..12)
    /// );
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Function(name, args, _)` follows the semantics of PRISM functions if `name` is
    /// a PRISM function.
    ///
    /// # Types
    ///
    /// The arguments of a function call must adhere to the types allowed by PRISM. Integer
    /// arguments are automatically converted to floats when appropriate.
    ///
    /// In `Expression::Function(_, _, span)`, `span` stores the expression's [`Span`].
    Function(Identifier<S>, Vec<Expression<V, S>>, S),

    /// Integer and float negation (-x)
    ///
    /// Booleans are negated using [`Expression::Negation`] and the associated constructors
    /// [`Expression::negate_bool()`] and [`Expression::negate_bool_spanned()`].
    ///
    /// # Construction
    ///
    /// An expression can be negated using [`Expression::negate_value()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::int(12).negate_value();
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::negate_value_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::int(4).negate_value_spanned(FullSpan::from_range(3..6));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Minus(inner, _)` evaluates to the negation of the value of `inner`, e.g. -5.5
    /// if `inner` evaluates to 5.5.
    ///
    /// # Types
    ///
    /// In `Expression::Minus(inner, span)`, `inner` must be of type integer or float.
    /// `Expression::Minus(_, _)` is of the same type as `inner`.
    ///
    /// `span` stores the expression's [`Span`].
    Minus(Box<Expression<V, S>>, S),

    /// Multiplication (x times y)
    ///
    /// # Construction
    ///
    /// A multiplication expression can be constructed using [`Expression::times()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::int(12).times(Expression::float(3.5));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::times_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::int(4).times_spanned(Expression::int(5), FullSpan::from_range(3..6));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Multiplication(lhs, rhs, _)` evaluates to the product of `lhs` and `rhs`.
    ///
    /// # Types
    ///
    /// In `Expression::Multiplication(lhs, rhs, span)`, the types of `lhs` and `rhs` can be any
    /// combination of integer and float. `Expression::Multiplication(_, _, _)` is of type integer
    /// if both `lhs` and `rhs` are of type integer. Otherwise, it is of type float.
    ///
    /// `span` stores the expression's [`Span`].
    Multiplication(Box<Expression<V, S>>, Box<Expression<V, S>>, S),

    /// Division (x divided by y)
    ///
    /// # Construction
    ///
    /// A division expression can be constructed using [`Expression::divide_by()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::int(12).divide_by(Expression::float(3.5));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::divide_by_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::int(4).divide_by_spanned(Expression::int(5), FullSpan::from_range(3..6));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Division(lhs, rhs, _)` evaluates to `lhs` divided by `rhs`. Even if both `lhs`
    /// and `rhs` are of type integer, the result is of float. For example, 7/2 evaluates to 3.5.
    ///
    /// # Types
    ///
    /// In `Expression::Division(lhs, rhs, span)`, the types of `lhs` and `rhs` can be any
    /// combination of integer and float. `Expression::Division(_, _, _)` is always of type float.
    ///
    /// `span` stores the expression's [`Span`].
    Division(Box<Expression<V, S>>, Box<Expression<V, S>>, S),

    /// Addition (x plus y)
    ///
    /// # Construction
    ///
    /// An addition expression can be constructed using [`Expression::plus()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::int(12).plus(Expression::float(3.5));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::plus_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::int(4).plus_spanned(Expression::int(5), FullSpan::from_range(3..6));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Addition(lhs, rhs, _)` evaluates to the sum of `lhs` and `rhs`.
    ///
    /// # Types
    ///
    /// In `Expression::Addition(lhs, rhs, span)`, the types of `lhs` and `rhs` can be any
    /// combination of integer and float. `Expression::Addition(_, _, _)` is of type integer
    /// if both `lhs` and `rhs` are of type integer. Otherwise, it is of type float.
    ///
    /// `span` stores the expression's [`Span`].
    Addition(Box<Expression<V, S>>, Box<Expression<V, S>>, S),

    /// Subtraction (x minus y)
    ///
    /// # Construction
    ///
    /// A subtraction expression can be constructed using [`Expression::minus()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::int(12).minus(Expression::float(3.5));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::minus_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::int(4).minus_spanned(Expression::int(5), FullSpan::from_range(3..6));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Subtraction(lhs, rhs, _)` evaluates to `lhs` minus `rhs`.
    ///
    /// # Types
    ///
    /// In `Expression::Subtraction(lhs, rhs, span)`, the types of `lhs` and `rhs` can be any
    /// combination of integer and float. `Expression::Subtraction(_, _, _)` is of type integer
    /// if both `lhs` and `rhs` are of type integer. Otherwise, it is of type float.
    ///
    /// `span` stores the expression's [`Span`].
    Subtraction(Box<Expression<V, S>>, Box<Expression<V, S>>, S),

    /// Less-than comparison.
    ///
    /// # Construction
    ///
    /// A less-than comparison can be constructed using [`Expression::less_than()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::int(12).less_than(Expression::float(3.5));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::less_than_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::int(12).less_than_spanned(Expression::float(3.5), FullSpan::from_range(3..17));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::LessThan(lhs, rhs, _)` evaluates to `true` if `lhs` is strictly smaller than
    /// `rhs`. Otherwise, it evaluates to `false`.
    ///
    /// # Types
    ///
    /// In `Expression::LessThan(lhs, rhs, span)`, the types of `lhs` and `rhs` can be any combination
    /// of integer and float. `Expression::LessThan(_, _, _)` always has type boolean.
    ///
    /// `span` stores the expression's [`Span`].
    LessThan(Box<Expression<V, S>>, Box<Expression<V, S>>, S),

    /// Less-or-equal comparison.
    ///
    /// # Construction
    ///
    /// A less-or-equal comparison can be constructed using [`Expression::less_or_equal()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::int(12).less_or_equal(Expression::float(3.5));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::less_or_equal_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::int(12).less_or_equal_spanned(Expression::float(3.5), FullSpan::from_range(3..17));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::LessOrEqual(lhs, rhs, _)` evaluates to `true` if `lhs` is smaller than or equal
    /// to `rhs`. Otherwise, it evaluates to `false`.
    ///
    /// # Types
    ///
    /// In `Expression::LessOrEqual(lhs, rhs, span)`, the types of `lhs` and `rhs` can be any
    /// combination of integer and float. `Expression::LessOrEqual(_, _, _)` always has type boolean.
    ///
    /// `span` stores the expression's [`Span`].
    LessOrEqual(Box<Expression<V, S>>, Box<Expression<V, S>>, S),

    /// Greater-than comparison.
    ///
    /// # Construction
    ///
    /// A greater-than comparison can be constructed using [`Expression::greater_than()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::int(12).greater_than(Expression::float(3.5));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::greater_than_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::int(12).greater_than_spanned(Expression::float(3.5), FullSpan::from_range(3..17));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::GreaterThan(lhs, rhs, _)` evaluates to `true` if `lhs` is strictly greater than
    /// `rhs`. Otherwise, it evaluates to `false`.
    ///
    /// # Types
    ///
    /// In `Expression::GreaterThan(lhs, rhs, span)`, the types of `lhs` and `rhs` can be any
    /// combination of integer and float. `Expression::GreaterThan(_, _, _)` always has type boolean.
    ///
    /// `span` stores the expression's [`Span`].
    GreaterThan(Box<Expression<V, S>>, Box<Expression<V, S>>, S),

    /// Greater-or-equal comparison.
    ///
    /// # Construction
    ///
    /// A greater-or-equal comparison can be constructed using [`Expression::greater_or_equal()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::int(12).greater_or_equal(Expression::float(3.5));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::greater_or_equal_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::int(3).greater_or_equal_spanned(Expression::int(4), FullSpan::from_range(3..17));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::GreaterOrEqual(lhs, rhs, _)` evaluates to `true` if `lhs` is greater than or
    /// equal to `rhs`. Otherwise, it evaluates to `false`.
    ///
    /// # Types
    ///
    /// In `Expression::GreaterOrEqual(lhs, rhs, span)`, the types of `lhs` and `rhs` can be any
    /// combination of integer and float. `Expression::GreaterOrEqual(_, _, _)` always has type boolean.
    ///
    /// `span` stores the expression's [`Span`].
    GreaterOrEqual(Box<Expression<V, S>>, Box<Expression<V, S>>, S),

    /// Equality comparison.
    ///
    /// # Construction
    ///
    /// An equality comparison can be constructed using [`Expression::equals_to()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::bool(false).equals_to(Expression::bool(true));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::equals_to_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::int(3).equals_to_spanned(Expression::int(3), FullSpan::from_range(3..17));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Equals(lhs, rhs, _)` evaluates to `true` if `lhs` is equals to `rhs`.
    /// Otherwise, it evaluates to `false`. If the types of `lhs` or `rhs` are a combination of
    /// integer and float, both values are first converted to floats and then compared. For example,
    /// the following expression evaluates to `true`:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::int(3).equals_to(Expression::float(3.0));
    /// ```
    ///
    /// # Types
    ///
    /// In `Expression::Equals(lhs, rhs, span)`, the types of `lhs` and `rhs` can either both be
    /// boolean or any combination of integer and float. `Expression::Equals(_, _, _)` always
    /// has type boolean.
    ///
    /// `span` stores the expression's [`Span`].
    Equals(Box<Expression<V, S>>, Box<Expression<V, S>>, S),

    /// Inequality comparison.
    ///
    /// # Construction
    ///
    /// An inequality comparison can be constructed using [`Expression::not_equals_to()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::bool(false).not_equals_to(Expression::bool(true));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::not_equals_to_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::int(3).not_equals_to_spanned(Expression::int(3), FullSpan::from_range(3..17));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::NotEquals(lhs, rhs, _)` evaluates to `true` if `lhs` is not equals to `rhs`.
    /// Otherwise, it evaluates to `false`. If the types of `lhs` or `rhs` are a combination of
    /// integer and float, both values are first converted to floats and then compared. For example,
    /// the following expression evaluates to `false`:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::int(3).not_equals_to(Expression::float(3.0));
    /// ```
    ///
    /// # Types
    ///
    /// In `Expression::NotEquals(lhs, rhs, span)`, the types of `lhs` and `rhs` can either both be
    /// boolean or any combination of integer and float. `Expression::NotEquals(_, _, _)` always
    /// has type boolean.
    ///
    /// `span` stores the expression's [`Span`].
    NotEquals(Box<Expression<V, S>>, Box<Expression<V, S>>, S),

    /// Boolean negation.
    ///
    /// # Construction
    ///
    /// A boolean negation can be constructed using [`Expression::negate_bool()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::bool(false).negate_bool();
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::negate_bool_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::bool_spanned(true, FullSpan::from_range(4..8))
    ///     .negate_bool_spanned(FullSpan::from_range(3..8));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Negation(inner, _)` evaluates to `true` if `inner` is `false`. Otherwise, it
    /// evaluates to `false`.
    ///
    /// # Types
    ///
    /// In `Expression::Negation(inner, span)`, `inner` must be of type boolean.
    /// `Expression::Negation(_, _)` is of type boolean.
    ///
    /// `span` stores the expression's [`Span`].
    Negation(Box<Expression<V, S>>, S),

    /// Boolean conjunction ("x and y").
    ///
    /// # Construction
    ///
    /// A boolean conjunction can be constructed using [`Expression::and()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::bool(false).and(Expression::bool(true));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::and_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::bool(true).and_spanned(Expression::bool(false), FullSpan::from_range(3..17));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Conjunction(lhs, rhs, _)` evaluates to `true` if `lhs` and `rhs` both evaluate
    /// to `true`. Otherwise, it returns `false`.
    ///
    /// # Types
    ///
    /// In `Expression::Conjunction(lhs, rhs, span)`, both `lhs` and `rhs` must be of type boolean.
    /// `Expression::Conjunction(_, _, _)` always has type boolean.
    ///
    /// `span` stores the expression's [`Span`].
    Conjunction(Box<Expression<V, S>>, Box<Expression<V, S>>, S),

    /// Boolean disjunction ("x or y").
    ///
    /// # Construction
    ///
    /// A boolean disjunction can be constructed using [`Expression::or()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::bool(false).or(Expression::bool(true));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::or_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::bool(true).or_spanned(Expression::bool(false), FullSpan::from_range(3..17));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Disjunction(lhs, rhs, _)` evaluates to `true` if `lhs` or `rhs` (or both)
    /// evaluate to `true`. Otherwise, it returns `false`.
    ///
    /// # Types
    ///
    /// In `Expression::Disjunction(lhs, rhs, span)`, both `lhs` and `rhs` must be of type boolean.
    /// `Expression::Disjunction(_, _, _)` always has type boolean.
    ///
    /// `span` stores the expression's [`Span`].
    Disjunction(Box<Expression<V, S>>, Box<Expression<V, S>>, S),

    /// Boolean equivalence ("x if and only if y").
    ///
    /// # Construction
    ///
    /// A boolean equivalence can be constructed using [`Expression::if_and_only_if()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::bool(false).if_and_only_if(Expression::bool(true));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::if_and_only_if_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::bool(true).if_and_only_if_spanned(Expression::bool(false), FullSpan::from_range(3..17));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::IfAndOnlyIf(lhs, rhs, _)` evaluates to `true` if `lhs` and `rhs` evaluate to the
    /// same value. Otherwise, it returns `false`.
    ///
    /// This is equivalent to the semantics of [`Expression::Equals`], restricted to booleans.
    ///
    /// # Types
    ///
    /// In `Expression::IfAndOnlyIf(lhs, rhs, span)`, both `lhs` and `rhs` must be of type boolean.
    /// `Expression::IfAndOnlyIf(_, _, _)` always has type boolean.
    ///
    /// `span` stores the expression's [`Span`].
    IfAndOnlyIf(Box<Expression<V, S>>, Box<Expression<V, S>>, S),

    /// Boolean implication ("x implies y").
    ///
    /// # Construction
    ///
    /// A boolean implication can be constructed using [`Expression::implies()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::bool(false).implies(Expression::bool(true));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::implies_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::bool(true).implies_spanned(Expression::bool(false), FullSpan::from_range(3..17));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Implies(lhs, rhs, _)` evaluates to `true` if `lhs` evaluates to `false` or
    /// `rhs` evaluates to `true`. Otherwise, it returns `false`.
    ///
    /// This can equivalently be expressed as `(!lhs) | rhs`.
    ///
    /// # Types
    ///
    /// In `Expression::Implies(lhs, rhs, span)`, both `lhs` and `rhs` must be of type boolean.
    /// `Expression::Implies(_, _, _)` always has type boolean.
    ///
    /// `span` stores the expression's [`Span`].
    Implies(Box<Expression<V, S>>, Box<Expression<V, S>>, S),

    /// Ternary operator ("if x, then y, otherwise z").
    ///
    /// # Construction
    ///
    /// A ternary expression can be constructed using [`Expression::ternary()`]:
    ///
    /// ```
    /// # use prism_model::Expression;
    /// # let ex: Expression =
    /// Expression::bool(false).ternary(Expression::int(3), Expression::float(3.2));
    /// ```
    ///
    /// To include a [`Span`], use [`Expression::ternary_spanned()`]:
    ///
    /// ```
    /// # use prism_model::{Expression, FullSpan, Span};
    /// # let ex: Expression =
    /// Expression::bool(false)
    ///     .ternary_spanned(Expression::int(3), Expression::int(5), FullSpan::from_range(3..17));
    /// ```
    ///
    /// # Evaluation
    ///
    /// `Expression::Ternary(cond, lhs, rhs, _)` evaluates to `lhs` if `cond` evaluates to `true`.
    /// Otherwise, it evaluates to `rhs`.
    ///
    /// # Types
    ///
    /// In `Expression::Ternary(cond, lhs, rhs, span)`, `cond` must be of type boolean. `lhs` and
    /// `rhs` must be of compatible types (i.e. either both are of type boolean or their types are
    /// a combination of integers and floats).
    ///
    /// If `lhs` and `rhs` are of type boolean, then `Expression::Ternary(_, _, _, _)` is of type
    /// boolean. If both `lhs` and `rhs` are of type integer, then `Expression::Ternary(_, _, _, _)`
    /// is of type integer. Otherwise, it is of type float.
    ///
    /// `span` stores the expression's [`Span`].
    Ternary(
        Box<Expression<V, S>>,
        Box<Expression<V, S>>,
        Box<Expression<V, S>>,
        S,
    ),
}

impl<V, S: Span> Expression<V, S> {
    /// Returns the span of the expression.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::*;
    /// let expr: Expression = Expression::int_spanned(123, FullSpan::from_range(10..13));
    /// let span = expr.span();
    /// assert_eq!(span.start(), Some(10));
    /// assert_eq!(span.end(), Some(13));
    /// ```
    pub fn span(&self) -> &S {
        match self {
            Expression::Int(_, s) => s,
            Expression::Float(_, s) => s,
            Expression::Bool(_, s) => s,
            Expression::VarOrConst(_, s) => s,
            Expression::Label(_, s) => s,
            Expression::Function(_, _, s) => s,
            Expression::Minus(_, s) => s,
            Expression::Multiplication(_, _, s) => s,
            Expression::Division(_, _, s) => s,
            Expression::Addition(_, _, s) => s,
            Expression::Subtraction(_, _, s) => s,
            Expression::LessThan(_, _, s) => s,
            Expression::LessOrEqual(_, _, s) => s,
            Expression::GreaterThan(_, _, s) => s,
            Expression::GreaterOrEqual(_, _, s) => s,
            Expression::Equals(_, _, s) => s,
            Expression::NotEquals(_, _, s) => s,
            Expression::Negation(_, s) => s,
            Expression::Conjunction(_, _, s) => s,
            Expression::Disjunction(_, _, s) => s,
            Expression::IfAndOnlyIf(_, _, s) => s,
            Expression::Implies(_, _, s) => s,
            Expression::Ternary(_, _, _, s) => s,
        }
    }

    /// Maps the span of the expression to another value of (potentially) different type.
    ///
    /// # Example
    ///
    /// Offset every span by 1:
    ///
    /// ```
    /// # use prism_model::*;
    /// let expr_1: Expression = Expression::bool_spanned(true, FullSpan::from_range(4..8));
    /// let expr_2: Expression = expr_1.map_span(&| s| {
    ///     FullSpan::from_range(s.start().unwrap() + 1.. s.end().unwrap() + 1)
    /// });
    /// assert_eq!(expr_2.span(), &FullSpan::from_range(5..9));
    /// ```
    ///
    /// This function can also be used to erase the span:
    ///
    /// ```
    /// # use prism_model::*;
    /// # let expr_1: Expression = Expression::bool_spanned(true, FullSpan::from_range(4..8));
    /// # let expr_2: Expression = expr_1.map_span(&| s| {
    /// #     FullSpan::from_range(s.start().unwrap() + 1.. s.end().unwrap() + 1)
    /// # });
    /// let expr_3 = expr_2.map_span(&|_| ());
    /// assert_eq!(expr_3.span(), &());
    /// ```
    ///
    /// # Notes
    ///
    /// This operation is eager, i.e. it directly computes the new expression.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> Expression<V, S2> {
        let mut visitor = maps::map_span::MapSpan::new(map);
        self.visit(&mut visitor)
    }

    /// Maps every variable reference in the expression to another value of (potentially) different
    /// type.
    ///
    /// This is the general-purpose variable-transformation method. For the common case of
    /// converting an [`Expression<Identifier>`] into an [`Expression<VariableReference>`],
    /// prefer [`Expression::replace_identifiers_by_variable_indices()`], which additionally
    /// reports unknown variables as errors.
    ///
    /// # Example
    ///
    /// Append `"_var"` to every variable name:
    ///
    /// ```
    /// # use prism_model::*;
    /// let e: Expression<Identifier> = Expression::var_or_const(Identifier::new("x").unwrap())
    ///     .plus(Expression::var_or_const(Identifier::new("y").unwrap()));
    ///
    /// let shifted: Expression<Identifier>
    ///     = e.map_variable(&|r| Identifier::new(format!("{}_var", r.name)).unwrap());
    /// ```
    ///
    /// # Notes
    ///
    /// This operation is eager, i.e. it directly computes the new expression.
    pub fn map_variable<V2, F: Fn(V) -> V2>(self, map: &F) -> Expression<V2, S> {
        let mut visitor = maps::map_variable::MapVariable::new(|v, _| map(v), ());
        self.visit(&mut visitor)
    }

    /// Returns the precedence of this expression as defined by PRISM [1].
    ///
    /// Higher values bind more tightly. The scale runs from 1 (loosest binding) to 12
    /// (tightest binding). The value 0 is not returned by this method; it can be used to indicate
    /// that there is no surrounding expression (to prevent parentheses from being inserted at the
    /// top level).
    ///
    /// # Usage
    ///
    /// The function can be used to insert parentheses only when necessary. For surrounding
    /// expression `e_outer` and inner expression `e_inner`, parentheses around `e_inner` are only
    /// required when `e_outer.get_precedence() >= e_inner.get_precedence()`.
    ///
    /// # Details
    ///
    /// | Precedence | Operators |
    /// |------------|-----------|
    /// | 12 | atoms: literals, variables, labels, function calls |
    /// | 11 | unary minus (`-x`) |
    /// | 10 | `*`, `/` |
    /// | 9 | `+`, `-` |
    /// | 8 | `<`, `<=`, `>`, `>=` |
    /// | 7 | `=`, `!=` |
    /// | 6 | boolean negation (`!`) |
    /// | 5 | `&` |
    /// | 4 | `\|` |
    /// | 3 | `<=>` |
    /// | 2 | `=>` |
    /// | 1 | ternary `? :` |
    ///
    /// [1]: https://prismmodelchecker.org/manual/ThePRISMLanguage/Expressions
    pub fn get_precedence(&self) -> usize {
        match self {
            Expression::Int(_, _) => 12,
            Expression::Float(_, _) => 12,
            Expression::Bool(_, _) => 12,
            Expression::VarOrConst(_, _) => 12,
            Expression::Label(_, _) => 12,
            Expression::Function(_, _, _) => 12,
            Expression::Minus(_, _) => 11,
            Expression::Multiplication(_, _, _) => 10,
            Expression::Division(_, _, _) => 10,
            Expression::Addition(_, _, _) => 9,
            Expression::Subtraction(_, _, _) => 9,
            Expression::LessThan(_, _, _) => 8,
            Expression::LessOrEqual(_, _, _) => 8,
            Expression::GreaterThan(_, _, _) => 8,
            Expression::GreaterOrEqual(_, _, _) => 8,
            Expression::Equals(_, _, _) => 7,
            Expression::NotEquals(_, _, _) => 7,
            Expression::Negation(_, _) => 6,
            Expression::Conjunction(_, _, _) => 5,
            Expression::Disjunction(_, _, _) => 4,
            Expression::IfAndOnlyIf(_, _, _) => 3,
            Expression::Implies(_, _, _) => 2,
            Expression::Ternary(_, _, _, _) => 1,
        }
    }
}

impl<V, S: Span> Expression<V, S> {
    /// Constructs an integer literal expression with empty span.
    ///
    /// For details, see [`Expression::Int`].
    ///
    /// To construct an integer literal expression with custom [`Span`], use
    /// [`Expression::int_spanned()`].
    pub fn int(val: i64) -> Self {
        Expression::Int(val, S::empty())
    }

    /// Constructs an integer literal expression with given [`Span`].
    ///
    /// For details, see [`Expression::Int`].
    ///
    /// To construct an integer literal expression with empty span, use [`Expression::int()`].
    pub fn int_spanned(val: i64, span: S) -> Self {
        Expression::Int(val, span)
    }
    /// Constructs a floating-point literal expression with empty span.
    ///
    /// For details, see [`Expression::Float`].
    ///
    /// To construct a floating-point literal expression with custom [`Span`], use
    /// [`Expression::float_spanned()`].
    pub fn float(val: f64) -> Self {
        Expression::Float(val, S::empty())
    }

    /// Constructs a floating-point literal expression with given [`Span`].
    ///
    /// For details, see [`Expression::Float`].
    ///
    /// To construct a floating-point literal expression with empty span, use
    /// [`Expression::float()`].
    pub fn float_spanned(val: f64, span: S) -> Self {
        Expression::Float(val, span)
    }

    /// Constructs a boolean literal expression with empty span.
    ///
    /// For details, see [`Expression::Bool`].
    ///
    /// To construct a boolean literal expression with custom [`Span`], use
    /// [`Expression::bool_spanned()`].
    pub fn bool(val: bool) -> Self {
        Expression::Bool(val, S::empty())
    }

    /// Constructs a boolean literal expression with given [`Span`].
    ///
    /// For details, see [`Expression::Bool`].
    ///
    /// To construct a boolean literal expression with empty span, use [`Expression::bool()`].
    pub fn bool_spanned(val: bool, span: S) -> Self {
        Expression::Bool(val, span)
    }

    /// Constructs a variable, constant, or formula reference expression with empty span.
    ///
    /// For details, see [`Expression::VarOrConst`].
    ///
    /// To construct a variable reference expression with custom [`Span`], use
    /// [`Expression::var_or_const_spanned()`].
    pub fn var_or_const(id: V) -> Self {
        Expression::VarOrConst(id, S::empty())
    }

    /// Constructs a variable, constant, or formula reference expression with given [`Span`].
    ///
    /// For details, see [`Expression::VarOrConst`].
    ///
    /// To construct a variable reference expression with empty span, use
    /// [`Expression::var_or_const()`].
    pub fn var_or_const_spanned(id: V, span: S) -> Self {
        Expression::VarOrConst(id, span)
    }

    /// Constructs a label expression with empty span.
    ///
    /// For details, see [`Expression::Label`].
    ///
    /// To construct a label expression with custom [`Span`], use [`Expression::label_spanned()`].
    pub fn label(id: V) -> Self {
        Expression::Label(id, S::empty())
    }

    /// Constructs a label expression with given [`Span`].
    ///
    /// For details, see [`Expression::Label`].
    ///
    /// To construct a label expression with empty span, use [`Expression::label()`].
    pub fn label_spanned(id: V, span: S) -> Self {
        Expression::Label(id, span)
    }

    /// Constructs a function call expression with empty span.
    ///
    /// For details, see [`Expression::Function`].
    ///
    /// To construct a function call expression with custom [`Span`], use
    /// [`Expression::function_spanned()`].
    pub fn function<A: Into<Vec<Self>>>(identifier: Identifier<S>, args: A) -> Self {
        Expression::Function(identifier, args.into(), S::empty())
    }

    /// Constructs a function call expression with given [`Span`].
    ///
    /// For details, see [`Expression::Function`].
    ///
    /// To construct a function call expression with empty span, use [`Expression::function()`].
    pub fn function_spanned<A: Into<Vec<Self>>>(
        identifier: Identifier<S>,
        args: A,
        span: S,
    ) -> Self {
        Expression::Function(identifier, args.into(), span)
    }

    /// Constructs an integer and float negation expression with empty span.
    ///
    /// For details, see [`Expression::Minus`].
    ///
    /// To construct a negation expression with custom [`Span`], use
    /// [`Expression::negate_value_spanned()`].
    pub fn negate_value(self) -> Self {
        Expression::Minus(Box::new(self), S::empty())
    }

    /// Constructs an integer and float negation expression with given [`Span`].
    ///
    /// For details, see [`Expression::Minus`].
    ///
    /// To construct a negation expression with empty span, use [`Expression::negate_value()`].
    pub fn negate_value_spanned(self, span: S) -> Self {
        Expression::Minus(Box::new(self), span)
    }

    /// Constructs a multiplication expression with empty span.
    ///
    /// For details, see [`Expression::Multiplication`].
    ///
    /// To construct a multiplication expression with custom [`Span`], use
    /// [`Expression::times_spanned()`].
    pub fn times(self, other: Self) -> Self {
        Expression::Multiplication(Box::new(self), Box::new(other), S::empty())
    }

    /// Constructs a multiplication expression with given [`Span`].
    ///
    /// For details, see [`Expression::Multiplication`].
    ///
    /// To construct a multiplication expression with empty span, use [`Expression::times()`].
    pub fn times_spanned(self, other: Self, span: S) -> Self {
        Expression::Multiplication(Box::new(self), Box::new(other), span)
    }

    /// Constructs a division expression with empty span.
    ///
    /// For details, see [`Expression::Division`].
    ///
    /// To construct a division expression with custom [`Span`], use
    /// [`Expression::divide_by_spanned()`].
    pub fn divide_by(self, other: Self) -> Self {
        Expression::Division(Box::new(self), Box::new(other), S::empty())
    }

    /// Constructs a division expression with given [`Span`].
    ///
    /// For details, see [`Expression::Division`].
    ///
    /// To construct a division expression with empty span, use [`Expression::divide_by()`].
    pub fn divide_by_spanned(self, other: Self, span: S) -> Self {
        Expression::Division(Box::new(self), Box::new(other), span)
    }

    /// Constructs an addition expression with empty span.
    ///
    /// For details, see [`Expression::Addition`].
    ///
    /// To construct an addition expression with custom [`Span`], use
    /// [`Expression::plus_spanned()`].
    pub fn plus(self, other: Self) -> Self {
        Expression::Addition(Box::new(self), Box::new(other), S::empty())
    }

    /// Constructs an addition expression with given [`Span`].
    ///
    /// For details, see [`Expression::Addition`].
    ///
    /// To construct an addition expression with empty span, use [`Expression::plus()`].
    pub fn plus_spanned(self, other: Self, span: S) -> Self {
        Expression::Addition(Box::new(self), Box::new(other), span)
    }

    /// Constructs a subtraction expression with empty span.
    ///
    /// For details, see [`Expression::Subtraction`].
    ///
    /// To construct a subtraction expression with custom [`Span`], use
    /// [`Expression::minus_spanned()`].
    pub fn minus(self, other: Self) -> Self {
        Expression::Subtraction(Box::new(self), Box::new(other), S::empty())
    }

    /// Constructs a subtraction expression with given [`Span`].
    ///
    /// For details, see [`Expression::Subtraction`].
    ///
    /// To construct a subtraction expression with empty span, use [`Expression::minus()`].
    pub fn minus_spanned(self, other: Self, span: S) -> Self {
        Expression::Subtraction(Box::new(self), Box::new(other), span)
    }

    /// Constructs a less-than comparison expression with empty span.
    ///
    /// For details, see [`Expression::LessThan`].
    ///
    /// To construct a less-than comparison expression with custom [`Span`], use
    /// [`Expression::less_than_spanned()`].
    pub fn less_than(self, other: Self) -> Self {
        Expression::LessThan(Box::new(self), Box::new(other), S::empty())
    }

    /// Constructs a less-than comparison expression with given [`Span`].
    ///
    /// For details, see [`Expression::LessThan`].
    ///
    /// To construct a less-than comparison expression with empty span, use
    /// [`Expression::less_than()`].
    pub fn less_than_spanned(self, other: Self, span: S) -> Self {
        Expression::LessThan(Box::new(self), Box::new(other), span)
    }

    /// Constructs a less-or-equal comparison expression with empty span.
    ///
    /// For details, see [`Expression::LessOrEqual`].
    ///
    /// To construct a less-or-equal comparison expression with custom [`Span`], use
    /// [`Expression::less_or_equal_spanned()`].
    pub fn less_or_equal(self, other: Self) -> Self {
        Expression::LessOrEqual(Box::new(self), Box::new(other), S::empty())
    }

    /// Constructs a less-or-equal comparison expression with given [`Span`].
    ///
    /// For details, see [`Expression::LessOrEqual`].
    ///
    /// To construct a less-or-equal comparison expression with empty span, use
    /// [`Expression::less_or_equal()`].
    pub fn less_or_equal_spanned(self, other: Self, span: S) -> Self {
        Expression::LessOrEqual(Box::new(self), Box::new(other), span)
    }

    /// Constructs a greater-than comparison expression with empty span.
    ///
    /// For details, see [`Expression::GreaterThan`].
    ///
    /// To construct a greater-than comparison expression with custom [`Span`], use
    /// [`Expression::greater_than_spanned()`].
    pub fn greater_than(self, other: Self) -> Self {
        Expression::GreaterThan(Box::new(self), Box::new(other), S::empty())
    }

    /// Constructs a greater-than comparison expression with given [`Span`].
    ///
    /// For details, see [`Expression::GreaterThan`].
    ///
    /// To construct a greater-than comparison expression with empty span, use
    /// [`Expression::greater_than()`].
    pub fn greater_than_spanned(self, other: Self, span: S) -> Self {
        Expression::GreaterThan(Box::new(self), Box::new(other), span)
    }

    /// Constructs a greater-or-equal comparison expression with empty span.
    ///
    /// For details, see [`Expression::GreaterOrEqual`].
    ///
    /// To construct a greater-or-equal comparison expression with custom [`Span`], use
    /// [`Expression::greater_or_equal_spanned()`].
    pub fn greater_or_equal(self, other: Self) -> Self {
        Expression::GreaterOrEqual(Box::new(self), Box::new(other), S::empty())
    }

    /// Constructs a greater-or-equal comparison expression with given [`Span`].
    ///
    /// For details, see [`Expression::GreaterOrEqual`].
    ///
    /// To construct a greater-or-equal comparison expression with empty span, use
    /// [`Expression::greater_or_equal()`].
    pub fn greater_or_equal_spanned(self, other: Self, span: S) -> Self {
        Expression::GreaterOrEqual(Box::new(self), Box::new(other), span)
    }

    /// Constructs an equality comparison expression with empty span.
    ///
    /// For details, see [`Expression::Equals`].
    ///
    /// To construct an equality comparison expression with custom [`Span`], use
    /// [`Expression::equals_to_spanned()`].
    pub fn equals_to(self, other: Self) -> Self {
        Expression::Equals(Box::new(self), Box::new(other), S::empty())
    }

    /// Constructs an equality comparison expression with given [`Span`].
    ///
    /// For details, see [`Expression::Equals`].
    ///
    /// To construct an equality comparison expression with empty span, use
    /// [`Expression::equals_to()`].
    pub fn equals_to_spanned(self, other: Self, span: S) -> Self {
        Expression::Equals(Box::new(self), Box::new(other), span)
    }

    /// Constructs an inequality comparison expression with empty span.
    ///
    /// For details, see [`Expression::NotEquals`].
    ///
    /// To construct an inequality comparison expression with custom [`Span`], use
    /// [`Expression::not_equals_to_spanned()`].
    pub fn not_equals_to(self, other: Self) -> Self {
        Expression::NotEquals(Box::new(self), Box::new(other), S::empty())
    }

    /// Constructs an inequality comparison expression with given [`Span`].
    ///
    /// For details, see [`Expression::NotEquals`].
    ///
    /// To construct an inequality comparison expression with empty span, use
    /// [`Expression::not_equals_to()`].
    pub fn not_equals_to_spanned(self, other: Self, span: S) -> Self {
        Expression::NotEquals(Box::new(self), Box::new(other), span)
    }

    /// Constructs a boolean negation expression with empty span.
    ///
    /// For details, see [`Expression::Negation`].
    ///
    /// To construct a boolean negation expression with custom [`Span`], use
    /// [`Expression::negate_bool_spanned()`].
    pub fn negate_bool(self) -> Self {
        Expression::Negation(Box::new(self), S::empty())
    }

    /// Constructs a boolean negation expression with given [`Span`].
    ///
    /// For details, see [`Expression::Negation`].
    ///
    /// To construct a boolean negation expression with empty span, use
    /// [`Expression::negate_bool()`].
    pub fn negate_bool_spanned(self, span: S) -> Self {
        Expression::Negation(Box::new(self), span)
    }

    /// Constructs a boolean conjunction expression with empty span.
    ///
    /// For details, see [`Expression::Conjunction`].
    ///
    /// To construct a boolean conjunction expression with custom [`Span`], use
    /// [`Expression::and_spanned()`].
    pub fn and(self, other: Self) -> Self {
        Expression::Conjunction(Box::new(self), Box::new(other), S::empty())
    }

    /// Constructs a boolean conjunction expression with given [`Span`].
    ///
    /// For details, see [`Expression::Conjunction`].
    ///
    /// To construct a boolean conjunction expression with empty span, use [`Expression::and()`].
    pub fn and_spanned(self, other: Self, span: S) -> Self {
        Expression::Conjunction(Box::new(self), Box::new(other), span)
    }

    /// Constructs a boolean disjunction expression with empty span.
    ///
    /// For details, see [`Expression::Disjunction`].
    ///
    /// To construct a boolean disjunction expression with custom [`Span`], use
    /// [`Expression::or_spanned()`].
    pub fn or(self, other: Self) -> Self {
        Expression::Disjunction(Box::new(self), Box::new(other), S::empty())
    }

    /// Constructs a boolean disjunction expression with given [`Span`].
    ///
    /// For details, see [`Expression::Disjunction`].
    ///
    /// To construct a boolean disjunction expression with empty span, use [`Expression::or()`].
    pub fn or_spanned(self, other: Self, span: S) -> Self {
        Expression::Disjunction(Box::new(self), Box::new(other), span)
    }

    /// Constructs a boolean equivalence expression with empty span.
    ///
    /// For details, see [`Expression::IfAndOnlyIf`].
    ///
    /// To construct a boolean equivalence expression with custom [`Span`], use
    /// [`Expression::if_and_only_if_spanned()`].
    pub fn if_and_only_if(self, other: Self) -> Self {
        Expression::IfAndOnlyIf(Box::new(self), Box::new(other), S::empty())
    }

    /// Constructs a boolean equivalence expression with given [`Span`].
    ///
    /// For details, see [`Expression::IfAndOnlyIf`].
    ///
    /// To construct a boolean equivalence expression with empty span, use
    /// [`Expression::if_and_only_if()`].
    pub fn if_and_only_if_spanned(self, other: Self, span: S) -> Self {
        Expression::IfAndOnlyIf(Box::new(self), Box::new(other), span)
    }

    /// Constructs a boolean implication expression with empty span.
    ///
    /// For details, see [`Expression::Implies`].
    ///
    /// To construct a boolean implication expression with custom [`Span`], use
    /// [`Expression::implies_spanned()`].
    pub fn implies(self, other: Self) -> Self {
        Expression::Implies(Box::new(self), Box::new(other), S::empty())
    }

    /// Constructs a boolean implication expression with given [`Span`].
    ///
    /// For details, see [`Expression::Implies`].
    ///
    /// To construct a boolean implication expression with empty span, use
    /// [`Expression::implies()`].
    pub fn implies_spanned(self, other: Self, span: S) -> Self {
        Expression::Implies(Box::new(self), Box::new(other), span)
    }

    /// Constructs a ternary expression with empty span.
    ///
    /// For details, see [`Expression::Ternary`].
    ///
    /// To construct a ternary expression with custom [`Span`], use
    /// [`Expression::ternary_spanned()`].
    pub fn ternary(self, branch_1: Self, branch_2: Self) -> Self {
        Expression::Ternary(
            Box::new(self),
            Box::new(branch_1),
            Box::new(branch_2),
            S::empty(),
        )
    }

    /// Constructs a ternary expression with given [`Span`].
    ///
    /// For details, see [`Expression::Ternary`].
    ///
    /// To construct a ternary expression with empty span, use [`Expression::ternary()`].
    pub fn ternary_spanned(self, branch_1: Self, branch_2: Self, span: S) -> Self {
        Expression::Ternary(Box::new(self), Box::new(branch_1), Box::new(branch_2), span)
    }
}

impl<S: Span> Expression<Identifier<S>, S> {
    /// Replaces every [`Expression::Label`] in this expression whose name matches a label in
    /// `labels` with that label's condition expression.
    ///
    /// Label occurrences that have no matching entry in `labels` are left unchanged as
    /// [`Expression::Label`] nodes.
    ///
    /// # Example
    ///
    /// Let `labels` be a [`LabelManager`] with a label `"done"` with value `true`.
    /// ```
    /// # use prism_model::*;
    /// let done = Identifier::new("done").unwrap();
    /// let labels: LabelManagerNamedVars = LabelManager::with_labels(vec![
    ///     Label::new(done.clone(), Expression::bool(true)),
    /// ]).unwrap();
    /// ```
    ///
    /// Let `expr` be the expression `"done" & false`.
    ///
    /// ```
    /// # use prism_model::*;
    /// # let done = Identifier::new("done").unwrap();
    /// # let labels: LabelManagerNamedVars = LabelManager::with_labels(vec![
    /// #     Label::new(done.clone(), Expression::bool(true)),
    /// # ]).unwrap();
    /// let mut expr = Expression::label(done).and(Expression::bool(false));
    /// # expr.substitute_labels(&labels);
    /// # assert_eq!(expr, Expression::bool(true).and(Expression::bool(false)));
    /// ```
    ///
    /// Then calling `substitute_labels()` yields the expression `true & false`.
    ///
    /// ```
    /// # use prism_model::*;
    /// # let done = Identifier::new("done").unwrap();
    /// # let labels: LabelManagerNamedVars = LabelManager::with_labels(vec![
    /// #     Label::new(done.clone(), Expression::bool(true)),
    /// # ]).unwrap();
    /// # let mut expr = Expression::label(done).and(Expression::bool(false));
    /// expr.substitute_labels(&labels);
    /// assert_eq!(expr, Expression::bool(true).and(Expression::bool(false)));
    /// ```
    pub fn substitute_labels(&mut self, labels: &LabelManager<S, Expression<Identifier<S>, S>>) {
        for label in &labels.labels {
            let mut visitor = LabelSubstitutionVisitor {
                label_name: &label.name,
                expression: &label.condition,
            };

            let condition = std::mem::replace(self, Expression::Bool(false, S::empty()));
            *self = condition.visit(&mut visitor);
        }
    }

    /// Replaces every [`Expression::VarOrConst`] in this expression whose name matches a formula
    /// in `formulas` with that formula's condition expression.
    ///
    /// Formulas may reference other formulas. This method expands formulas according to the
    /// topological ordering of the formula dependency graph. This ensures nested formulas are fully
    /// expanded.
    ///
    /// # Errors
    ///
    /// Returns [`CyclicDependency`] if the formula dependency graph contains a cycle, i.e. a
    /// formula (directly or transitively) references itself.
    ///
    /// # Example
    ///
    /// Let `formulas` be a [`FormulaManager`] with formulas `thresh = 10` and
    /// `almost_max = threshold - 1`:
    ///
    /// ```
    /// # use prism_model::*;
    /// let thresh: Identifier = Identifier::new("thresh").unwrap();
    /// let almost: Identifier = Identifier::new("almost").unwrap();
    /// let mut formulas: FormulaManagerNamedVars = FormulaManager::with_formulas(vec![
    ///     Formula::new(thresh.clone(), Expression::int(10)),
    ///     Formula::new(almost.clone(), Expression::var_or_const(thresh.clone()).minus(Expression::int(1))),
    /// ]).unwrap();
    /// ```
    ///
    /// Let `expr = x > almost`:
    ///
    /// ```
    /// # use prism_model::*;
    /// # let thresh: Identifier = Identifier::new("thresh").unwrap();
    /// # let almost: Identifier = Identifier::new("almost").unwrap();
    /// # let mut formulas: FormulaManagerNamedVars = FormulaManager::with_formulas(vec![
    /// #     Formula::new(thresh.clone(), Expression::int(10)),
    /// #     Formula::new(almost.clone(), Expression::var_or_const(thresh.clone()).minus(Expression::int(1))),
    /// # ]).unwrap();
    /// # let x: Identifier = Identifier::new("x").unwrap();
    /// let mut expr = Expression::var_or_const(x.clone()).greater_than(Expression::var_or_const(almost));
    /// # expr.substitute_formulas(&formulas).unwrap();
    /// # assert_eq!(
    /// #     expr,
    /// #     Expression::var_or_const(x).greater_than(Expression::int(10).minus(Expression::int(1)))
    /// # );
    /// ```
    ///
    /// Then calling `substitute_formulas()` yields `expr = x > 10 - 1`:
    ///
    /// ```
    /// # use prism_model::*;
    /// # let thresh: Identifier = Identifier::new("thresh").unwrap();
    /// # let almost: Identifier = Identifier::new("almost").unwrap();
    /// # let mut formulas: FormulaManagerNamedVars = FormulaManager::with_formulas(vec![
    /// #     Formula::new(thresh.clone(), Expression::int(10)),
    /// #     Formula::new(almost.clone(), Expression::var_or_const(thresh.clone()).minus(Expression::int(1))),
    /// # ]).unwrap();
    /// # let x: Identifier = Identifier::new("x").unwrap();
    /// # let mut expr = Expression::var_or_const(x.clone()).greater_than(Expression::var_or_const(almost));
    /// expr.substitute_formulas(&formulas).expect("Circular dependency");
    /// assert_eq!(
    ///     expr,
    ///     Expression::var_or_const(x).greater_than(Expression::int(10).minus(Expression::int(1)))
    /// );
    /// ```
    pub fn substitute_formulas(
        &mut self,
        formulas: &FormulaManager<S, Expression<Identifier<S>, S>>,
    ) -> Result<(), CyclicDependency<S>> {
        let order = formulas.get_formula_replacement_ordering()?;

        for formula_index in order {
            let formula = formulas.get(formula_index).unwrap();
            let mut visitor = crate::model::FormulaSubstitutionVisitor {
                formula_name: &formula.name,
                expression: &formula.condition,
            };

            let condition = std::mem::replace(self, Expression::Bool(false, S::empty()));
            *self = condition.visit(&mut visitor);
        }

        Ok(())
    }

    /// Renames the variables in the expression according to the given [`RenameRules`].
    ///
    /// This function is mainly used to expand a [`crate::RenamedModule`].
    ///
    /// # Example
    ///
    /// These rename rules swap `x` and `y`:
    ///
    /// ```
    /// # use prism_model::*;
    /// let x = Identifier::new("x").unwrap();
    /// let y = Identifier::new("y").unwrap();
    /// let rename_rules: RenameRules = RenameRules::with_rules(&[
    ///     RenameRule::new(x.clone(), y.clone()),
    ///     RenameRule::new(y.clone(), x.clone()),
    /// ]);
    /// ```
    /// Given the expression `x / y`, calling `renamed()` produces the expression `y / x`:
    ///
    /// ```
    /// # use prism_model::*;
    /// # let x = Identifier::new("x").unwrap();
    /// # let y = Identifier::new("y").unwrap();
    /// # let rename_rules: RenameRules = RenameRules::with_rules(&[
    /// #     RenameRule::new(x.clone(), y.clone()),
    /// #     RenameRule::new(y.clone(), x.clone()),
    /// # ]);
    /// let x_by_y = Expression::var_or_const(x.clone())
    ///     .divide_by(Expression::var_or_const(y.clone()));
    /// let y_by_x = x_by_y.renamed(&rename_rules);
    /// assert_eq!(y_by_x, Expression::var_or_const(y).divide_by(Expression::var_or_const(x)))
    /// ```
    pub fn renamed(&self, rename_rules: &RenameRules<S>) -> Self {
        let mut visitor = RenamingVisitor { rename_rules };
        self.clone().visit(&mut visitor) // This clone is not required in principle, but cannot be avoided as long as visitors consume their expression
    }
}
struct RenamingVisitor<'a, S: Span> {
    rename_rules: &'a RenameRules<S>,
}

impl<'a, S: Span> identity_map::Private for RenamingVisitor<'a, S> {}
impl<'a, S: Span> IdentityMapExpression<Identifier<S>, S> for RenamingVisitor<'a, S> {
    fn visit_var_or_const(&mut self, name: Identifier<S>, span: S) -> Expression<Identifier<S>, S> {
        match self.rename_rules.get_renaming(&name) {
            None => Expression::VarOrConst(name, span),
            Some(renaming) => Expression::VarOrConst(renaming, span),
        }
    }
}
impl<S: Span> Expression<Identifier<S>, S> {
    /// Replaces each [`Expression::VarOrConst`] by the variable's or constant's index in
    /// `variable_manager`.
    ///
    /// # Errors
    ///
    /// Returns `Vec<`[`UnknownVariableError<S>`]`>`, which contains an entry for every
    /// [`Expression::VarOrConst`] which does not correspond to a variable or constant in
    /// `variable_manager`.
    ///
    /// # Formulas
    ///
    /// The expression must not contain any formulas. Use [`Expression::substitute_formulas()`] to
    /// replace each formula with its corresponding condition before calling this function.
    ///
    /// # Example
    ///
    /// Let `variable_manager` be a variable manager with a single variable `x`.
    ///
    /// ```
    /// # use prism_model::*;
    /// let mut variable_manager: VariableManagerNamedVars = VariableManager::new();
    /// let var_ref = variable_manager.add_variable(
    ///     VariableInfo::global_var(Identifier::new("x").unwrap(), VariableRange::unbounded_int())
    /// ).unwrap();
    /// #
    /// # let named_var: ExpressionNamedVars = Expression::var_or_const(Identifier::new("x").unwrap());
    /// # let refed_var = Expression::var_or_const(var_ref);
    /// #
    /// # assert_eq!(
    /// #     refed_var, named_var.replace_identifiers_by_variable_indices(&variable_manager).unwrap()
    /// # );
    /// ```
    ///
    /// Let `named_var` be an expression referring to `x` by its name.
    ///
    /// ```
    /// # use prism_model::*;
    /// # let mut variable_manager: VariableManagerNamedVars = VariableManager::new();
    /// # let var_ref = variable_manager.add_variable(
    /// #     VariableInfo::global_var(Identifier::new("x").unwrap(), VariableRange::unbounded_int())
    /// # ).unwrap();
    /// #
    /// let named_var: ExpressionNamedVars = Expression::var_or_const(Identifier::new("x").unwrap());
    /// # let refed_var = Expression::var_or_const(var_ref);
    /// #
    /// # assert_eq!(
    /// #     refed_var, named_var.replace_identifiers_by_variable_indices(&variable_manager).unwrap()
    /// # );
    /// ```
    ///
    /// Let `refed_var` be an expression referring to `x` by its reference. (This can alternatively
    /// be obtained by calling [`VariableManager::get_reference()`]).
    ///
    /// ```
    /// # use prism_model::*;
    /// # let mut variable_manager: VariableManagerNamedVars = VariableManager::new();
    /// # let var_ref = variable_manager.add_variable(
    /// #     VariableInfo::global_var(Identifier::new("x").unwrap(), VariableRange::unbounded_int())
    /// # ).unwrap();
    /// #
    /// # let named_var: ExpressionNamedVars = Expression::var_or_const(Identifier::new("x").unwrap());
    /// let refed_var = Expression::var_or_const(var_ref);
    /// #
    /// # assert_eq!(
    /// #     refed_var, named_var.replace_identifiers_by_variable_indices(&variable_manager).unwrap()
    /// # );
    /// ```
    ///
    /// Then calling `replace_identifiers_by_variable_indices` on `named_var` yields `refed_var`.
    ///
    /// ```
    /// # use prism_model::*;
    /// # let mut variable_manager: VariableManagerNamedVars = VariableManager::new();
    /// # let var_ref = variable_manager.add_variable(
    /// #     VariableInfo::global_var(Identifier::new("x").unwrap(), VariableRange::unbounded_int())
    /// # ).unwrap();
    /// #
    /// # let named_var: ExpressionNamedVars = Expression::var_or_const(Identifier::new("x").unwrap());
    /// # let refed_var = Expression::var_or_const(var_ref);
    /// #
    /// assert_eq!(
    ///     refed_var, named_var.replace_identifiers_by_variable_indices(&variable_manager).unwrap()
    /// );
    /// ```
    pub fn replace_identifiers_by_variable_indices<R>(
        self,
        variable_manager: &VariableManager<S, R>,
    ) -> Result<Expression<VariableReference, S>, Vec<UnknownVariableError<S>>> {
        let errors = Vec::new();
        let mut replace_visitor: MapVariable<Identifier<S>, VariableReference, _, _> =
            MapVariable::new(
                |f, e| match variable_manager.get_reference(&f) {
                    Some(index) => index,
                    None => {
                        e.push(UnknownVariableError {
                            identifier: f.clone(),
                        });
                        VariableReference::new(0)
                    }
                },
                errors,
            );
        let new_expression = self.visit(&mut replace_visitor);
        if !replace_visitor.context.is_empty() {
            Err(replace_visitor.context)
        } else {
            Ok(new_expression)
        }
    }
}

/// Error to indicate that a variable was not found.
///
/// This is thrown when [`Expression::replace_identifiers_by_variable_indices()`] is called on an
/// expression that contains [`Expression::VarOrConst`] with an identifier that is not in the given
/// [`VariableManager`].
///
/// This error is also thrown when the expression contains a formula. See
/// [`Expression::replace_identifiers_by_variable_indices()`] for details.
#[derive(Clone, Debug)]
pub struct UnknownVariableError<S: Span> {
    /// The identifier of the variable, constant or formula that was not found.
    pub identifier: Identifier<S>,
}

impl<V, S: Span> Expression<V, S> {
    fn fmt_internal<VD: Display, F: Fn(&V) -> VD + Clone>(
        &self,
        f: &mut Formatter<'_>,
        surrounding_precedence: usize,
        variable_to_display: F,
    ) -> std::fmt::Result {
        let precedence = self.get_precedence();
        if surrounding_precedence > precedence {
            write!(f, "(")?;
        }
        match self {
            Expression::Int(a, _) => {
                write!(f, "{}", a)?;
            }
            Expression::Float(a, _) => {
                write!(f, "{}", a)?;
            }
            Expression::Bool(true, _) => {
                write!(f, "true")?;
            }
            Expression::VarOrConst(name, _) => {
                write!(f, "{}", variable_to_display(name))?;
            }
            Expression::Label(name, _) => {
                write!(f, "\"{}\"", variable_to_display(name))?;
            }
            Expression::Bool(false, _) => {
                write!(f, "false")?;
            }
            Expression::Function(n, a, _) => {
                write!(f, "{}(", n)?;
                for (index, argument) in a.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    argument.fmt_internal(f, 0, variable_to_display.clone())?;
                }
                write!(f, ")")?;
            }
            Expression::Minus(inner, _) => {
                write!(f, "-")?;
                inner.fmt_internal(f, precedence, variable_to_display)?;
            }
            Expression::Multiplication(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence, variable_to_display.clone())?;
                write!(f, "*")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::Division(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence, variable_to_display.clone())?;
                write!(f, "/")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::Addition(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence, variable_to_display.clone())?;
                write!(f, "+")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::Subtraction(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence, variable_to_display.clone())?;
                write!(f, "-")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::LessThan(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence, variable_to_display.clone())?;
                write!(f, "<")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::LessOrEqual(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence, variable_to_display.clone())?;
                write!(f, "<=")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::GreaterThan(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence, variable_to_display.clone())?;
                write!(f, ">")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::GreaterOrEqual(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence, variable_to_display.clone())?;
                write!(f, ">=")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::Equals(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence, variable_to_display.clone())?;
                write!(f, "=")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::NotEquals(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence, variable_to_display.clone())?;
                write!(f, "!=")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::Negation(inner, _) => {
                write!(f, "!")?;
                inner.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::Conjunction(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence, variable_to_display.clone())?;
                write!(f, "&")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::Disjunction(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence, variable_to_display.clone())?;
                write!(f, "|")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::IfAndOnlyIf(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence, variable_to_display.clone())?;
                write!(f, "<=>")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::Implies(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence, variable_to_display.clone())?;
                write!(f, "=>")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
            Expression::Ternary(cond, lhs, rhs, _) => {
                cond.fmt_internal(f, precedence + 1, variable_to_display.clone())?;
                write!(f, "?")?;
                lhs.fmt_internal(f, precedence + 1, variable_to_display.clone())?;
                write!(f, ":")?;
                rhs.fmt_internal(f, precedence + 1, variable_to_display)?;
            }
        }

        if surrounding_precedence > precedence {
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl<V: std::fmt::Debug, S: Span> std::fmt::Debug for Expression<V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Int(a, _) => {
                write!(f, "{}", a)
            }
            Expression::Float(a, _) => {
                write!(f, "{}", a)
            }
            Expression::Bool(true, _) => {
                write!(f, "true")
            }
            Expression::VarOrConst(name, _) => {
                write!(f, "{:?}", name)
            }
            Expression::Label(name, _) => {
                write!(f, "\"{:?}\"", name)
            }
            Expression::Bool(false, _) => {
                write!(f, "false")
            }
            Expression::Function(n, a, _) => {
                write!(
                    f,
                    "{:?}({})",
                    n,
                    a.iter()
                        .map(|e| format!("{:?}", e))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expression::Minus(a, _) => {
                write!(f, "-({:?})", a)
            }
            Expression::Multiplication(a, b, _) => {
                write!(f, "({:?})*({:?})", a, b)
            }
            Expression::Division(a, b, _) => {
                write!(f, "({:?})/({:?})", a, b)
            }
            Expression::Addition(a, b, _) => {
                write!(f, "({:?})+({:?})", a, b)
            }
            Expression::Subtraction(a, b, _) => {
                write!(f, "({:?})-({:?})", a, b)
            }
            Expression::LessThan(a, b, _) => {
                write!(f, "({:?})<({:?})", a, b)
            }
            Expression::LessOrEqual(a, b, _) => {
                write!(f, "({:?})<=({:?})", a, b)
            }
            Expression::GreaterThan(a, b, _) => {
                write!(f, "({:?})>({:?})", a, b)
            }
            Expression::GreaterOrEqual(a, b, _) => {
                write!(f, "({:?})>=({:?})", a, b)
            }
            Expression::Equals(a, b, _) => {
                write!(f, "({:?})=({:?})", a, b)
            }
            Expression::NotEquals(a, b, _) => {
                write!(f, "({:?})!=({:?})", a, b)
            }
            Expression::Negation(a, _) => {
                write!(f, "!({:?})", a)
            }
            Expression::Conjunction(a, b, _) => {
                write!(f, "({:?})&({:?})", a, b)
            }
            Expression::Disjunction(a, b, _) => {
                write!(f, "({:?})|({:?})", a, b)
            }
            Expression::IfAndOnlyIf(a, b, _) => {
                write!(f, "({:?})<=>({:?})", a, b)
            }
            Expression::Implies(a, b, _) => {
                write!(f, "({:?})=>({:?})", a, b)
            }
            Expression::Ternary(a, b, c, _) => {
                write!(f, "({:?})?({:?}):({:?})", a, b, c)
            }
        }
    }
}

impl<V, S: Span> crate::private::Sealed for Expression<V, S> {}
impl<Ctx, V: Displayable<Ctx>, S: Span> Displayable<Ctx> for Expression<V, S> {
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result {
        self.fmt_internal(f, 0, |v| format!("{}", v.displayable(context)))
    }
}
