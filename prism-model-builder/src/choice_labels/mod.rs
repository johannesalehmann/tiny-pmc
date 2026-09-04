mod action_names;
pub use action_names::ActionNameChoiceLabels;

mod command_indices;
pub use command_indices::{CommandIndexComponent, CommandIndexLabels, CommandIndices};

mod no_choice_labels;
pub use no_choice_labels::NoChoiceLabels;

pub trait ChoiceLabelBuilder {
    type ChoiceIdx;
    type NameIndex;
    type ContextType: Context;
    type ChoiceLabels;
    fn name_to_index(&mut self, name: Option<&str>) -> Self::NameIndex;
    fn label_choice(
        &mut self,
        choice_index: Self::ChoiceIdx,
        name: &Self::NameIndex,
        context: &Self::ContextType,
    );
    fn into_choice_labels(self) -> Self::ChoiceLabels;
}

pub trait Context {
    fn new_unsynchronised(module_index: usize, command_index: usize) -> Self;
    fn new_synchronised(component_count: usize) -> Self;
    fn new_deadlock_fix() -> Self;
    fn set_synchronised_component(
        &mut self,
        component_index: usize,
        module_index: usize,
        command_index: usize,
    );
}

impl Context for () {
    fn new_unsynchronised(_module_index: usize, _command_index: usize) -> Self {
        ()
    }

    fn new_synchronised(_component_count: usize) -> Self {
        ()
    }
    fn new_deadlock_fix() -> Self {
        ()
    }

    fn set_synchronised_component(
        &mut self,
        _component_index: usize,
        _module_index: usize,
        _command_index: usize,
    ) {
    }
}
