use crate::expressions::MapExpression;
use crate::spans::Span;
use crate::{Expression, Identifier};

pub(crate) trait Private {}

/// A (sealed) helper trait for implementing [`MapExpression`] when the output is another
/// expression, where only a few expression variants are changed.
///
/// Every `visit_*()` method has a default implementation that takes results of visiting the
/// children and constructs an expression of the same type.
///
/// Implementing this trait automatically implements [`MapExpression<V, S, Expression<V, S>>`].
///
/// See `FormulaSubstitutionVisitor` for an example implementation.
#[allow(private_bounds)]
pub trait IdentityMapExpression<V, S: Span>: Private {
    /// Visits [`Expression::Int`]. Returns the same expression by default.
    ///
    /// * `val`: The value of the integer literal
    /// * `span`: The [`Span`] of the expression
    fn visit_int(&mut self, val: i64, span: S) -> Expression<V, S> {
        Expression::Int(val, span)
    }

    /// Visits [`Expression::Float`]. Returns the same expression by default.
    ///
    /// * `val`: The value of the floating-point literal
    /// * `span`: The [`Span`] of the expression
    fn visit_float(&mut self, val: f64, span: S) -> Expression<V, S> {
        Expression::Float(val, span)
    }

    /// Visits [`Expression::Bool`]. Returns the same expression by default.
    ///
    /// * `val`: The value of the boolean literal
    /// * `span`: The [`Span`] of the expression
    fn visit_bool(&mut self, val: bool, span: S) -> Expression<V, S> {
        Expression::Bool(val, span)
    }

    /// Visits [`Expression::VarOrConst`]. Returns the same expression by default.
    ///
    /// * `name`: The name of the variable or constant
    /// * `span`: The [`Span`] of the expression
    fn visit_var_or_const(&mut self, name: V, span: S) -> Expression<V, S> {
        Expression::VarOrConst(name, span)
    }

    /// Visits [`Expression::Label`]. Returns the same expression by default.
    ///
    /// * `name`: The name of the label
    /// * `span`: The [`Span`] of the expression
    fn visit_label(&mut self, name: V, span: S) -> Expression<V, S> {
        Expression::Label(name, span)
    }

    /// Visits [`Expression::Function`]. Returns the same expression by default, except that the
    /// arguments to the call are transformed by visiting them.
    ///
    /// * `identifier`: The name of the function being called
    /// * `arguments`: The results of visiting each argument expression
    /// * `span`: The [`Span`] of the expression
    fn visit_function(
        &mut self,
        identifier: Identifier<S>,
        arguments: Vec<Expression<V, S>>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Function(identifier, arguments, span)
    }

    /// Visits [`Expression::Minus`], i.e. an expression of form `-inner`. Returns the same
    /// expression by default, except that `inner` is transformed by visiting it.
    ///
    /// * `inner`: The result of visiting `inner`
    /// * `span`: The [`Span`] of the expression
    fn visit_minus(&mut self, inner: Expression<V, S>, span: S) -> Expression<V, S> {
        Expression::Minus(Box::new(inner), span)
    }

