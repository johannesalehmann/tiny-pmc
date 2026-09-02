use crate::Model;
use crate::base_model::BaseModel;
use crate::labels::Labels;
use typed_index_collections::Index;

impl<M: BaseModel, Ini, BrLabel, O, APs, R, A, V, P>
    Model<M, Ini, (), BrLabel, O, APs, R, A, V, P>
{
    pub fn with_choice_labels<ChoiceActionIdx: Index, E>(
        self,
        labels: Labels<M::ChoiceIdx, ChoiceActionIdx, E>,
    ) -> Model<M, Ini, Labels<M::ChoiceIdx, ChoiceActionIdx, E>, BrLabel, O, APs, R, A, V, P> {
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
