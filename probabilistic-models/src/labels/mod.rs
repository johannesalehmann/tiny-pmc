mod branch_labels;
mod choice_labels;

use typed_index_collections::{Index, To1};

pub trait ReadLabels {
    type EntityIdx: Index;
    type ActionIdx: Index;
    type E;

    fn label(&self, entity: Self::EntityIdx) -> &Self::E;
    fn action_index(&self, entity: Self::EntityIdx) -> Self::ActionIdx;
    fn label_of_action(&self, action: Self::ActionIdx) -> &Self::E;
}

pub struct Labels<EntityIdx: Index, ActionIdx: Index, E> {
    entity_to_action: To1<EntityIdx, ActionIdx>,
    action_to_label: To1<ActionIdx, E>,
}

impl<EntityIdx: Index, ActionIdx: Index, E> Labels<EntityIdx, ActionIdx, E> {
    pub fn new() -> Self {
        Self {
            entity_to_action: To1::new(),
            action_to_label: To1::new(),
        }
    }

    pub fn add_action(&mut self, label: E) -> ActionIdx {
        self.action_to_label.add(label)
    }

    pub fn label_entity(&mut self, entity: EntityIdx, action: ActionIdx) {
        self.entity_to_action.add_checked(entity, action);
    }
}

impl<EntityIdx: Index, ActionIdx: Index, E> ReadLabels for Labels<EntityIdx, ActionIdx, E> {
    type EntityIdx = EntityIdx;
    type ActionIdx = ActionIdx;
    type E = E;

    fn label(&self, entity: Self::EntityIdx) -> &Self::E {
        &self.action_to_label[self.entity_to_action[entity]]
    }

    fn action_index(&self, entity: Self::EntityIdx) -> Self::ActionIdx {
        self.entity_to_action[entity]
    }

    fn label_of_action(&self, action: Self::ActionIdx) -> &Self::E {
        &self.action_to_label[action]
    }
}
