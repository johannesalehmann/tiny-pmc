mod formula_substitution;

mod map_expression_cloned;
mod map_expression_into;
mod map_expressions;
mod renamed_module_expansion;
mod variable_indexing;

pub use formula_substitution::FormulaSubstitutionVisitor;
pub use renamed_module_expansion::ModuleExpansionError;

use crate::formulas::FormulaManager;
use crate::module::RenamedModule;
use crate::rewards::RewardsManager;
use crate::spans::{FullSpan, Span};
use crate::{
    Displayable, Expression, Identifier, LabelManager, ModuleManager, VariableInfo,
    VariableManager, VariablePrintingStyle, VariableRange, VariableReference,
};
use std::fmt::{Display, Formatter};

pub type ModelNamedVars<S: Span = FullSpan, A = Identifier<S>> =
    Model<Identifier<S>, S, Expression<Identifier<S>, S>, A>;

pub struct Model<V = VariableReference, S: Span = FullSpan, E = Expression<V, S>, A = Identifier<S>>
{
    pub model_type: ModelType<S>,

    pub variable_manager: VariableManager<S, E>,
    pub formulas: FormulaManager<S, E>,

    pub modules: ModuleManager<V, S, E, A>,
    pub renamed_modules: Vec<RenamedModule<S>>,

    pub init_constraint: Option<E>,

    pub labels: LabelManager<S, E>,
    pub rewards: RewardsManager<S, E, A>,

    pub span: S,
}

impl<V, S: Span, E, A> Model<V, S, E, A> {
    pub fn new(model_type: ModelType<S>) -> Self {
        Self::new_spanned(model_type, S::empty())
    }

    pub fn new_spanned(model_type: ModelType<S>, span: S) -> Self {
        Self {
            model_type,
            variable_manager: VariableManager::new(),
            formulas: FormulaManager::new(),
            modules: ModuleManager::new(),
            renamed_modules: Vec::new(),
            init_constraint: None,
            labels: LabelManager::new(),
            rewards: RewardsManager::new(),
            span,
        }
    }

    pub fn from_components(
        model_type: ModelType<S>,
        variable_manager: VariableManager<S, E>,
        formulas: FormulaManager<S, E>,
        modules: ModuleManager<V, S, E, A>,
        renamed_modules: Vec<RenamedModule<S>>,
        init_constraint: Option<E>,
        labels: LabelManager<S, E>,
        rewards: RewardsManager<S, E, A>,
        span: S,
    ) -> Self {
        Self {
            model_type,
            variable_manager,
            formulas,
            modules,
            renamed_modules,
            init_constraint,
            labels,
            rewards,
            span,
        }
    }
}
impl<V, S: Span, E> Model<V, S, E, Identifier<S>> {
    pub fn name_unnamed_actions(&mut self) {
        self.name_unnamed_actions_with_custom_name(|i, _| format!("unnamed_action_{i}"))
    }

    pub fn name_unnamed_actions_with_custom_name<F: FnMut(usize, &S) -> String>(
        &mut self,
        mut name_function: F,
    ) {
        let mut counter = 0;
        for module in &mut self.modules.modules {
            for command in &mut module.commands {
                if command.action.is_none() {
                    command.action = Some(
                        crate::Identifier::new_potentially_reserved_spanned(
                            name_function(counter, &command.action_span),
                            command.action_span.clone(),
                        )
                        .unwrap(),
                    );
                    counter += 1;
                }
            }
        }
    }

    pub fn actually_synchronising_actions(&self) -> std::collections::HashSet<String> {
        use std::collections::HashSet;
        let mut seen_before = HashSet::new();
        let mut actually_synchronising = HashSet::new();
        for module in &self.modules.modules {
            let mut module_actions = HashSet::new();
            for command in &module.commands {
                if let Some(command) = &command.action {
                    if !module_actions.contains(&command.name) {
                        module_actions.insert(command.name.clone());
                    }
                }
            }

            for action in module_actions {
                if seen_before.contains(&action) {
                    actually_synchronising.insert(action);
                } else {
                    seen_before.insert(action);
                }
            }
        }

        actually_synchronising
    }
}

