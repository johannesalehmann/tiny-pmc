use crate::labels::ReadLabels;
use crate::{ActionIndex, BranchIndex, HasIx, Model};

pub trait ReadBranchLabels: HasIx {
    type E;

    fn branch_label(&self, entity: BranchIndex<Self::Ix>) -> &Self::E;
    fn branch_action_index(&self, entity: BranchIndex<Self::Ix>) -> ActionIndex<Self::Ix>;
    fn label_of_branch_action(&self, action: ActionIndex<Self::Ix>) -> &Self::E;
}

#[allow(unused)]
macro_rules! derive_read_branch_labels {
    ($subcomponent:ident) => {
        fn branch_label(&self, entity: $crate::BranchIndex<Self::Ix>) -> &Self::E {
            self.$subcomponent.branch_label(entity)
        }

        fn branch_action_index(
            &self,
            entity: $crate::BranchIndex<Self::Ix>,
        ) -> $crate::ActionIndex<Self::Ix> {
            self.$subcomponent.branch_action_index(entity)
        }

        fn label_of_branch_action(&self, action: $crate::ActionIndex<Self::Ix>) -> &Self::E {
            self.$subcomponent.label_of_branch_action(action)
        }
    };
}
pub(crate) use derive_read_branch_labels;

impl<
    M: HasIx,
    Ini,
    ChLabel,
    BrLabel: ReadLabels<EntityIdx = BranchIndex<M::Ix>, ActionIdx = ActionIndex<M::Ix>>,
    Obs,
    APs,
    Rew,
    Ann,
    StateVals,
    Preds,
> ReadBranchLabels for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type E = BrLabel::E;

    fn branch_label(&self, entity: BranchIndex<Self::Ix>) -> &Self::E {
        self.branch_labels.label(entity)
    }

    fn branch_action_index(&self, entity: BranchIndex<Self::Ix>) -> ActionIndex<Self::Ix> {
        self.branch_labels.action_index(entity)
    }

    fn label_of_branch_action(&self, action: ActionIndex<Self::Ix>) -> &Self::E {
        self.branch_labels.label_of_action(action)
    }
}

impl<T: ReadBranchLabels> ReadBranchLabels for &T {
    type E = T::E;

    fn branch_label(&self, entity: BranchIndex<Self::Ix>) -> &Self::E {
        (**self).branch_label(entity)
    }
    fn branch_action_index(&self, entity: BranchIndex<Self::Ix>) -> ActionIndex<Self::Ix> {
        (**self).branch_action_index(entity)
    }
    fn label_of_branch_action(&self, action: ActionIndex<Self::Ix>) -> &Self::E {
        (**self).label_of_branch_action(action)
    }
}

impl<T: ReadBranchLabels> ReadBranchLabels for &mut T {
    type E = T::E;

    fn branch_label(&self, entity: BranchIndex<Self::Ix>) -> &Self::E {
        (**self).branch_label(entity)
    }
    fn branch_action_index(&self, entity: BranchIndex<Self::Ix>) -> ActionIndex<Self::Ix> {
        (**self).branch_action_index(entity)
    }
    fn label_of_branch_action(&self, action: ActionIndex<Self::Ix>) -> &Self::E {
        (**self).label_of_branch_action(action)
    }
}

pub trait ReadBranchLabelsMaybe: HasIx {
    type E;
    type WithBranchLabels: ReadBranchLabels<Ix = Self::Ix, E = Self::E>;

    fn has_branch_labels(&self) -> bool;
    fn try_with_branch_labels(self) -> Option<Self::WithBranchLabels>;
}

impl<T: ReadBranchLabels> ReadBranchLabelsMaybe for T {
    type E = T::E;
    type WithBranchLabels = T;

    fn has_branch_labels(&self) -> bool {
        true
    }

    fn try_with_branch_labels(self) -> Option<Self::WithBranchLabels> {
        Some(self)
    }
}

impl<T: ReadBranchLabels> ReadBranchLabelsMaybe for Option<T> {
    type E = T::E;
    type WithBranchLabels = T;

    fn has_branch_labels(&self) -> bool {
        self.is_some()
    }

    fn try_with_branch_labels(self) -> Option<Self::WithBranchLabels> {
        self
    }
}

impl<
    M: HasIx,
    Ini,
    ChLabel,
    BrLabel: ReadLabels<EntityIdx = BranchIndex<M::Ix>, ActionIdx = ActionIndex<M::Ix>>,
    Obs,
    APs,
    Rew,
    Ann,
    StateVals,
    Preds,
> ReadBranchLabelsMaybe
    for Model<M, Ini, ChLabel, Option<BrLabel>, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type E = BrLabel::E;
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
