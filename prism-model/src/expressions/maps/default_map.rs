#[cfg(doc)]
use crate::Expression;
use crate::Identifier;
use crate::expressions::MapExpression;
use crate::spans::Span;

/// A helper trait for implementing [`MapExpression`] when most expression nodes should produce
/// `T::default()`.
///
/// Every method has a default implementation that returns `T::default()`, so only the nodes of
/// interest need to be overridden. This is mainly useful when `T = ()` and results are instead
/// accumulated as mutable state on `self`.
///
/// Implementing this trait automatically implements [`MapExpression<V, S, T>`].
///
/// # Example
///
/// ```
/// # use prism_model::{DefaultMapExpression, Expression, Identifier, Span};
/// struct VarCollector{ vars: Vec<String>};
///
/// impl<S: Span> DefaultMapExpression<Identifier<S>, S, ()> for VarCollector {
///     fn visit_var_or_const(&mut self, name: Identifier<S>, span: S) -> () {
///         self.vars.push(name.name.clone());
///     }
/// }
/// ```
pub trait DefaultMapExpression<V, S: Span, T: Default> {
    /// Visits [`Expression::Int`]. Returns `T::default()` by default.
    ///
    /// * `val`: The value of the integer literal
    /// * `span`: The [`Span`] of the expression
    fn visit_int(&mut self, val: i64, span: S) -> T {
        let _ = (val, span);
        T::default()
    }

    /// Visits [`Expression::Float`]. Returns `T::default()` by default.
    ///
    /// * `val`: The value of the floating-point literal
    /// * `span`: The [`Span`] of the expression
    fn visit_float(&mut self, val: f64, span: S) -> T {
        let _ = (val, span);
        T::default()
    }

    /// Visits [`Expression::Bool`]. Returns `T::default()` by default.
    ///
    /// * `val`: The value of the boolean literal
    /// * `span`: The [`Span`] of the expression
    fn visit_bool(&mut self, val: bool, span: S) -> T {
        let _ = (val, span);
        T::default()
    }

    /// Visits [`Expression::VarOrConst`]. Returns `T::default()` by default.
    ///
    /// * `name`: The name of the variable or constant
    /// * `span`: The [`Span`] of the expression
    fn visit_var_or_const(&mut self, name: V, span: S) -> T {
        let _ = (name, span);
        T::default()
    }

    /// Visits [`Expression::Label`]. Returns `T::default()` by default.
    ///
    /// * `name`: The name of the label
    /// * `span`: The [`Span`] of the expression
    fn visit_label(&mut self, name: V, span: S) -> T {
        let _ = (name, span);
        T::default()
    }

    /// Visits [`Expression::Function`]. Returns `T::default()` by default.
    ///
    /// * `identifier`: The name of the function being called
    /// * `arguments`: The results of visiting each argument expression
    /// * `span`: The [`Span`] of the expression
    fn visit_function(&mut self, identifier: Identifier<S>, arguments: Vec<T>, span: S) -> T {
        let _ = (identifier, arguments, span);
        T::default()
    }

    /// Visits [`Expression::Minus`], i.e. an expression of form `-inner`.
    /// Returns `T::default()` by default.
    ///
    /// * `inner`: The result of visiting `inner`
    /// * `span`: The [`Span`] of the expression
    fn visit_minus(&mut self, inner: T, span: S) -> T {
        let _ = (inner, span);
        T::default()
    }

