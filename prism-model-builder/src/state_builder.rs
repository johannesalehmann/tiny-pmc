use crate::choice_labels::Context;
use crate::expression_context::ExpressionContext;
use crate::initial_states_source;
use crate::initial_states_source::InitialStateSource;
use crate::labels::Labels;
use crate::synchronised_actions::SynchronisedActions;
use crate::variables::ModelVariableInfo;
use crate::{ModelBuildingError, choice_labels};
use prism_model::{
    Assignment, Command, Expression, Identifier, Model, Span, Update, VariableManager,
    VariableRange, VariableReference,
};
use probabilistic_models::valuations::{
    BareStandaloneValuation, GetValuationClassIndex, GetValuationData, StandaloneValuation,
    ValuationBits, ValuationBitsMut, ValuationEntry,
};
use std::collections::VecDeque;
use std::iter::once;
use typed_index_collections::Index;

pub struct StateBuilder<
    'a,
    S: Span,
    E,
    EC: ExpressionContext<E>,
    Base: crate::bases::BaseModelBuilder,
    IniBuilder: crate::initial_states_builder::InitialStatesBuilder,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder,
    CL: choice_labels::ChoiceLabelBuilder<ChoiceIdx = Base::ChoiceIdx>,
> {
    pub synchronising_action: SynchronisedActions,
    pub labels: &'a Labels<APs::APIdx, E>,

    pub base: &'a mut Base,
    pub initial_states_builder: &'a mut IniBuilder,
    pub atomic_propositions: &'a mut APs,
    pub choice_labels: &'a mut CL,

    pub open_states: VecDeque<Base::StateIdx>,

    pub variables: StateBuilderVariables<'a, S, E, EC, Base>,
}

pub struct StateBuilderVariables<
    'a,
    S: Span,
    E,
    EC: ExpressionContext<E>,
    Base: crate::bases::BaseModelBuilder,
> {
    pub info: ModelVariableInfo<Base::ClassIdx, Base::ClassEntryIdx>,
    pub model: &'a Model<VariableReference, S, E, Identifier<S>>,
    pub expr_context: &'a mut EC,
}

impl<
    'a,
    S: Span,
    E,
    EC: ExpressionContext<E>,
    Base: crate::bases::BaseModelBuilder,
    IniBuilder: crate::initial_states_builder::InitialStatesBuilder<StateIdx = Base::StateIdx>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = Base::StateIdx>,
    CL: choice_labels::ChoiceLabelBuilder<ChoiceIdx = Base::ChoiceIdx>,
> initial_states_source::Context for StateBuilder<'a, S, E, EC, Base, IniBuilder, APs, CL>
{
    type Span = S;
    type Expression = E;
    type ExpressionContext = EC;
    type ClassIdx = Base::ClassIdx;
    type ClassEntryIdx = Base::ClassEntryIdx;
    type ValuationIdx = Base::ValuationIdx;

    fn info(
        &mut self,
    ) -> (
        &Model<VariableReference, Self::Span, Self::Expression, Identifier<Self::Span>>,
        &ModelVariableInfo<Self::ClassIdx, Self::ClassEntryIdx>,
        &mut Self::ExpressionContext,
    ) {
        (
            self.variables.model,
            &self.variables.info,
            self.variables.expr_context,
        )
    }

    fn create_valuation(
        &self,
    ) -> StandaloneValuation<'_, Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx> {
        self.base.create_valuation(self.variables.info.class_index)
    }

    fn add_state(
        &mut self,
        valuation: BareStandaloneValuation<Self::ClassIdx, Self::ValuationIdx>,
        is_initial: bool,
    ) {
        let index = Self::get_or_add_state(
            self.base,
            self.initial_states_builder,
            &mut self.open_states,
            valuation,
        );
        if is_initial {
            self.initial_states_builder.mark_state(index);
        }
    }
}