    /// Visits [`Expression::Multiplication`], i.e. an expression of form `left * right`.
    /// Returns the same expression by default, except that `left` and `right` are transformed
    /// by visiting them.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_multiplication(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Multiplication(Box::new(left), Box::new(right), span)
    }

    /// Visits [`Expression::Division`], i.e. an expression of form `left / right`.
    /// Returns the same expression by default, except that `left` and `right` are transformed
    /// by visiting them.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_division(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Division(Box::new(left), Box::new(right), span)
    }

    /// Visits [`Expression::Addition`], i.e. an expression of form `left + right`.
    /// Returns the same expression by default, except that `left` and `right` are transformed
    /// by visiting them.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_addition(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Addition(Box::new(left), Box::new(right), span)
    }

    /// Visits [`Expression::Subtraction`], i.e. an expression of form `left - right`.
    /// Returns the same expression by default, except that `left` and `right` are transformed
    /// by visiting them.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_subtraction(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Subtraction(Box::new(left), Box::new(right), span)
    }

    /// Visits [`Expression::LessThan`], i.e. an expression of form `left < right`.
    /// Returns the same expression by default, except that `left` and `right` are transformed
    /// by visiting them.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_less_than(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::LessThan(Box::new(left), Box::new(right), span)
    }

    /// Visits [`Expression::LessOrEqual`], i.e. an expression of form `left <= right`.
    /// Returns the same expression by default, except that `left` and `right` are transformed
    /// by visiting them.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_less_or_equal(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::LessOrEqual(Box::new(left), Box::new(right), span)
    }

    /// Visits [`Expression::GreaterThan`], i.e. an expression of form `left > right`.
    /// Returns the same expression by default, except that `left` and `right` are transformed
    /// by visiting them.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_greater_than(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::GreaterThan(Box::new(left), Box::new(right), span)
    }

    /// Visits [`Expression::GreaterOrEqual`], i.e. an expression of form `left >= right`.
    /// Returns the same expression by default, except that `left` and `right` are transformed
    /// by visiting them.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_greater_or_equal(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::GreaterOrEqual(Box::new(left), Box::new(right), span)
    }

    /// Visits [`Expression::Equals`], i.e. an expression of form `left = right`.
    /// Returns the same expression by default, except that `left` and `right` are transformed
    /// by visiting them.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_equals(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Equals(Box::new(left), Box::new(right), span)
    }

    /// Visits [`Expression::NotEquals`], i.e. an expression of form `left != right`.
    /// Returns the same expression by default, except that `left` and `right` are transformed
    /// by visiting them.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_not_equals(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::NotEquals(Box::new(left), Box::new(right), span)
    }

    /// Visits [`Expression::Negation`], i.e. an expression of form `!inner`.
    /// Returns the same expression by default, except that `inner` is transformed by visiting it.
    ///
    /// * `inner`: The result of visiting `inner`
    /// * `span`: The [`Span`] of the expression
    fn visit_negation(&mut self, inner: Expression<V, S>, span: S) -> Expression<V, S> {
        Expression::Negation(Box::new(inner), span)
    }

    /// Visits [`Expression::Conjunction`], i.e. an expression of form `left & right`.
    /// Returns the same expression by default, except that `left` and `right` are transformed
    /// by visiting them.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_conjunction(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Conjunction(Box::new(left), Box::new(right), span)
    }

    /// Visits [`Expression::Disjunction`], i.e. an expression of form `left | right`.
    /// Returns the same expression by default, except that `left` and `right` are transformed
    /// by visiting them.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_disjunction(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Disjunction(Box::new(left), Box::new(right), span)
    }

    /// Visits [`Expression::IfAndOnlyIf`], i.e. an expression of form `left <=> right`.
    /// Returns the same expression by default, except that `left` and `right` are transformed
    /// by visiting them.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_if_and_only_if(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::IfAndOnlyIf(Box::new(left), Box::new(right), span)
    }

    /// Visits [`Expression::Implies`], i.e. an expression of form `left => right`.
    /// Returns the same expression by default, except that `left` and `right` are transformed
    /// by visiting them.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_implies(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Implies(Box::new(left), Box::new(right), span)
    }

    /// Visits [`Expression::Ternary`], i.e. an expression of form `condition ? left : right`.
    /// Returns the same expression by default, except that `condition`, `left`, and `right`
    /// are transformed by visiting them.
    ///
    /// * `condition`: The result of visiting `condition`
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_ternary(
        &mut self,
        condition: Expression<V, S>,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Ternary(Box::new(condition), Box::new(left), Box::new(right), span)
    }
}

impl<V, S: Span, M: IdentityMapExpression<V, S>> MapExpression<V, S, Expression<V, S>> for M {
    fn visit_int(&mut self, val: i64, span: S) -> Expression<V, S> {
        self.visit_int(val, span)
    }
    fn visit_float(&mut self, val: f64, span: S) -> Expression<V, S> {
        self.visit_float(val, span)
    }
    fn visit_bool(&mut self, val: bool, span: S) -> Expression<V, S> {
        self.visit_bool(val, span)
    }
    fn visit_var_or_const(&mut self, name: V, span: S) -> Expression<V, S> {
        self.visit_var_or_const(name, span)
    }
    fn visit_label(&mut self, name: V, span: S) -> Expression<V, S> {
        self.visit_label(name, span)
    }
    fn visit_function(
        &mut self,
        identifier: Identifier<S>,
        arguments: Vec<Expression<V, S>>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_function(identifier, arguments, span)
    }
    fn visit_minus(&mut self, inner: Expression<V, S>, span: S) -> Expression<V, S> {
        self.visit_minus(inner, span)
    }
    fn visit_multiplication(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_multiplication(left, right, span)
    }
    fn visit_division(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_division(left, right, span)
    }
    fn visit_addition(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_addition(left, right, span)
    }
    fn visit_subtraction(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_subtraction(left, right, span)
    }
    fn visit_less_than(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_less_than(left, right, span)
    }
    fn visit_less_or_equal(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_less_or_equal(left, right, span)
    }
    fn visit_greater_than(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_greater_than(left, right, span)
    }
    fn visit_greater_or_equal(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_greater_or_equal(left, right, span)
    }
    fn visit_equals(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_equals(left, right, span)
    }
    fn visit_not_equals(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_not_equals(left, right, span)
    }
    fn visit_negation(&mut self, inner: Expression<V, S>, span: S) -> Expression<V, S> {
        self.visit_negation(inner, span)
    }
    fn visit_conjunction(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_conjunction(left, right, span)
    }
    fn visit_disjunction(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_disjunction(left, right, span)
    }
    fn visit_if_and_only_if(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_if_and_only_if(left, right, span)
    }
    fn visit_implies(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_implies(left, right, span)
    }
    fn visit_ternary(
        &mut self,
        condition: Expression<V, S>,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_ternary(condition, left, right, span)
    }
}