    /// Visits [`Expression::Multiplication`], i.e. an expression of form `left * right`.
    /// Returns `T::default()` by default.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_multiplication(&mut self, left: T, right: T, span: S) -> T {
        let _ = (left, right, span);
        T::default()
    }

    /// Visits [`Expression::Division`], i.e. an expression of form `left / right`.
    /// Returns `T::default()` by default.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_division(&mut self, left: T, right: T, span: S) -> T {
        let _ = (left, right, span);
        T::default()
    }

    /// Visits [`Expression::Addition`], i.e. an expression of form `left + right`.
    /// Returns `T::default()` by default.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_addition(&mut self, left: T, right: T, span: S) -> T {
        let _ = (left, right, span);
        T::default()
    }

    /// Visits [`Expression::Subtraction`], i.e. an expression of form `left - right`.
    /// Returns `T::default()` by default.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_subtraction(&mut self, left: T, right: T, span: S) -> T {
        let _ = (left, right, span);
        T::default()
    }

    /// Visits [`Expression::LessThan`], i.e. an expression of form `left < right`.
    /// Returns `T::default()` by default.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_less_than(&mut self, left: T, right: T, span: S) -> T {
        let _ = (left, right, span);
        T::default()
    }

    /// Visits [`Expression::LessOrEqual`], i.e. an expression of form `left <= right`.
    /// Returns `T::default()` by default.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_less_or_equal(&mut self, left: T, right: T, span: S) -> T {
        let _ = (left, right, span);
        T::default()
    }

    /// Visits [`Expression::GreaterThan`], i.e. an expression of form `left > right`.
    /// Returns `T::default()` by default.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_greater_than(&mut self, left: T, right: T, span: S) -> T {
        let _ = (left, right, span);
        T::default()
    }

    /// Visits [`Expression::GreaterOrEqual`], i.e. an expression of form `left >= right`.
    /// Returns `T::default()` by default.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_greater_or_equal(&mut self, left: T, right: T, span: S) -> T {
        let _ = (left, right, span);
        T::default()
    }

    /// Visits [`Expression::Equals`], i.e. an expression of form `left = right`.
    /// Returns `T::default()` by default.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_equals(&mut self, left: T, right: T, span: S) -> T {
        let _ = (left, right, span);
        T::default()
    }

    /// Visits [`Expression::NotEquals`], i.e. an expression of form `left != right`.
    /// Returns `T::default()` by default.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_not_equals(&mut self, left: T, right: T, span: S) -> T {
        let _ = (left, right, span);
        T::default()
    }

    /// Visits [`Expression::Negation`], i.e. an expression of form `!inner`.
    /// Returns `T::default()` by default.
    ///
    /// * `inner`: The result of visiting `inner`
    /// * `span`: The [`Span`] of the expression
    fn visit_negation(&mut self, inner: T, span: S) -> T {
        let _ = (inner, span);
        T::default()
    }

    /// Visits [`Expression::Conjunction`], i.e. an expression of form `left & right`.
    /// Returns `T::default()` by default.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_conjunction(&mut self, left: T, right: T, span: S) -> T {
        let _ = (left, right, span);
        T::default()
    }

    /// Visits [`Expression::Disjunction`], i.e. an expression of form `left | right`.
    /// Returns `T::default()` by default.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_disjunction(&mut self, left: T, right: T, span: S) -> T {
        let _ = (left, right, span);
        T::default()
    }

    /// Visits [`Expression::IfAndOnlyIf`], i.e. an expression of form `left <=> right`.
    /// Returns `T::default()` by default.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_if_and_only_if(&mut self, left: T, right: T, span: S) -> T {
        let _ = (left, right, span);
        T::default()
    }

    /// Visits [`Expression::Implies`], i.e. an expression of form `left => right`.
    /// Returns `T::default()` by default.
    ///
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_implies(&mut self, left: T, right: T, span: S) -> T {
        let _ = (left, right, span);
        T::default()
    }

    /// Visits [`Expression::Ternary`], i.e. an expression of form `condition ? left : right`.
    /// Returns `T::default()` by default.
    ///
    /// * `condition`: The result of visiting `condition`
    /// * `left`: The result of visiting `left`
    /// * `right`: The result of visiting `right`
    /// * `span`: The [`Span`] of the expression
    fn visit_ternary(&mut self, condition: T, left: T, right: T, span: S) -> T {
        let _ = (condition, left, right, span);
        T::default()
    }
}

impl<V, S: Span, T: Default, M: DefaultMapExpression<V, S, T>> MapExpression<V, S, T> for M {
    fn visit_int(&mut self, val: i64, span: S) -> T {
        self.visit_int(val, span)
    }
    fn visit_float(&mut self, val: f64, span: S) -> T {
        self.visit_float(val, span)
    }
    fn visit_bool(&mut self, val: bool, span: S) -> T {
        self.visit_bool(val, span)
    }
    fn visit_var_or_const(&mut self, name: V, span: S) -> T {
        self.visit_var_or_const(name, span)
    }
    fn visit_label(&mut self, name: V, span: S) -> T {
        self.visit_label(name, span)
    }
    fn visit_function(&mut self, identifier: Identifier<S>, arguments: Vec<T>, span: S) -> T {
        self.visit_function(identifier, arguments, span)
    }
    fn visit_minus(&mut self, inner: T, span: S) -> T {
        self.visit_minus(inner, span)
    }
    fn visit_multiplication(&mut self, left: T, right: T, span: S) -> T {
        self.visit_multiplication(left, right, span)
    }
    fn visit_division(&mut self, left: T, right: T, span: S) -> T {
        self.visit_division(left, right, span)
    }
    fn visit_addition(&mut self, left: T, right: T, span: S) -> T {
        self.visit_addition(left, right, span)
    }
    fn visit_subtraction(&mut self, left: T, right: T, span: S) -> T {
        self.visit_subtraction(left, right, span)
    }
    fn visit_less_than(&mut self, left: T, right: T, span: S) -> T {
        self.visit_less_than(left, right, span)
    }
    fn visit_less_or_equal(&mut self, left: T, right: T, span: S) -> T {
        self.visit_less_or_equal(left, right, span)
    }
    fn visit_greater_than(&mut self, left: T, right: T, span: S) -> T {
        self.visit_greater_than(left, right, span)
    }
    fn visit_greater_or_equal(&mut self, left: T, right: T, span: S) -> T {
        self.visit_greater_or_equal(left, right, span)
    }
    fn visit_equals(&mut self, left: T, right: T, span: S) -> T {
        self.visit_equals(left, right, span)
    }
    fn visit_not_equals(&mut self, left: T, right: T, span: S) -> T {
        self.visit_not_equals(left, right, span)
    }
    fn visit_negation(&mut self, inner: T, span: S) -> T {
        self.visit_negation(inner, span)
    }
    fn visit_conjunction(&mut self, left: T, right: T, span: S) -> T {
        self.visit_conjunction(left, right, span)
    }
    fn visit_disjunction(&mut self, left: T, right: T, span: S) -> T {
        self.visit_disjunction(left, right, span)
    }
    fn visit_if_and_only_if(&mut self, left: T, right: T, span: S) -> T {
        self.visit_if_and_only_if(left, right, span)
    }
    fn visit_implies(&mut self, left: T, right: T, span: S) -> T {
        self.visit_implies(left, right, span)
    }
    fn visit_ternary(&mut self, condition: T, left: T, right: T, span: S) -> T {
        self.visit_ternary(condition, left, right, span)
    }
}
