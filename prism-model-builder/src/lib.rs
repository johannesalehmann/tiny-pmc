pub use probabilistic_models;

pub mod expressions;
mod synchronised_actions;
mod variables;

use crate::expressions::stack_based_expressions::{
    StackBasedExpression, SubExpressionManager, SubExpressionManagerWithCache,
    SubExpressionProvider,
};
use crate::expressions::{TreeWalkingEvaluator, ValuationSource, VariableType};
use crate::synchronised_actions::{SynchronisedAction, SynchronisedActions};
use crate::variables::{ConstAndVarValuationSource, ModelVariableInfo};
use log::info;
use prism_model::{
    Command, Expression, Identifier, Model, Span, Update, VariableManager, VariableRange,
    VariableReference,
};
use probabilistic_models::builder;
use probabilistic_models::valuations::{
    BareStandaloneValuation, GetValuationClassIndex, GetValuationData, StandaloneValuation,
    ValuationBits, ValuationBitsMut, ValuationEntry,
};
use probabilistic_properties::Query;
use std::collections::{HashMap, VecDeque};
use typed_index_collections::{Index, RawIndex, index};

pub use typed_index_collections::To1;

index!(AtomicPropositionIndex);

pub fn build_model<
    S: Span,
    Base: builder::BaseModelBuilder,
    Ini: builder::InitialStatesBuilder<StateIdx = Base::StateIdx>,
    APs: builder::AtomicPropositionBuilder<StateIdx = Base::StateIdx>,
    M: Into<builder::ModelBuilder<Base, Ini, APs>>,
    Raw: RawIndex,
    I: Iterator<
        Item = Query<
            Expression<VariableReference, S>,
            Expression<VariableReference, S>,
            AtomicPropositionIndex<Raw>,
        >,
    >,
>(
    prism_model: &mut Model<VariableReference, S, Expression<VariableReference, S>, Identifier<S>>,
    explicit_builder: M,
    atomic_propositions: &To1<AtomicPropositionIndex<Raw>, Expression<VariableReference, S>>,
    properties: I,
    user_provided_consts: &HashMap<String, UserProvidedConstValue>,
) -> Result<ModelBuildingOutput<Base, Ini, APs, AtomicPropositionIndex<Raw>>, ModelBuildingError> {
    ExplicitModelBuilder::<Base, Ini, APs>::build_model::<S, M, Raw, I>(
        prism_model,
        explicit_builder.into(),
        atomic_propositions,
        properties,
        user_provided_consts,
    )
}

pub enum UserProvidedConstValue {
    Int(i64),
    Bool(bool),
    Float(f64),
}

pub struct ModelBuildingOutput<
    Base: builder::BaseModelBuilder,
    Ini: builder::InitialStatesBuilder,
    APs: builder::AtomicPropositionBuilder,
    APIdx: Index,
> {
    pub model: builder::ModelBuilderOutput<Base, Ini, APs>,
    pub properties: Vec<Query<i64, f64, APIdx>>,
}

pub trait ExpressionContext<E> {
    fn reset_context(&mut self);

    fn evaluate_int<V: ValuationSource>(&mut self, expression: &E, valuations: &V) -> i64;
    fn evaluate_int_with_separate_context<V: ValuationSource>(
        &self,
        expression: &E,
        valuations: &V,
    ) -> i64;
    fn evaluate_float<V: ValuationSource>(&mut self, expression: &E, valuations: &V) -> f64;
    fn evaluate_float_with_separate_context<V: ValuationSource>(
        &self,
        expression: &E,
        valuations: &V,
    ) -> f64;
    fn evaluate_bool<V: ValuationSource>(&mut self, expression: &E, valuations: &V) -> bool;
    fn evaluate_bool_with_separate_context<V: ValuationSource>(
        &self,
        expression: &E,
        valuations: &V,
    ) -> bool;
}

pub struct SubExpressionExpressionContext<'a, SE: SubExpressionProvider> {
    sub_expressions: &'a SE,
    context: SE::EvaluationContext,
}