// TODO: This trait is only used to enable init_statements_to_init_block to work both when `V` is
//  `Identifier` and when `V` is `VariableReference`. Perhaps we can use some more general mechanism
//  or expose this trait more broadly?
pub trait VariableIdentifierProvider<S: Span, E> {
    fn get_variable_identifier(info: &VariableInfo<S, E>, index: usize) -> Self;
}

impl<S: Span, E> VariableIdentifierProvider<S, E> for VariableReference {
    fn get_variable_identifier(info: &VariableInfo<S, E>, index: usize) -> Self {
        let _ = info;
        VariableReference::new(index)
    }
}

impl<S: Span, E> VariableIdentifierProvider<S, E> for Identifier<S> {
    fn get_variable_identifier(info: &VariableInfo<S, E>, index: usize) -> Self {
        let _ = index;
        info.name.clone()
    }
}

impl<V, S: Span, A> Model<V, S, Expression<V, S>, A> {
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> Model<V, S2, Expression<V, S2>, A> {
        Model {
            model_type: self.model_type.map_span(map),
            variable_manager: self.variable_manager.map_span(map),
            formulas: self.formulas.map_span(map),
            modules: self.modules.map_span(map),
            renamed_modules: self
                .renamed_modules
                .into_iter()
                .map(|m| m.map_span(map))
                .collect(),
            init_constraint: self.init_constraint.map(|i| i.map_span(map)),
            labels: self.labels.map_span(map),
            rewards: self.rewards.map_span(map),
            span: map(self.span),
        }
    }

    pub fn replace_empty_updates_with_identity_update(&mut self) {
        for module in &mut self.modules.modules {
            for command in &mut module.commands {
                if command.updates.len() == 0 {
                    command.updates.push(crate::Update::new_spanned(
                        Expression::Float(1.0, command.span.clone()),
                        command.span.clone(),
                    )); // TODO: The expression's and update's span should only cover the `true` token, but its span is currently not tracked
                }
            }
        }
    }

    pub fn add_missing_init_statements(&mut self)
    where
        V: Clone,
    {
        if self.init_constraint.is_some() {
            panic!(
                "Cannot add missing init statements because the model uses an init constraint instead of init statements"
            );
        }

        for variable in &mut self.variable_manager.variables {
            if !variable.is_constant {
                if variable.initial_value.is_none() {
                    variable.initial_value = Some(match &variable.range {
                        VariableRange::BoundedInt { min, .. } => min.clone(),
                        VariableRange::UnboundedInt { .. } => {
                            panic!("Unbounded integers must have an initial value.")
                        }
                        VariableRange::Boolean { .. } => {
                            Expression::Bool(false, variable.range.span().clone())
                        }
                        VariableRange::Float { .. } => {
                            panic!("Unbounded integers must have an initial value.")
                        }
                    });
                }
            }
        }
    }

    pub fn init_statements_to_init_block(&mut self)
    where
        V: Clone + VariableIdentifierProvider<S, Expression<V, S>>,
    {
        // TODO: Fix how new spans are created
        if self.init_constraint.is_some() {
            panic!(
                "Cannot transform init statements to init block because the model already uses an init block"
            );
        }

        let mut init_constraint: Option<Expression<V, S>> = None;

        self.add_missing_init_statements();

        for (variable_index, variable) in self.variable_manager.variables.iter_mut().enumerate() {
            if !variable.is_constant {
                match std::mem::replace(&mut variable.initial_value, None) {
                    Some(value) => {
                        let identifier = V::get_variable_identifier(variable, variable_index);
                        let variable_constraint = Expression::Equals(
                            Box::new(Expression::VarOrConst(identifier, self.span.clone())),
                            Box::new(value),
                            self.span.clone(),
                        );
                        if let Some(prev_init) = init_constraint.take() {
                            let span = prev_init.span().clone();
                            init_constraint = Some(Expression::Conjunction(
                                Box::new(prev_init),
                                Box::new(variable_constraint),
                                span,
                            ));
                        } else {
                            init_constraint = Some(variable_constraint);
                        }
                        variable.initial_value = None;
                    }
                    None => {
                        panic!("Variable {} does not have initial value.", variable.name)
                    }
                }
            }
        }

        self.init_constraint = init_constraint;
    }
}

