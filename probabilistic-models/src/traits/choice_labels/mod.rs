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
