use crate::Model;
use crate::base_model::BaseModel;
use crate::valuations::Valuations;
use typed_index_collections::Index;

impl<M: BaseModel, Ini, ChLabel, BrLabel, O, APs, R, A, P>
    Model<M, Ini, ChLabel, BrLabel, O, APs, R, A, (), P>
{
    pub fn with_valuations<ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index>(
        self,
        valuations: Valuations<M::StateIndex, ClassIdx, ClassEntryIdx, ValuationIdx>,
    ) -> Model<
        M,
        Ini,
        ChLabel,
        BrLabel,
        O,
        APs,
        R,
        A,
        Valuations<M::StateIndex, ClassIdx, ClassEntryIdx, ValuationIdx>,
        P,
    > {
        Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: valuations,
            predecessors: self.predecessors,
        }
    }
}

impl<M, Ini, ChLabel, BrLabel, O, APs, R, A, V, P>
    Model<M, Ini, ChLabel, BrLabel, O, APs, R, A, V, P>
{
    pub fn without_valuations(self) -> Model<M, Ini, ChLabel, BrLabel, O, APs, R, A, (), P> {
        Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: (),
            predecessors: self.predecessors,
        }
    }
}
