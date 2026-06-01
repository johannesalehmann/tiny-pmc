use crate::expressions::DefaultMapExpression;
use crate::spans::Span;
use crate::{Expression, Formula, FormulaManager, Identifier};
use std::collections::HashMap;

/// A formula index and its [`Span`].
///
/// This is returned by [`FormulaManager::spanned_formulas_in_expression()`]. The index is with
/// respect to this formula manager.
pub struct SpannedDependency<S> {
    /// The index of the formula in the [`FormulaManager`].
    pub dependency: usize,

    /// The [`Span`] of the formula within the expression given to
    /// [`FormulaManager::spanned_formulas_in_expression()`].
    pub span: S,
}

impl<S> SpannedDependency<S> {
    /// Constructs a new `SpannedDependency` with the given parameters.
    pub fn new(dependency: usize, span: S) -> Self {
        Self { dependency, span }
    }
}

pub struct FormulaCountingVisitor<'a, S: Span> {
    formulas: &'a Vec<Formula<S, Expression<Identifier<S>, S>>>,
    found_formulas: Vec<SpannedDependency<S>>,
}

impl<'a, S: Span> DefaultMapExpression<Identifier<S>, S, ()> for FormulaCountingVisitor<'a, S> {
    fn visit_var_or_const(&mut self, name: Identifier<S>, span: S) -> () {
        for (i, formula) in self.formulas.iter().enumerate() {
            if formula.name == name {
                if !self.found_formulas.iter().any(|f| f.dependency == i) {
                    self.found_formulas.push(SpannedDependency::new(i, span));
                }
                break;
            }
        }
    }
}

impl<S: Span> FormulaManager<S, Expression<Identifier<S>, S>> {
    /// Returns a list of the formulas occurring in the given expression.
    ///
    /// This is useful to determine which formulas an expression depends on, which in turn can be
    /// used e.g. to expand formulas in the right order.
    pub fn spanned_formulas_in_expression(
        &self,
        expression: Expression<Identifier<S>, S>,
    ) -> Vec<SpannedDependency<S>> {
        let mut formulas = HashMap::new();
        for formula in &self.formulas {
            formulas.insert(formula.name.name.clone(), false);
        }
        let mut visitor = FormulaCountingVisitor {
            formulas: &self.formulas,
            found_formulas: Vec::new(),
        };

        expression.visit(&mut visitor);

        visitor.found_formulas
    }
}

impl<S: Span> FormulaManager<S, Expression<Identifier<S>, S>> {
    /// Computes the order in which formulas can be expanded.
    ///
    /// Returns a vector of formula indices. Expanding formulas in this order ensures that
    /// dependencies between formulas are resolved correctly. To this end, a dependency graph
    /// between formulas is constructed.
    ///
    /// # Example
    ///
    /// Consider these formulas:
    ///
    /// ```prism
    /// formula a = 123 / 7;
    /// formula b = a / c;
    /// formula c = a * 2;
    /// ```
    ///
    /// A suitable expansion order is `["a", "c", "b"]`, as `c` depends on `a` and `b` depends on
    /// `a` and `c`.
    ///
    /// # Errors
    ///
    /// If there is a cyclic dependency, [`CyclicDependency`] is returned, which includes details
    /// on the cycle.
    pub fn get_formula_replacement_ordering(&self) -> Result<Vec<usize>, CyclicDependency<S>> {
        let mut dependencies = self
            .formulas
            .iter()
            .map(|f| DependencyData::new(self.spanned_formulas_in_expression(f.condition.clone())))
            .collect::<Vec<_>>();

        let mut output = Vec::new();

        let mut stack: Vec<TraversalData<S>> = Vec::new();
        for i in 0..dependencies.len() {
            if dependencies[i].was_visited {
                continue;
            }

            stack.push(TraversalData {
                dep_index: i,
                next_child: 0,
                span_in_parent_expression: None,
            });

            while let Some(data) = stack.last_mut() {
                let deps = &dependencies[data.dep_index];
                if deps.was_visited && deps.was_emitted {
                    stack.pop();
                } else if data.next_child == 0 && deps.was_visited && !deps.was_emitted {
                    let mut cycle = Vec::new();
                    let mut previous_span =
                        data.span_in_parent_expression.as_ref().unwrap().clone();
                    let first_index = data.dep_index;

                    for element in stack.iter().rev().skip(1) {
                        cycle.push(CyclicDependencyEntry {
                            formula_name: self.formulas[element.dep_index].name.clone(),
                            formula_span: self.formulas[element.dep_index].span.clone(),
                            dependency_span: previous_span,
                        });
                        if element.dep_index == first_index {
                            break;
                        } else {
                            previous_span =
                                element.span_in_parent_expression.as_ref().unwrap().clone();
                        }
                    }

                    return Err(CyclicDependency { entries: cycle });
                } else {
                    dependencies[data.dep_index].was_visited = true;
                    if data.next_child < dependencies[data.dep_index].dependencies.len() {
                        let child_index =
                            dependencies[data.dep_index].dependencies[data.next_child].dependency;
                        let span_in_parent_expression = Some(
                            dependencies[data.dep_index].dependencies[data.next_child]
                                .span
                                .clone(),
                        );
                        let child = TraversalData {
                            dep_index: child_index,
                            next_child: 0,
                            span_in_parent_expression,
                        };
                        data.next_child += 1;
                        stack.push(child);
                    } else {
                        let index = stack.pop().unwrap().dep_index;
                        output.push(index);
                        dependencies[index].was_emitted = true;
                    }
                }
            }
        }

        output.reverse();
        Ok(output)
    }
}

/// Error produced by [`FormulaManager::get_formula_replacement_ordering()`], indicating that there
/// is a cycle in formula dependencies.
///
/// # Example
///
/// These formulas have a cyclic dependency, because `a` depends on `c`, which depends on `b`, which
/// depends on `a`.
///
/// ```prism
/// formula a = c + 5;
/// formula b = a / 2;
/// formula c = b * 2;
/// ```
// TODO: Describe how the cyclic dependency of these formulas looks
#[derive(Debug, PartialEq)]
pub struct CyclicDependency<S: Span> {
    /// The list of formulas that cyclically depend on each other.
    ///
    /// An entry with index `i + 1` depends on entry `i` and entry `0` depends on the last entry in
    /// `entries`. Note that `entries` may contain just one entry if a formula depends on itself.
    pub entries: Vec<CyclicDependencyEntry<S>>,
}

/// An entry in [`CyclicDependency`], containing details on a single formula.
#[derive(Debug, PartialEq)]
pub struct CyclicDependencyEntry<S: Span> {
    /// The name of the formula
    pub formula_name: Identifier<S>,

    /// The span of the formula's definition
    pub formula_span: S,

    /// The part of the formula's definition that refers to the previous entry in
    /// [`CyclicDependency`].
    pub dependency_span: S,
}

struct DependencyData<S: Clone> {
    dependencies: Vec<SpannedDependency<S>>,
    was_emitted: bool,
    was_visited: bool,
}

impl<S: Clone> DependencyData<S> {
    pub fn new(dependencies: Vec<SpannedDependency<S>>) -> Self {
        Self {
            dependencies,
            was_emitted: false,
            was_visited: false,
        }
    }
}

struct TraversalData<S: Clone> {
    dep_index: usize,
    next_child: usize,
    span_in_parent_expression: Option<S>,
}
