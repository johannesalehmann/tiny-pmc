use crate::Model;
use crate::labels::ChoiceLabels;
use crate::traits::ReadStateSpace;

impl<M: ReadStateSpace, Ini, BrLabel, O, APs, R, A, V, P>
    Model<M, Ini, (), BrLabel, O, APs, R, A, V, P>
{
    pub fn with_choice_labels<E>(
        self,
        labels: ChoiceLabels<M::Ix, E>,
    ) -> Model<M, Ini, ChoiceLabels<M::Ix, E>, BrLabel, O, APs, R, A, V, P> {
        Model {
            base: self.base,
            initial: self.initial,
            choice_labels: labels,
            branch_labels: self.branch_labels,
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
    pub fn without_choice_labels(self) -> Model<M, Ini, (), BrLabel, O, APs, R, A, V, P> {
        Model {
            base: self.base,
            initial: self.initial,
            choice_labels: (),
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        }
    }
}