#[derive(Copy, Clone)]
pub enum ModelType<S: Span = FullSpan> {
    Dtmc(S),
    Ctmc(S),
    Mdp(S),
}

impl<S: Span> ModelType<S> {
    pub fn dtmc() -> Self {
        Self::Dtmc(S::empty())
    }

    pub fn dtmc_spanned(span: S) -> Self {
        Self::Dtmc(span)
    }

    pub fn ctmc() -> Self {
        Self::Ctmc(S::empty())
    }

    pub fn ctmc_spanned(span: S) -> Self {
        Self::Ctmc(span)
    }

    pub fn mdp() -> Self {
        Self::Mdp(S::empty())
    }

    pub fn mdp_spanned(span: S) -> Self {
        Self::Mdp(span)
    }

    pub fn get_span(&self) -> &S {
        match self {
            ModelType::Dtmc(s) => s,
            ModelType::Ctmc(s) => s,
            ModelType::Mdp(s) => s,
        }
    }

    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> ModelType<S2> {
        match self {
            ModelType::Dtmc(span) => ModelType::Dtmc(map(span)),
            ModelType::Ctmc(span) => ModelType::Ctmc(map(span)),
            ModelType::Mdp(span) => ModelType::Mdp(map(span)),
        }
    }
}

impl<S: Span> Display for ModelType<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Dtmc(_) => {
                write!(f, "dtmc")
            }
            ModelType::Ctmc(_) => {
                write!(f, "ctmc")
            }
            ModelType::Mdp(_) => {
                write!(f, "mdp")
            }
        }
    }
}

impl<A, E, V, S: Span> crate::private::Sealed for Model<V, S, E, A> {}
impl<Ctx, A: Display, E: Displayable<Ctx>, V: Displayable<Ctx>, S: Span> Displayable<Ctx>
    for Model<V, S, E, A>
{
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result {
        writeln!(f, "{}", self.model_type)?;
        writeln!(f, "")?;
        write!(
            f,
            "{}",
            self.variable_manager
                .displayable(&(VariablePrintingStyle::Const, &context))
        )?;
        write!(
            f,
            "{}",
            self.variable_manager
                .displayable(&(VariablePrintingStyle::GlobalVar, &context))
        )?;
        write!(f, "{}", self.formulas.displayable(context))?;
        write!(f, "{}", self.labels.displayable(context))?;
        if let Some(init) = &self.init_constraint {
            writeln!(f, "init")?;
            writeln!(f, "    {}", init.displayable(context))?;
            writeln!(f, "endinit")?;
        }
        for (i, module) in self.modules.modules.iter().enumerate() {
            writeln!(
                f,
                "{}",
                module.displayable(&(i, &self.variable_manager, context))
            )?;
        }
        for renamed_module in &self.renamed_modules {
            writeln!(f, "{}", renamed_module)?;
        }
        for rewards in &self.rewards.rewards {
            writeln!(f, "{}", rewards.displayable(context))?;
        }

        Ok(())
    }
}

impl<S: Span, A: Display> Display
    for Model<VariableReference, S, Expression<VariableReference, S>, A>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.displayable(&self.variable_manager).fmt(f)
    }
}

impl<S: Span, A: Display> Display for Model<Identifier<S>, S, Expression<Identifier<S>, S>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.displayable(&()).fmt(f)
    }
}
