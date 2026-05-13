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

#[derive(PartialEq, Clone)]
pub struct GlobalVariableReference {
    reference: VariableReference,
    scope: VariableScope,
}

#[derive(PartialEq, Clone)]
pub enum VariableScope {
    GlobalVariable,
    GlobalConst,
    Formula,
    LocalVariable { module: String },
}

pub type ExpressionNamedVars<S: Span = FullSpan> = Expression<Identifier<S>, S>;

#[derive(PartialEq, Clone)]
pub enum Expression<V = VariableReference, S: Span = FullSpan> {
    Int(i64, S),
    Float(f64, S),
    Bool(bool, S),
    VarOrConst(V, S),
    Label(V, S),
    Function(Identifier<S>, Vec<Expression<V, S>>, S),
    Minus(Box<Expression<V, S>>, S),
    Multiplication(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Division(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Addition(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Subtraction(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    LessThan(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    LessOrEqual(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    GreaterThan(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    GreaterOrEqual(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Equals(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    NotEquals(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Negation(Box<Expression<V, S>>, S),
    Conjunction(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Disjunction(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    IfAndOnlyIf(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Implies(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Ternary(
        Box<Expression<V, S>>,
        Box<Expression<V, S>>,
        Box<Expression<V, S>>,
        S,
    ),
}

impl<V, S: Span> Expression<V, S> {
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

    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> Expression<V, S2> {
        let mut visitor = maps::map_span::MapSpan::new(map);
        self.visit(&mut visitor)
    }

    pub fn map_variable<V2, F: Fn(V) -> V2>(self, map: &F) -> Expression<V2, S> {
        let mut visitor = maps::map_variable::MapVariable::new(|v, _| map(v), ());
        self.visit(&mut visitor)
    }

    pub fn get_precedence(&self) -> usize {
        // As per https://www.prismmodelchecker.org/manual/ThePRISMLanguage/Expressions
        // ranging from 1 (for ternary) to 11 (for unary minus)
        // Precedence 12 is used for atoms (literals, variables)
        // 0 is used to indicate there is no surrounding precedence
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
    pub fn int(val: i64) -> Self {
        Expression::Int(val, S::empty())
    }
    pub fn int_spanned(val: i64, span: S) -> Self {
        Expression::Int(val, span)
    }
    pub fn float(val: f64) -> Self {
        Expression::Float(val, S::empty())
    }
    pub fn float_spanned(val: f64, span: S) -> Self {
        Expression::Float(val, span)
    }
    pub fn bool(val: bool) -> Self {
        Expression::Bool(val, S::empty())
    }
    pub fn bool_spanned(val: bool, span: S) -> Self {
        Expression::Bool(val, span)
    }
    pub fn var_or_const(id: V) -> Self {
        Expression::VarOrConst(id, S::empty())
    }
    pub fn var_or_const_spanned(id: V, span: S) -> Self {
        Expression::VarOrConst(id, span)
    }
    pub fn label(id: V) -> Self {
        Expression::Label(id, S::empty())
    }
    pub fn label_spanned(id: V, span: S) -> Self {
        Expression::Label(id, span)
    }

    pub fn function<A: Into<Vec<Self>>>(identifier: Identifier<S>, args: A) -> Self {
        Expression::Function(identifier, args.into(), S::empty())
    }
    pub fn function_spanned<A: Into<Vec<Self>>>(
        identifier: Identifier<S>,
        args: A,
        span: S,
    ) -> Self {
        Expression::Function(identifier, args.into(), span)
    }

    pub fn negate_value(self) -> Self {
        Expression::Minus(Box::new(self), S::empty())
    }
    pub fn negate_value_spanned(self, span: S) -> Self {
        Expression::Minus(Box::new(self), span)
    }

    pub fn times(self, other: Self) -> Self {
        Expression::Multiplication(Box::new(self), Box::new(other), S::empty())
    }
    pub fn times_spanned(self, other: Self, span: S) -> Self {
        Expression::Multiplication(Box::new(self), Box::new(other), span)
    }

    pub fn divide_by(self, other: Self) -> Self {
        Expression::Division(Box::new(self), Box::new(other), S::empty())
    }
    pub fn divide_by_spanned(self, other: Self, span: S) -> Self {
        Expression::Division(Box::new(self), Box::new(other), span)
    }

    pub fn plus(self, other: Self) -> Self {
        Expression::Addition(Box::new(self), Box::new(other), S::empty())
    }
    pub fn plus_spanned(self, other: Self, span: S) -> Self {
        Expression::Addition(Box::new(self), Box::new(other), span)
    }

    pub fn minus(self, other: Self) -> Self {
        Expression::Subtraction(Box::new(self), Box::new(other), S::empty())
    }
    pub fn minus_spanned(self, other: Self, span: S) -> Self {
        Expression::Subtraction(Box::new(self), Box::new(other), span)
    }

    pub fn less_than(self, other: Self) -> Self {
        Expression::LessThan(Box::new(self), Box::new(other), S::empty())
    }
    pub fn less_than_spanned(self, other: Self, span: S) -> Self {
        Expression::LessThan(Box::new(self), Box::new(other), span)
    }

    pub fn less_or_equal(self, other: Self) -> Self {
        Expression::LessOrEqual(Box::new(self), Box::new(other), S::empty())
    }
    pub fn less_or_equal_spanned(self, other: Self, span: S) -> Self {
        Expression::LessOrEqual(Box::new(self), Box::new(other), span)
    }

    pub fn greater_than(self, other: Self) -> Self {
        Expression::GreaterThan(Box::new(self), Box::new(other), S::empty())
    }
    pub fn greater_than_spanned(self, other: Self, span: S) -> Self {
        Expression::GreaterThan(Box::new(self), Box::new(other), span)
    }

    pub fn greater_or_equal(self, other: Self) -> Self {
        Expression::GreaterOrEqual(Box::new(self), Box::new(other), S::empty())
    }
    pub fn greater_or_equal_spanned(self, other: Self, span: S) -> Self {
        Expression::GreaterOrEqual(Box::new(self), Box::new(other), span)
    }

    pub fn equals_to(self, other: Self) -> Self {
        Expression::Equals(Box::new(self), Box::new(other), S::empty())
    }
    pub fn equals_to_spanned(self, other: Self, span: S) -> Self {
        Expression::Equals(Box::new(self), Box::new(other), span)
    }

    pub fn not_equals_to(self, other: Self) -> Self {
        Expression::NotEquals(Box::new(self), Box::new(other), S::empty())
    }
    pub fn not_equals_to_spanned(self, other: Self, span: S) -> Self {
        Expression::NotEquals(Box::new(self), Box::new(other), span)
    }

    pub fn negate_bool(self) -> Self {
        Expression::Negation(Box::new(self), S::empty())
    }
    pub fn negate_bool_spanned(self, span: S) -> Self {
        Expression::Negation(Box::new(self), span)
    }

    pub fn and(self, other: Self) -> Self {
        Expression::Conjunction(Box::new(self), Box::new(other), S::empty())
    }
    pub fn and_spanned(self, other: Self, span: S) -> Self {
        Expression::Conjunction(Box::new(self), Box::new(other), span)
    }

    pub fn or(self, other: Self) -> Self {
        Expression::Disjunction(Box::new(self), Box::new(other), S::empty())
    }
    pub fn or_spanned(self, other: Self, span: S) -> Self {
        Expression::Disjunction(Box::new(self), Box::new(other), span)
    }

    pub fn if_and_only_if(self, other: Self) -> Self {
        Expression::IfAndOnlyIf(Box::new(self), Box::new(other), S::empty())
    }
    pub fn if_and_only_if_spanned(self, other: Self, span: S) -> Self {
        Expression::IfAndOnlyIf(Box::new(self), Box::new(other), span)
    }

    pub fn implies(self, other: Self) -> Self {
        Expression::Implies(Box::new(self), Box::new(other), S::empty())
    }
    pub fn implies_spanned(self, other: Self, span: S) -> Self {
        Expression::Implies(Box::new(self), Box::new(other), span)
    }

    pub fn ternary(self, branch_1: Self, branch_2: Self) -> Self {
        Expression::Ternary(
            Box::new(self),
            Box::new(branch_1),
            Box::new(branch_2),
            S::empty(),
        )
    }
    pub fn ternary_spanned(self, branch_1: Self, branch_2: Self, span: S) -> Self {
        Expression::Ternary(Box::new(self), Box::new(branch_1), Box::new(branch_2), span)
    }
}

impl<S: Span> Expression<Identifier<S>, S> {
    pub fn substitute_labels(
        &mut self,
        default_span: S,
        labels: &LabelManager<S, Expression<Identifier<S>, S>>,
    ) {
        for label in &labels.labels {
            let mut visitor = LabelSubstitutionVisitor {
                label_name: &label.name,
                expression: &label.condition,
            };

            let condition = std::mem::replace(self, Expression::Bool(false, default_span.clone()));
            *self = condition.visit(&mut visitor);
        }
    }
    pub fn substitute_formulas(
        &mut self,
        default_span: S,
        formulas: &FormulaManager<S, Expression<Identifier<S>, S>>,
    ) -> Result<(), CyclicDependency<S>> {
        let order = formulas.get_formula_replacement_ordering()?;

        for formula_index in order {
            let formula = formulas.get(formula_index).unwrap();
            let mut visitor = crate::model::FormulaSubstitutionVisitor {
                formula_name: &formula.name,
                expression: &formula.condition,
            };

            let condition = std::mem::replace(self, Expression::Bool(false, default_span.clone()));
            *self = condition.visit(&mut visitor);
        }

        Ok(())
    }

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

#[derive(Clone)]
pub struct UnknownVariableError<S: Span> {
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