impl<'a, SE: SubExpressionProvider> ExpressionContext<usize>
    for SubExpressionExpressionContext<'a, SE>
{
    fn reset_context(&mut self) {
        self.sub_expressions.reset_context(&mut self.context);
    }

    fn evaluate_int<V: ValuationSource>(&mut self, expression: &usize, valuations: &V) -> i64 {
        self.sub_expressions
            .evaluate_as_int(*expression, valuations, &mut self.context)
    }

    fn evaluate_int_with_separate_context<V: ValuationSource>(
        &self,
        expression: &usize,
        valuations: &V,
    ) -> i64 {
        let mut context = self.sub_expressions.create_context();
        self.sub_expressions
            .evaluate_as_int(*expression, valuations, &mut context)
    }

    fn evaluate_float<V: ValuationSource>(&mut self, expression: &usize, valuations: &V) -> f64 {
        self.sub_expressions
            .evaluate_as_float(*expression, valuations, &mut self.context)
    }

    fn evaluate_float_with_separate_context<V: ValuationSource>(
        &self,
        expression: &usize,
        valuations: &V,
    ) -> f64 {
        let mut context = self.sub_expressions.create_context();
        self.sub_expressions
            .evaluate_as_float(*expression, valuations, &mut context)
    }

    fn evaluate_bool<V: ValuationSource>(&mut self, expression: &usize, valuations: &V) -> bool {
        self.sub_expressions
            .evaluate_as_bool(*expression, valuations, &mut self.context)
    }

    fn evaluate_bool_with_separate_context<V: ValuationSource>(
        &self,
        expression: &usize,
        valuations: &V,
    ) -> bool {
        let mut context = self.sub_expressions.create_context();
        self.sub_expressions
            .evaluate_as_bool(*expression, valuations, &mut context)
    }
}

pub struct ExplicitModelBuilder<
    Base: builder::BaseModelBuilder,
    Ini: builder::InitialStatesBuilder<StateIdx = Base::StateIdx>,
    APs: builder::AtomicPropositionBuilder<StateIdx = Base::StateIdx>,
> {
    explicit_builder: builder::ModelBuilder<Base, Ini, APs>,
    open_states: VecDeque<Base::StateIdx>,
    variable_info: ModelVariableInfo<Base::ClassIdx, Base::ClassEntryIdx>,
}

impl<
    Base: builder::BaseModelBuilder,
    Ini: builder::InitialStatesBuilder<StateIdx = Base::StateIdx>,
    APs: builder::AtomicPropositionBuilder<StateIdx = Base::StateIdx>,
