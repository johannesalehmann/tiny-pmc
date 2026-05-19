mod default_map;
pub use default_map::DefaultMapExpression;

pub mod identity_map;
use crate::spans::Span;
pub use identity_map::IdentityMapExpression;

pub mod map_span;

pub mod map_variable;

use super::{Expression, Identifier};

/// A trait used to implement a visitor pattern on [`Expression<V, S>`], producing a value of type
/// `T`.
///
/// Calling [`Expression::visit()`] explores a depth-first traversal of the expression tree. Each
/// node is transformed into a value of type `T`. For nodes with children, the children are visited
/// first and the resulting values of type `T` are given to the parent node.
///
/// `T` can be of type [`Expression<V2, S2>`] to transform an expression into another expression or
/// can be of arbitrary type to collect other information about the expression (for example, `T`
/// could be `bool` to determine whether an expression contains a specific variable).
///
/// # Helpers
///
/// Use sealed trait [`IdentityMapExpression`] to implement `MapExpression` when most expression
/// elements should be mapped to themselves. For example, to double every  `Expression::Int` in an
/// expression, only `visit_int` in `IdentityMapExpression` needs to be implemented. The remaining
/// are covered automatically. See `FormulaSubstitutionVisitor` for an example.
///
/// Use trait [`DefaultMapExpression`] to map most elements to `T::default()`. This is mainly used
/// when `T = ()`, with information about the expression instead tracked in the implementor of
/// `MapExpression` directly. See `FormulaCountingVisitor` for an example.
pub trait MapExpression<V, S: Span, T> {
    /// Visits [`Expression::Int`].
    ///
    /// * `val`: The value of the integer literal
    /// * `span`: The [`Span`] of the expression
    fn visit_int(&mut self, val: i64, span: S) -> T;

    /// Visits [`Expression::Float`].
    ///
    /// * `val`: The value of the floating-point literal
    /// * `span`: The [`Span`] of the expression
    fn visit_float(&mut self, val: f64, span: S) -> T;

    /// Visits [`Expression::Bool`].
    ///
    /// * `val`: The value of the boolean literal
    /// * `span`: The [`Span`] of the expression
    fn visit_bool(&mut self, val: bool, span: S) -> T;

    /// Visits [`Expression::VarOrConst`], i.e. an expression representing a variable, constant or
    /// formula.
    ///
    /// * `name`: The variable, constant or formula
    /// * `span`: The [`Span`] of the expression
    fn visit_var_or_const(&mut self, name: V, span: S) -> T;

    /// Visits [`Expression::Label`], i.e. a reference to a label by name.
    ///
    /// * `name`: The name of the label
    /// * `span`: The [`Span`] of the expression
    fn visit_label(&mut self, name: V, span: S) -> T;

    /// Visits [`Expression::Function`], i.e. a function call of form `identifier(arguments)`.
    ///
    /// * `identifier`: The name of the function being called
    /// * `arguments`: The results of visiting each argument expression
    /// * `span`: The [`Span`] of the expression
    fn visit_function(&mut self, identifier: Identifier<S>, arguments: Vec<T>, span: S) -> T;

    /// Visits [`Expression::Minus`], i.e. an expression of form `-inner`.
    ///
    /// * `inner`: The result of visiting `inner`
    /// * `span`: The [`Span`] of the expression
    fn visit_minus(&mut self, inner: T, span: S) -> T;

    /// Visits [`Expression::Multiplication`], i.e. an expression of form `left * right`.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_multiplication(&mut self, left: T, right: T, span: S) -> T;

    /// Visits [`Expression::Division`], i.e. an expression of form `left / right`.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_division(&mut self, left: T, right: T, span: S) -> T;

    /// Visits [`Expression::Addition`], i.e. an expression of form `left + right`.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_addition(&mut self, left: T, right: T, span: S) -> T;

    /// Visits [`Expression::Subtraction`], i.e. an expression of form `left - right`.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_subtraction(&mut self, left: T, right: T, span: S) -> T;

    /// Visits [`Expression::LessThan`], i.e. an expression of form `left < right`.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_less_than(&mut self, left: T, right: T, span: S) -> T;

    /// Visits [`Expression::LessOrEqual`], i.e. an expression of form `left <= right`.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_less_or_equal(&mut self, left: T, right: T, span: S) -> T;

