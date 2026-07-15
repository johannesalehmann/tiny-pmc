use crate::Model;
use crate::base_model::BaseModel;
use crate::labels::Labels;
use typed_index_collections::Index;

impl<M: BaseModel, Ini, ChLabel, O, APs, R, A, V, P>
    Model<M, Ini, ChLabel, (), O, APs, R, A, V, P>
{
    pub fn with_branch_labels<BranchActionIdx: Index, E>(
        self,
        labels: Labels<M::BranchIdx, BranchActionIdx, E>,
    ) -> Model<M, Ini, ChLabel, Labels<M::BranchIdx, BranchActionIdx, E>, O, APs, R, A, V, P> {
        Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        }
    }
}

impl<M, Ini, ChLabel, BrLabel, O, APs, R, A, V, P>
    Model<M, Ini, ChLabel, BrLabel, O, APs, R, A, V, P>
{
    pub fn without_branch_labels(self) -> Model<M, Ini, ChLabel, (), O, APs, R, A, V, P> {
        Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: (),
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        }
    }
}
