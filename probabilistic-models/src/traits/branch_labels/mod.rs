use crate::Model;
use crate::labels::ReadLabels;
use typed_index_collections::Index;

pub trait ReadBranchLabels {
    type BranchIdx: Index;
    type BranchActionIdx: Index;
    type E;

    fn branch_label(&self, entity: Self::BranchIdx) -> &Self::E;
    fn branch_action_index(&self, entity: Self::BranchIdx) -> Self::BranchActionIdx;
    fn label_of_branch_action(&self, action: Self::BranchActionIdx) -> &Self::E;
}

#[allow(unused)]
macro_rules! derive_read_branch_labels {
    ($subcomponent:ident) => {
        fn branch_label(&self, entity: Self::BranchIdx) -> &Self::E {
            self.$subcomponent.branch_label(entity)
        }

        fn branch_action_index(&self, entity: Self::BranchIdx) -> Self::BranchActionIdx {
            self.$subcomponent.branch_action_index(entity)
        }

        fn label_of_branch_action(&self, action: Self::BranchActionIdx) -> &Self::E {
            self.$subcomponent.label_of_branch_action(action)
        }
    };
}
pub(crate) use derive_read_branch_labels;

impl<M, Ini, ChLabel, BrLabel: ReadLabels, Obs, APs, Rew, Ann, StateVals, Preds> ReadBranchLabels
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type BranchIdx = BrLabel::EntityIdx;
    type BranchActionIdx = BrLabel::ActionIdx;
    type E = BrLabel::E;

    fn branch_label(&self, entity: Self::BranchIdx) -> &Self::E {
        self.branch_labels.label(entity)
    }

    fn branch_action_index(&self, entity: Self::BranchIdx) -> Self::BranchActionIdx {
        self.branch_labels.action_index(entity)
    }

    fn label_of_branch_action(&self, action: Self::BranchActionIdx) -> &Self::E {
        self.branch_labels.label_of_action(action)
    }
}

impl<T: ReadBranchLabels> ReadBranchLabels for &T {
    type BranchIdx = T::BranchIdx;
    type BranchActionIdx = T::BranchActionIdx;
    type E = T::E;

    fn branch_label(&self, entity: Self::BranchIdx) -> &Self::E {
        (**self).branch_label(entity)
    }
    fn branch_action_index(&self, entity: Self::BranchIdx) -> Self::BranchActionIdx {
        (**self).branch_action_index(entity)
    }
    fn label_of_branch_action(&self, action: Self::BranchActionIdx) -> &Self::E {
        (**self).label_of_branch_action(action)
    }
}

impl<T: ReadBranchLabels> ReadBranchLabels for &mut T {
    type BranchIdx = T::BranchIdx;
    type BranchActionIdx = T::BranchActionIdx;
    type E = T::E;

    fn branch_label(&self, entity: Self::BranchIdx) -> &Self::E {
        (**self).branch_label(entity)
    }
    fn branch_action_index(&self, entity: Self::BranchIdx) -> Self::BranchActionIdx {
        (**self).branch_action_index(entity)
    }
    fn label_of_branch_action(&self, action: Self::BranchActionIdx) -> &Self::E {
        (**self).label_of_branch_action(action)
    }
}

pub trait ReadBranchLabelsMaybe {
    type WithBranchLabels: ReadBranchLabels;

    fn has_branch_labels(&self) -> bool;
    fn try_with_branch_labels(self) -> Option<Self::WithBranchLabels>;
}

impl<T: ReadBranchLabels> ReadBranchLabelsMaybe for T {
    type WithBranchLabels = T;

    fn has_branch_labels(&self) -> bool {
        true
    }

    fn try_with_branch_labels(self) -> Option<Self::WithBranchLabels> {
        Some(self)
    }
}

impl<T: ReadBranchLabels> ReadBranchLabelsMaybe for Option<T> {
    type WithBranchLabels = T;

    fn has_branch_labels(&self) -> bool {
        self.is_some()
    }

    fn try_with_branch_labels(self) -> Option<Self::WithBranchLabels> {
        self
    }
}

impl<M, Ini, ChLabel, BrLabel: ReadLabels, Obs, APs, Rew, Ann, StateVals, Preds>
    ReadBranchLabelsMaybe
    for Model<M, Ini, ChLabel, Option<BrLabel>, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type WithBranchLabels = Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>;

    fn has_branch_labels(&self) -> bool {
        self.branch_labels.is_some()
    }

    fn try_with_branch_labels(self) -> Option<Self::WithBranchLabels> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels?,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        })
    }
}