    /// Visits [`Expression::GreaterThan`], i.e. an expression of form `left > right`.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_greater_than(&mut self, left: T, right: T, span: S) -> T;

    /// Visits [`Expression::GreaterOrEqual`], i.e. an expression of form `left >= right`.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_greater_or_equal(&mut self, left: T, right: T, span: S) -> T;

    /// Visits [`Expression::Equals`], i.e. an expression of form `left = right`.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_equals(&mut self, left: T, right: T, span: S) -> T;

    /// Visits [`Expression::NotEquals`], i.e. an expression of form `left != right`.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_not_equals(&mut self, left: T, right: T, span: S) -> T;

    /// Visits [`Expression::Negation`], i.e. an expression of form `!inner`.
    ///
    /// * `inner`: The result of visiting `inner`
    /// * `span`: The [`Span`] of the expression
    fn visit_negation(&mut self, inner: T, span: S) -> T;

    /// Visits [`Expression::Conjunction`], i.e. an expression of form `left & right`.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_conjunction(&mut self, left: T, right: T, span: S) -> T;

    /// Visits [`Expression::Disjunction`], i.e. an expression of form `left | right`.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_disjunction(&mut self, left: T, right: T, span: S) -> T;

    /// Visits [`Expression::IfAndOnlyIf`], i.e. an expression of form `left <=> right`.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_if_and_only_if(&mut self, left: T, right: T, span: S) -> T;

    /// Visits [`Expression::Implies`], i.e. an expression of form `left => right`.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_implies(&mut self, left: T, right: T, span: S) -> T;

    /// Visits [`Expression::Ternary`], i.e. an expression of form `condition ? left : right`.
    ///
    /// * `condition`: The result of visiting `condition`
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_ternary(&mut self, condition: T, left: T, right: T, span: S) -> T;
}

impl<V, S: Span> Expression<V, S> {
    /// Traverses the expression tree depth-first, applying `m` to each node.
    ///
    /// Children are visited before their parent, so by the time a `visit_*` method on `m` is
    /// called, all of its children have already been transformed into values of type `T`.
    pub fn visit<T, M: MapExpression<V, S, T>>(self, m: &mut M) -> T {
        match self {
            Expression::Int(val, s) => m.visit_int(val, s),
            Expression::Float(val, s) => m.visit_float(val, s),
            Expression::Bool(val, s) => m.visit_bool(val, s),
            Expression::VarOrConst(name, s) => m.visit_var_or_const(name, s),
            Expression::Label(name, s) => m.visit_label(name, s),
            Expression::Function(identifier, arguments, s) => {
                let mapped_args = arguments
                    .into_iter()
                    .map(|a| a.visit(m))
                    .collect::<Vec<_>>();
                m.visit_function(identifier, mapped_args, s)
            }
            Expression::Minus(inner, s) => {
                let inner = inner.visit(m);
                m.visit_minus(inner, s)
            }
            Expression::Multiplication(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_multiplication(lhs, rhs, s)
            }
            Expression::Division(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_division(lhs, rhs, s)
            }
            Expression::Addition(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_addition(lhs, rhs, s)
            }
            Expression::Subtraction(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_subtraction(lhs, rhs, s)
            }
            Expression::LessThan(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_less_than(lhs, rhs, s)
            }
            Expression::LessOrEqual(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_less_or_equal(lhs, rhs, s)
            }
            Expression::GreaterThan(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_greater_than(lhs, rhs, s)
            }
            Expression::GreaterOrEqual(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_greater_or_equal(lhs, rhs, s)
            }
            Expression::Equals(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_equals(lhs, rhs, s)
            }
            Expression::NotEquals(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_not_equals(lhs, rhs, s)
            }
            Expression::Negation(inner, s) => {
                let inner = inner.visit(m);
                m.visit_negation(inner, s)
            }
            Expression::Conjunction(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_conjunction(lhs, rhs, s)
            }
            Expression::Disjunction(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_disjunction(lhs, rhs, s)
            }
            Expression::IfAndOnlyIf(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_if_and_only_if(lhs, rhs, s)
            }
            Expression::Implies(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_implies(lhs, rhs, s)
            }
            Expression::Ternary(condition, lhs, rhs, s) => {
                let condition = condition.visit(m);
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_ternary(condition, lhs, rhs, s)
            }
        }
    }
}
