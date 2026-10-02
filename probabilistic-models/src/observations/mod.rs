use crate::base_model::BaseModel;
use crate::{Model, OptionalComponent};
use std::convert::Infallible;

impl crate::Component for Infallible {} // TODO: This is temporary -- remove once Observations and Annotations have been properly implemented

impl<M: BaseModel, I, CL, BL, Obs: OptionalComponent, APs, Rew, Ann, Val, Pred>
    Model<M, I, CL, BL, Obs, APs, Rew, Ann, Val, Pred>
{
    pub fn without_observations(self) -> Model<M, I, CL, BL, (), APs, Rew, Ann, Val, Pred> {
        Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: (),
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        }
    }
}
