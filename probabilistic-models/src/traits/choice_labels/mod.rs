use crate::Model;
use crate::labels::ReadLabels;
use typed_index_collections::Index;

pub trait ReadChoiceLabels {
    type ChoiceIdx: Index;
    type ChoiceActionIdx: Index;
    type E;

    fn choice_label(&self, entity: Self::ChoiceIdx) -> &Self::E;
    fn choice_action_index(&self, entity: Self::ChoiceIdx) -> Self::ChoiceActionIdx;
    fn label_of_choice_action(&self, action: Self::ChoiceActionIdx) -> &Self::E;
}

#[allow(unused)]
macro_rules! derive_read_choice_labels {
    ($subcomponent:ident) => {
        fn choice_label(&self, entity: Self::ChoiceIdx) -> &Self::E {
            self.$subcomponent.choice_label(entity)
        }

        fn choice_action_index(&self, entity: Self::ChoiceIdx) -> Self::ChoiceActionIdx {
            self.$subcomponent.choice_action_index(entity)
        }

        fn label_of_choice_action(&self, action: Self::ChoiceActionIdx) -> &Self::E {
            self.$subcomponent.label_of_choice_action(action)
        }
    };
}
pub(crate) use derive_read_choice_labels;

impl<M, Ini, ChLabel: ReadLabels, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds> ReadChoiceLabels
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type ChoiceIdx = ChLabel::EntityIdx;
    type ChoiceActionIdx = ChLabel::ActionIdx;
    type E = ChLabel::E;

    fn choice_label(&self, entity: Self::ChoiceIdx) -> &Self::E {
        self.choice_labels.label(entity)
    }

    fn choice_action_index(&self, entity: Self::ChoiceIdx) -> Self::ChoiceActionIdx {
        self.choice_labels.action_index(entity)
    }

    fn label_of_choice_action(&self, action: Self::ChoiceActionIdx) -> &Self::E {
        self.choice_labels.label_of_action(action)
    }
}

impl<T: ReadChoiceLabels> ReadChoiceLabels for &T {
    type ChoiceIdx = T::ChoiceIdx;
    type ChoiceActionIdx = T::ChoiceActionIdx;
    type E = T::E;

    fn choice_label(&self, entity: Self::ChoiceIdx) -> &Self::E {
        (**self).choice_label(entity)
    }
    fn choice_action_index(&self, entity: Self::ChoiceIdx) -> Self::ChoiceActionIdx {
        (**self).choice_action_index(entity)
    }
    fn label_of_choice_action(&self, action: Self::ChoiceActionIdx) -> &Self::E {
        (**self).label_of_choice_action(action)
    }
}

impl<T: ReadChoiceLabels> ReadChoiceLabels for &mut T {
    type ChoiceIdx = T::ChoiceIdx;
    type ChoiceActionIdx = T::ChoiceActionIdx;
    type E = T::E;

    fn choice_label(&self, entity: Self::ChoiceIdx) -> &Self::E {
        (**self).choice_label(entity)
    }
    fn choice_action_index(&self, entity: Self::ChoiceIdx) -> Self::ChoiceActionIdx {
        (**self).choice_action_index(entity)
    }
    fn label_of_choice_action(&self, action: Self::ChoiceActionIdx) -> &Self::E {
        (**self).label_of_choice_action(action)
    }
}

pub trait ReadChoiceLabelsMaybe {
    type WithChoiceLabels: ReadChoiceLabels;

    fn has_choice_labels(&self) -> bool;
    fn try_with_choice_labels(self) -> Option<Self::WithChoiceLabels>;
}

impl<T: ReadChoiceLabels> ReadChoiceLabelsMaybe for T {
    type WithChoiceLabels = T;

    fn has_choice_labels(&self) -> bool {
        true
    }

    fn try_with_choice_labels(self) -> Option<Self::WithChoiceLabels> {
        Some(self)
    }
}

impl<T: ReadChoiceLabels> ReadChoiceLabelsMaybe for Option<T> {
    type WithChoiceLabels = T;

    fn has_choice_labels(&self) -> bool {
        self.is_some()
    }

    fn try_with_choice_labels(self) -> Option<Self::WithChoiceLabels> {
        self
    }
}

impl<M, Ini, ChLabel: ReadLabels, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    ReadChoiceLabelsMaybe
    for Model<M, Ini, Option<ChLabel>, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type WithChoiceLabels = Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>;

    fn has_choice_labels(&self) -> bool {
        self.choice_labels.is_some()
    }

    fn try_with_choice_labels(self) -> Option<Self::WithChoiceLabels> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels?,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        })
    }
}