impl<
    'a,
    S: Span,
    E,
    EC: ExpressionContext<E>,
    Base: crate::bases::BaseModelBuilder,
    IniBuilder: crate::initial_states_builder::InitialStatesBuilder<StateIdx = Base::StateIdx>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = Base::StateIdx>,
    CL: choice_labels::ChoiceLabelBuilder<ChoiceIdx = Base::ChoiceIdx>,
> StateBuilder<'a, S, E, EC, Base, IniBuilder, APs, CL>
{
    pub fn create_initial_states<IniSource: InitialStateSource>(
        &mut self,
        initial_state_source: IniSource,
    ) -> Result<(), ModelBuildingError> {
        initial_state_source.mark_initial_states(self);
        Ok(())
    }

    pub fn expand_states(&mut self) -> Result<(), ModelBuildingError> {
        while let Some(state) = self.open_states.pop_front() {
            self.process_state(state)?;
        }
        Ok(())
    }
    fn process_state(&mut self, state: Base::StateIdx) -> Result<(), ModelBuildingError> {
        self.variables.expr_context.reset_context();
        self.base.add_state(state);

        self.evaluate_atomic_propositions(state);

        let mut choices_added = 0;

        for (module_index, module) in self.variables.model.modules.iter().enumerate() {
            for command_index in 0..module.commands.len() {
                let command = &module.commands[command_index];
                // TODO: We could handle commands with actions here, as long as they only occur in
                //  a single module.
                if command.action.is_some() {
                    continue; // Synchronising actions are handled separately
                }
                choices_added += self.process_nonsynchronised_command(
                    state,
                    module_index,
                    command_index,
                    &command,
                );
            }
        }

        for i in 0..self.synchronising_action.len() {
            choices_added += self.process_synchronising_action(state, i);
        }
        if choices_added == 0 {
            // Fix deadlocks: // TODO: Make this configurable
            let choice_index = self.base.start_choice();
            self.base.add_branch(1.0, state);
            self.base.finish_choice();

            let index = self.choice_labels.name_to_index(None);
            let context = CL::ContextType::new_deadlock_fix();
            self.choice_labels
                .label_choice(choice_index, &index, &context);
        }

        Ok(())
    }

    fn process_nonsynchronised_command(
        &mut self,
        state: Base::StateIdx,
        module_index: usize,
        command_index: usize,
        command: &'a Command<VariableReference, S, E, Identifier<S>>,
    ) -> usize {
        let valuation = &self.base.state_valuations().entry(state);
        let val_source = self.variables.info.get_valuation_source(valuation);
        let guard = self
            .variables
            .expr_context
            .evaluate_bool(&command.guard, &val_source);
        if guard {
            let choice_index = self.base.start_choice();
            for update_index in 0..command.updates.len() {
                let valuation = &self.base.state_valuations().entry(state);
                let val_source = self.variables.info.get_valuation_source(valuation);

                let update = &command.updates[update_index];
                let probability = self
                    .variables
                    .expr_context
                    .evaluate_float(&update.probability, &val_source);
                let new_valuation = self.variables.apply_assignments(valuation, once(update));

                let index = Self::get_or_add_state(
                    &mut self.base,
                    &mut self.initial_states_builder,
                    &mut self.open_states,
                    new_valuation,
                );
                self.base.add_branch(probability, index);
                // TODO: Add predecessors to model here?
            }
            let index = self
                .choice_labels
                .name_to_index(command.action.as_ref().map(|a| a.name.as_str()));
            let context = CL::ContextType::new_unsynchronised(module_index, command_index);
            self.choice_labels
                .label_choice(choice_index, &index, &context);

            self.base.finish_choice();
            1 // 1 new choice was added to the model
        } else {
            0 // 0 new choices were added to the model
        }
    }

    fn process_synchronising_action(
        &mut self,
        state: Base::StateIdx,
        synchronised_action_index: usize,
    ) -> usize {
        let synchronised_action = &self.synchronising_action[synchronised_action_index];

        let name = self
            .choice_labels
            .name_to_index(Some(&synchronised_action.name));

        let valuation = &self.base.state_valuations().entry(state);
        let val_source = self.variables.info.get_valuation_source(valuation);

        let mut satisfied_guards_indices = Vec::new();
        let mut all_satisfied = true;
        for action_module in &synchronised_action.participating_modules {
            let module = self
                .variables
                .model
                .modules
                .get(action_module.module_index)
                .unwrap();
            let mut module_info = Vec::new();
            for &command_index in &action_module.command_indices {
                let command = &module.commands[command_index];
                let guard = self
                    .variables
                    .expr_context
                    .evaluate_bool(&command.guard, &val_source);
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
            let mut choice_label_context = CL::ContextType::new_synchronised(n);
            while indices[0] < satisfied_guards_indices[0].len() {
                let choice_index = self.base.start_choice();

                let mut command_indices = Vec::with_capacity(n);
                for i in 0..n {
                    command_indices.push(satisfied_guards_indices[i][indices[i]]);
                }
                for i in 0..n {
                    choice_label_context.set_synchronised_component(
                        i,
                        modules[i].module_index,
                        command_indices[i],
                    );
                }
                self.choice_labels
                    .label_choice(choice_index, &name, &choice_label_context);

                let mut update_indices = vec![0; n];

                while update_indices[0]
                    < self
                        .variables
                        .model
                        .modules
                        .get(modules[0].module_index)
                        .unwrap()
                        .commands[command_indices[0]]
                        .updates
                        .len()
                        .max(1)
                // max(1) is required because a synchronising action may have an empty update ("true")
                {
                    let valuation = self.base.state_valuations().entry(state);
                    let mut updates = Vec::new(); // TODO: Avoid allocating a vector here?
                    for i in 0..n {
                        let command = &self
                            .variables
                            .model
                            .modules
                            .get(modules[i].module_index)
                            .unwrap()
                            .commands[command_indices[i]];
                        if command.updates.len() > 0 {
                            updates.push(&command.updates[update_indices[i]]);
                        }
                    }
                    let new_valuation = self.variables.apply_assignments(&valuation, updates);

                    let mut probability = 1.0;

                    let val_source = self.variables.info.get_valuation_source(&valuation);
                    for i in 0..n {
                        let command = &self
                            .variables
                            .model
                            .modules
                            .get(modules[i].module_index)
                            .unwrap()
                            .commands[command_indices[i]];
                        if command.updates.len() > 0 {
                            let ith_expression = &command.updates[update_indices[i]].probability;
                            let ith_probability = self
                                .variables
                                .expr_context
                                .evaluate_float(ith_expression, &val_source);
                            probability *= ith_probability;
                        }
                    }

                    let target = Self::get_or_add_state(
                        &mut self.base,
                        &mut self.initial_states_builder,
                        &mut self.open_states,
                        new_valuation,
                    );
                    self.base.add_branch(probability, target);

                    // TODO: Set predecessor here?

                    for i in (0..n).rev() {
                        if update_indices[i] + 1
                            < self
                                .variables
                                .model
                                .modules
                                .get(modules[i].module_index)
                                .unwrap()
                                .commands[command_indices[i]]
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
                self.base.finish_choice();

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

    fn evaluate_atomic_propositions(&mut self, state_index: Base::StateIdx) {
        let valuation = self.base.state_valuations().entry(state_index);
        let val_source = self.variables.info.get_valuation_source(&valuation);
        for (i, (_, atomic_proposition)) in self.labels.into_iter().enumerate() {
            let is_true = self
                .variables
                .expr_context
                .evaluate_bool(atomic_proposition, &val_source);
            self.atomic_propositions.set_value(i, state_index, is_true);
        }
    }

    pub fn get_or_add_state<
        Val: GetValuationData<Base::ValuationIdx> + GetValuationClassIndex<Base::ClassIdx>,
    >(
        base: &mut Base,
        initial_states_builder: &mut IniBuilder,
        open_states: &mut VecDeque<Base::StateIdx>,
        valuation: Val,
    ) -> Base::StateIdx {
        let index = base.state_by_valuation(&valuation);
        match index {
            Some(index) => index,
            None => {
                let index = base.add_valuation(valuation);
                open_states.push_back(index);
                initial_states_builder.state_added(index);
                index
            }
        }
    }

    #[allow(unused)]
    fn print_valuation(
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

impl<'a, S: Span, E, EC: ExpressionContext<E>, Base: crate::bases::BaseModelBuilder>
    StateBuilderVariables<'a, S, E, EC, Base>
{
    fn apply_assignments(
        &mut self,
        valuation: &ValuationEntry<'_, Base::ClassIdx, Base::ClassEntryIdx, Base::ValuationIdx>,
        updates: impl IntoIterator<Item = &'a Update<VariableReference, S, E>>,
    ) -> BareStandaloneValuation<Base::ClassIdx, Base::ValuationIdx> {
        let mut new_valuation = valuation.clone_into_standalone_valuation();
        for update in updates {
            new_valuation.apply_assignments(
                &update.assignments,
                valuation,
                self.expr_context,
                &self.info,
                &self.model.variable_manager,
            );
        }
        new_valuation.into()
    }
}

pub trait UpdatableValuation {
    type ClassIdx: Index;
    type ClassEntryIdx: Index;
    type ValuationIdx: Index;

    fn apply_assignments<S: Span, E, EC: ExpressionContext<E>>(
        &mut self,
        assignments: &[Assignment<VariableReference, S, E>],
        valuation: &ValuationEntry<Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx>,
        evaluator: &mut EC,
        variable_info: &ModelVariableInfo<Self::ClassIdx, Self::ClassEntryIdx>,
        variables: &VariableManager<S, E>,
    );
}

impl<CI: Index, CEI: Index, VI: Index> UpdatableValuation for StandaloneValuation<'_, CI, CEI, VI> {
    type ClassIdx = CI;
    type ClassEntryIdx = CEI;
    type ValuationIdx = VI;

    fn apply_assignments<S: Span, E, EC: ExpressionContext<E>>(
        &mut self,
        assignments: &[Assignment<VariableReference, S, E>],
        valuation: &ValuationEntry<Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx>,
        evaluator: &mut EC,
        variable_info: &ModelVariableInfo<Self::ClassIdx, Self::ClassEntryIdx>,
        variables: &VariableManager<S, E>,
    ) {
        let val_source = variable_info.get_valuation_source(valuation);
        for assignment in assignments {
            let target = variables.get(&assignment.target).unwrap();
            let target_index = variable_info
                .valuation_map
                .map_to_variable(assignment.target.index)
                .expect("Cannot assign to constant");
            match target.range {
                VariableRange::BoundedInt { .. } => {
                    let value = evaluator.evaluate_int(&assignment.value, &val_source);
                    let (min, max) = variable_info.details[target_index].bounds.unwrap();
                    if value < min || value > max {
                        panic!(
                            "Value for {} exceeds variable bounds, bounds are ({}, {}), value is {}",
                            variables.variables[assignment.target.index].name, min, max, value
                        );
                    } else {
                        self.set_int(target_index, value);
                    }
                }
                VariableRange::UnboundedInt { .. } => {
                    let value = evaluator.evaluate_int(&assignment.value, &val_source);
                    self.set_int(target_index, value);
                }
                VariableRange::Boolean { .. } => {
                    let value = evaluator.evaluate_bool(&assignment.value, &val_source);
                    self.set_bool(target_index, value);
                }
                VariableRange::Float { .. } => {
                    let value = evaluator.evaluate_float(&assignment.value, &val_source);
                    self.set_double(target_index, value);
                }
            }
        }
    }
}
