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
