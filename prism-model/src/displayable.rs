use std::fmt::{Display, Formatter};

/// Used to display PRISM model components that require context for their `Display` implementation.
///
/// # Rationale
///
/// To avoid duplication, not every component of a PRISM model stores all the information required
/// to implement [`Display`]. For example, an [`Expression`](crate::Expression) using
/// [`VariableReference`](crate::VariableReference) for variables only knows the index of a variable
/// in the [`VariableManager`](crate::VariableManager), but not the identifier of the variable. In
/// order to format an expression, it is therefore necessary to provide additional context. This
/// trait encapsulates the necessary process.
///
/// By calling [`Displayable::displayable()`], the component to be displayed is transformed into a
/// [`DisplayableWithContext`], which stores the necessary context (e.g. a
/// [`VariableManager`](crate::VariableManager) for [`Expression`](crate::Expression) and implements
/// `Display`.
///
/// # Example
///
/// ```
/// # use prism_model::{Expression, Identifier, VariableInfo, VariableManager, VariableRange, Displayable};
/// let mut variable_manager: VariableManager = VariableManager::new();
/// let reference = variable_manager.add_variable(
///     VariableInfo::global_var(Identifier::new("x").unwrap(), VariableRange::unbounded_int())
/// ).expect("Variable exists");
///
/// let expression: Expression = Expression::var_or_const(reference).plus(Expression::int(12));
///
/// assert_eq!(
///     format!("{}", expression.displayable(&variable_manager)),
///     "x+12"
/// );
///
/// ```
pub trait Displayable<Ctx>: private::Sealed {
    /// Creates a type that includes the context necessary to display `Self`.
    ///
    /// See the trait-level documentation at [`Displayable`] for details.
    fn displayable<'a, 'b>(
        &'a self,
        context: &'b Ctx,
    ) -> DisplayableWithContext<'a, 'b, Self, Ctx> {
        DisplayableWithContext {
            element: self,
            context,
        }
    }

    /// Formats `Self` with the given context.
    ///
    /// This function has the same arguments as [`Display::fmt()`], except that a reference to
    /// `context` is also provided. This can include additional information required to format
    /// `Self`.
    ///
    /// See the trait-level documentation at [`Displayable`] for details.
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result;
}

/// Encapsulates `element` and the context necessary to display it.
///
/// Usually used in combination with [`Displayable`] for PRISM components that require additional
/// context for formatting. See the documentation of [`Displayable`] for details.
pub struct DisplayableWithContext<'a, 'b, O: ?Sized + Displayable<Ctx>, Ctx> {
    element: &'a O,
    context: &'b Ctx,
}

pub mod private {
    pub trait Sealed {}
}

impl<O: ?Sized + Displayable<Ctx>, Ctx> Display for DisplayableWithContext<'_, '_, O, Ctx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.element.fmt_internal(f, self.context)
    }
}