> ExplicitModelBuilder<Base, Ini, APs>
{
    fn build_properties<
        S: Span,
        Raw: RawIndex,
        I: Iterator<
            Item = Query<
                Expression<VariableReference, S>,
                Expression<VariableReference, S>,
                AtomicPropositionIndex<Raw>,
            >,
        >,
    >(
        properties: I,
        variable_info: &variables::ModelVariableInfo<Base::ClassIdx, Base::ClassEntryIdx>,
    ) -> Result<Vec<Query<i64, f64, AtomicPropositionIndex<Raw>>>, ModelBuildingError> {
        let const_valuation_source = variable_info.get_const_only_valuation_source();

        let mut result = Vec::new();
        for property in properties {
            result.push(
                property
                    .map_i(&mut |ex| {
                        TreeWalkingEvaluator::new().evaluate_as_int(&ex, &const_valuation_source)
                    })
                    .map_f(&mut |ex| {
                        TreeWalkingEvaluator::new().evaluate_as_float(&ex, &const_valuation_source)
                    }),
            );
        }
        Ok(result)
    }

    pub fn build_model<
        S: Span,
        M: Into<builder::ModelBuilder<Base, Ini, APs>>,
        Raw: RawIndex,
        I: Iterator<
            Item = Query<
                Expression<VariableReference, S>,
                Expression<VariableReference, S>,
                AtomicPropositionIndex<Raw>,
            >,
        >,
    >(
        model: &mut Model<VariableReference, S, Expression<VariableReference, S>, Identifier<S>>,
        mut explicit_builder: builder::ModelBuilder<Base, Ini, APs>,
        atomic_propositions: &To1<AtomicPropositionIndex<Raw>, Expression<VariableReference, S>>,
        properties: I,
        user_provided_consts: &HashMap<String, UserProvidedConstValue>,
    ) -> Result<ModelBuildingOutput<Base, Ini, APs, AtomicPropositionIndex<Raw>>, ModelBuildingError>
    {
        let start_time = std::time::Instant::now();

        model.replace_empty_updates_with_identity_update();

        let mut sub_expression_manager = SubExpressionManager::new();
        let model = model.map_expressions_cloned(|e| {
            let stack = StackBasedExpression::from_expression(e, &model.variable_manager);
            let sub_expression_index = sub_expression_manager.add_sub_expression(stack);
            sub_expression_index
        });

        let atomic_propositions = atomic_propositions
            .map(|ap| {
                let stack = StackBasedExpression::from_expression(ap, &model.variable_manager);
                let sub_expression = sub_expression_manager.add_sub_expression(stack);
                sub_expression
            })
            .change_key_type::<APs::AnnotationIdx>();
        for (index, _ap) in atomic_propositions.enumerate() {
            // TODO: Do this in a nicer way (perhaps find some way to do the key type change
            //  at the same time as this operation?
            // TODO: Get proper atomic proposition names
            let ap_index = explicit_builder
                .atomic_propositions
                .register_atomic_proposition(format!("ap_{}", index.raw()));
            assert_eq!(index, ap_index);
        }

        let mut sub_expression_cache = SubExpressionManagerWithCache::new(sub_expression_manager);
        let context = sub_expression_cache.create_context();
        let mut expression_context = SubExpressionExpressionContext {
            sub_expressions: &sub_expression_cache,
            context,
        };

        let variable_info = variables::ModelVariableInfo::new(
            &model,
            user_provided_consts,
            &mut expression_context,
            explicit_builder.base.valuation_builder_mut(),
        )?;

        sub_expression_cache
            .manager
            .optimise_expressions(&variable_info);

        // TODO: Reconstructing the context here is a bit unclean, but avoiding this requires some
        //  reorganisation to satisfy the borrow-checker.

        let context = sub_expression_cache.create_context();
        let mut expression_context = SubExpressionExpressionContext {
            sub_expressions: &sub_expression_cache,
            context,
        };

        let properties = Self::build_properties(properties, &variable_info)?;

        let synchronised_actions = SynchronisedActions::from_prism(&model);

        let mut builder = Self {
            explicit_builder,
            open_states: VecDeque::new(),
            variable_info,
        };

        builder.create_initial_states(&model, &mut expression_context)?;

        while let Some(state) = builder.open_states.pop_front() {
            builder.process_state(
                state,
                &model,
                &atomic_propositions,
                &synchronised_actions,
                &mut expression_context,
            )?;
        }

        let model = builder.explicit_builder.finish();

        info!(
            "Model built in {:?} ({} states (number of states is TODO)",
            start_time.elapsed(),
            0 // TODO
        );
        Ok(ModelBuildingOutput { model, properties })
    }

    // This method cannot take `self` to placate the borrow checker (we want to feed values of
    // `apply_valuation` into this function, which use disjoint parts of `self`, but the borrow
    // checker does not know these parts are disjoint unless we pass the parts into the respective
    // functions individually.
    fn get_or_add_state<
        Val: GetValuationData<Base::ValuationIdx> + GetValuationClassIndex<Base::ClassIdx>,
    >(
        explicit_builder: &mut builder::ModelBuilder<Base, Ini, APs>,
        open_states: &mut VecDeque<Base::StateIdx>,
        valuation: Val,
    ) -> Base::StateIdx {
        let index = explicit_builder.base.state_by_valuation(&valuation);
        match index {
            Some(index) => index,
            None => {
                let index = explicit_builder.preregister_state(valuation);
                open_states.push_back(index);
                index
            }
        }
    }

    fn process_state<S: Span, E, EC: ExpressionContext<E>>(
        &mut self,
        state: Base::StateIdx,
        model: &Model<VariableReference, S, E, Identifier<S>>,
        atomic_propositions: &To1<APs::AnnotationIdx, E>,
        synchronised_actions: &SynchronisedActions,
        expression_context: &mut EC,
    ) -> Result<(), ModelBuildingError> {
        expression_context.reset_context();
        self.explicit_builder.add_state(state);

        self.evaluate_atomic_propositions(state, atomic_propositions, expression_context);

        let mut choices_added = 0;

        for module in model.modules.iter() {
            for command_index in 0..module.commands.len() {
                let command = &module.commands[command_index];
                if command.action.is_some() {
                    continue; // Synchronising actions are handled separately
                }
                choices_added += self.process_nonsynchronised_command(
                    state,
                    model,
                    &command,
                    expression_context,
                );
            }
        }

        for synchronised_action in synchronised_actions {
            choices_added += self.process_synchronising_action(
                state,
                model,
                &synchronised_action,
                expression_context,
            );
        }
        if choices_added == 0 {
            // Fix deadlocks:
            self.explicit_builder.base.add_choice();
            self.explicit_builder.base.add_branch(1.0, state);
            self.explicit_builder.base.finish_branch();
            self.explicit_builder.base.finish_choice();
        }

        Ok(())
    }

    fn process_nonsynchronised_command<S: Span, E, EC: ExpressionContext<E>>(
        &mut self,
        state: Base::StateIdx,
        model: &Model<VariableReference, S, E, Identifier<S>>,
        command: &Command<VariableReference, S, E, Identifier<S>>,
        expression_context: &mut EC,
    ) -> usize {
        let valuation = &self.explicit_builder.base.state_valuations().entry(state);
        let val_source = self.variable_info.get_valuation_source(valuation);
        let guard = expression_context.evaluate_bool(&command.guard, &val_source);
        if guard {
            self.explicit_builder.base.add_choice();
            for update_index in 0..command.updates.len() {
                let valuation = &self.explicit_builder.base.state_valuations().entry(state);
                let val_source = self.variable_info.get_valuation_source(valuation);

                let update = &command.updates[update_index];
                let probability =
                    expression_context.evaluate_float(&update.probability, &val_source);
                let new_valuation = Self::apply_assignments(
                    &self.variable_info,
                    &model.variable_manager,
                    valuation,
                    &val_source,
                    &[&update],
                    expression_context,
                );

                let index = Self::get_or_add_state(
                    &mut self.explicit_builder,
                    &mut self.open_states,
                    new_valuation,
                );
                self.explicit_builder.base.add_branch(probability, index);

                // TODO: Add predecessors to model here?

                self.explicit_builder.base.finish_branch();
            }

            // TODO: Add choice label

            self.explicit_builder.base.finish_choice();
            1 // 1 new choice was added to the model
        } else {
            0 // 0 new choices were added to the model
        }
    }

    fn process_synchronising_action<S: Span, E, EC: ExpressionContext<E>>(
        &mut self,
        state: Base::StateIdx,
        model: &Model<VariableReference, S, E, Identifier<S>>,
        synchronised_action: &SynchronisedAction,
        expression_context: &mut EC,
    ) -> usize {
        // TODO: Determine choice label here (and use it below?)

        let valuation = &self.explicit_builder.base.state_valuations().entry(state);
        let val_source = self.variable_info.get_valuation_source(valuation);

        let mut satisfied_guards_indices = Vec::new();
        let mut all_satisfied = true;
        for action_module in &synchronised_action.participating_modules {
            let module = model.modules.get(action_module.module_index).unwrap();
            let mut module_info = Vec::new();
            for &command_index in &action_module.command_indices {
                let command = &module.commands[command_index];
                let guard = expression_context.evaluate_bool(&command.guard, &val_source);
                if guard {
                    module_info.push(command_index);
                }
            }
            if module_info.is_empty() {
                all_satisfied = false;
            }
            satisfied_guards_indices.push(module_info);
        }

        let n = satisfied_guards_indices.len();

        if n == 0 {
            panic!(
                "Synchronised actions with zero associated modules are not yet supported (but they should be impossible to create anyways)"
            );
        }

        let mut choices_added = 0;

        if all_satisfied {
            let modules = &synchronised_action.participating_modules;
            let mut indices = vec![0; n];
            while indices[0] < satisfied_guards_indices[0].len() {
                self.explicit_builder.base.add_choice();

                let mut command_indices = Vec::with_capacity(n);
                for i in 0..n {
                    command_indices.push(satisfied_guards_indices[i][indices[i]]);
                }

                let mut update_indices = vec![0; n];

                while update_indices[0]
                    < model.modules.get(modules[0].module_index).unwrap().commands
                        [command_indices[0]]
                        .updates
                        .len()
                        .max(1)
                // max(1) is required because a synchronising action may have an empty update ("true")
                {
                    let valuation = self.explicit_builder.base.state_valuations().entry(state);
                    let val_source = self.variable_info.get_valuation_source(&valuation);
                    let mut updates = Vec::new(); // TODO: Avoid allocating a vector here?
                    for i in 0..n {
                        let command = &model.modules.get(modules[i].module_index).unwrap().commands
                            [command_indices[i]];
                        if command.updates.len() > 0 {
                            updates.push(&command.updates[update_indices[i]]);
                        }
                    }
                    let new_valuation = Self::apply_assignments(
                        &self.variable_info,
                        &model.variable_manager,
                        &valuation,
                        &val_source,
                        &updates[..],
                        expression_context,
                    );

                    let mut probability = 1.0;

                    for i in 0..n {
                        let command = &model.modules.get(modules[i].module_index).unwrap().commands
                            [command_indices[i]];
                        if command.updates.len() > 0 {
                            let ith_expression = &command.updates[update_indices[i]].probability;
                            let ith_probability =
                                expression_context.evaluate_float(ith_expression, &val_source);
                            probability *= ith_probability;
                        }
                    }

                    let target = Self::get_or_add_state(
                        &mut self.explicit_builder,
                        &mut self.open_states,
                        new_valuation,
                    );
                    self.explicit_builder.base.add_branch(probability, target);
                    self.explicit_builder.base.finish_branch();

                    // TODO: Set predecessor here?

                    for i in (0..n).rev() {
                        if update_indices[i] + 1
                            < model.modules.get(modules[i].module_index).unwrap().commands
                                [command_indices[i]]
                                .updates
                                .len()
                        {
                            update_indices[i] += 1;
                            for j in i + 1..n {
                                update_indices[j] = 0;
                            }
                            break;
                        } else {
                            if i == 0 {
                                update_indices[0] += 1;
                            }
                        }
                    }
                }

                choices_added += 1;
                self.explicit_builder.base.finish_choice();

                for i in (0..n).rev() {
                    if indices[i] + 1 < satisfied_guards_indices[i].len() {
                        indices[i] += 1;
                        for j in i + 1..n {
                            indices[j] = 0;
                        }
                        break;
                    } else {
                        if i == 0 {
                            indices[0] += 1;
                        }
                    }
                }
            }
        }

        choices_added
    }

    fn evaluate_atomic_propositions<E, EC: ExpressionContext<E>>(
        &mut self,
        state_index: Base::StateIdx,
        atomic_propositions: &To1<APs::AnnotationIdx, E>,
        expression_context: &mut EC,
    ) {
        let valuation = self
            .explicit_builder
            .base
            .state_valuations()
            .entry(state_index);
        let val_source = self.variable_info.get_valuation_source(&valuation);
        for (i, atomic_proposition) in atomic_propositions.enumerate() {
            let is_true = expression_context.evaluate_bool(atomic_proposition, &val_source);
            self.explicit_builder
                .atomic_propositions
                .set_value(i, state_index, is_true);
        }
    }

    fn apply_assignments<S: Span, E, EC: ExpressionContext<E>>(
        variable_info: &ModelVariableInfo<Base::ClassIdx, Base::ClassEntryIdx>,
        variable_manager: &VariableManager<S, E>,
        valuation: &ValuationEntry<'_, Base::ClassIdx, Base::ClassEntryIdx, Base::ValuationIdx>,
        val_source: &ConstAndVarValuationSource<
            Base::ClassIdx,
            Base::ClassEntryIdx,
            Base::ValuationIdx,
        >,
        updates: &[&Update<VariableReference, S, E>],
        expression_context: &mut EC,
    ) -> BareStandaloneValuation<Base::ClassIdx, Base::ValuationIdx> {
        let mut new_valuation = valuation.clone_into_standalone_valuation();
        for update in updates {
            for assignment in &update.assignments {
                let target = variable_manager.get(&assignment.target).unwrap();
                let target_index = variable_info
                    .valuation_map
                    .map_to_variable(assignment.target.index)
                    .expect("Cannot assign to constant");
                match target.range {
                    VariableRange::BoundedInt { .. } => {
                        let value = expression_context.evaluate_int(&assignment.value, &val_source);
                        let (min, max) = variable_info.details[target_index].bounds.unwrap();
                        if value < min || value > max {
                            panic!(
                                "Value for {} exceeds variable bounds, bounds are ({}, {}), value is {}",
                                variable_manager.variables[assignment.target.index].name,
                                min,
                                max,
                                value
                            );
                        } else {
                            new_valuation.set_int(target_index, value);
                        }
                    }
                    VariableRange::UnboundedInt { .. } => {
                        let value = expression_context.evaluate_int(&assignment.value, &val_source);
                        new_valuation.set_int(target_index, value);
                    }
                    VariableRange::Boolean { .. } => {
                        let value =
                            expression_context.evaluate_bool(&assignment.value, &val_source);
                        new_valuation.set_bool(target_index, value);
                    }
                    VariableRange::Float { .. } => {
                        let value =
                            expression_context.evaluate_float(&assignment.value, &val_source);
                        new_valuation.set_double(target_index, value);
                    }
                }
            }
        }
        new_valuation.into()
    }

    fn create_initial_states<S: Span, E, EC: ExpressionContext<E>>(
        &mut self,
        model: &Model<VariableReference, S, E, Identifier<S>>,
        expression_context: &mut EC,
    ) -> Result<(), ModelBuildingError> {
        if model.init_constraint.is_some() {
            panic!("Init constraints are not yet supported by the model builder");
        }
        let const_value_source = self.variable_info.get_const_only_valuation_source();

        // TODO: This call is very verbose -- perhaps there is some restructuring that makes it
        //  simpler?
        let mut valuation = StandaloneValuation::new(
            self.variable_info.class_index,
            self.explicit_builder
                .base
                .state_valuations()
                .class(self.variable_info.class_index),
        );

        for (i, variable) in model.variable_manager.variables.iter().enumerate() {
            if let Some(index) = &self.variable_info.valuation_map.map_to_variable(i) {
                match variable.range {
                    VariableRange::BoundedInt { .. } => match &variable.initial_value {
                        None => {
                            if let Some((min, _)) = self.variable_info.details[*index].bounds {
                                valuation.set_int(*index, min);
                            } else {
                                panic!("Variable bounds list is inconsistent");
                            }
                        }
                        Some(initial) => {
                            let value =
                                expression_context.evaluate_int(initial, &const_value_source);
                            valuation.set_int(*index, value);
                        }
                    },
                    VariableRange::UnboundedInt { .. } => match &variable.initial_value {
                        None => panic!("Unbounded int must have init expression"),
                        Some(initial) => {
                            let value =
                                expression_context.evaluate_int(initial, &const_value_source);
                            valuation.set_int(*index, value);
                        }
                    },
                    VariableRange::Boolean { .. } => match &variable.initial_value {
                        None => {
                            valuation.set_bool(*index, false);
                        }
                        Some(initial) => {
                            let value =
                                expression_context.evaluate_bool(initial, &const_value_source);
                            valuation.set_bool(*index, value);
                        }
                    },
                    VariableRange::Float { .. } => match &variable.initial_value {
                        None => {
                            panic!(
                                "Floats must have init expressions (I'm not sure whether this is PRISM-spec-compliant)"
                            )
                        }
                        Some(initial) => {
                            let value =
                                expression_context.evaluate_float(initial, &const_value_source);
                            valuation.set_double(*index, value);
                        }
                    },
                }
            }
        }
        let valuation = valuation.bare();

        let index =
            Self::get_or_add_state(&mut self.explicit_builder, &mut self.open_states, valuation);

        self.explicit_builder.initial_states.mark_state(index);

        Ok(())
    }

    #[allow(unused)]
    fn print_valuation<S: Span>(
        valuation: ValuationEntry<'_, Base::ClassIdx, Base::ClassEntryIdx, Base::ValuationIdx>,
        variable_info: &ModelVariableInfo<Base::ClassIdx, Base::ClassEntryIdx>,
        model: &Model<VariableReference, S, Expression<VariableReference, S>, Identifier<S>>,
    ) {
        print!("(");
        let mut first = true;
        for (i, var) in model.variable_manager.variables.iter().enumerate() {
            if let Some(index) = variable_info.valuation_map.map_to_variable(i) {
                if !first {
                    print!(", ");
                }
                first = false;
                print!("{} = ", var.name);
                match var.range {
                    VariableRange::BoundedInt { .. } => {
                        print!("{}", valuation.evaluate_int(index))
                    }
                    VariableRange::UnboundedInt { .. } => {
                        print!("{}", valuation.evaluate_int(index))
                    }
                    VariableRange::Boolean { .. } => {
                        print!("{}", valuation.evaluate_bool(index))
                    }
                    VariableRange::Float { .. } => {
                        print!("{}", valuation.evaluate_double(index))
                    }
                }
            }
        }
        print!(")");
    }
}

#[derive(Debug)]
pub enum ModelBuildingError {}
