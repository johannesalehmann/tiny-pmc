use crate::Model;
use crate::labels::BranchLabels;
use crate::traits::ReadStateSpace;

impl<M: ReadStateSpace, Ini, ChLabel, O, APs, R, A, V, P>
    Model<M, Ini, ChLabel, (), O, APs, R, A, V, P>
{
    pub fn with_branch_labels<E>(
        self,
        labels: BranchLabels<M::Ix, E>,
    ) -> Model<M, Ini, ChLabel, BranchLabels<M::Ix, E>, O, APs, R, A, V, P> {
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
