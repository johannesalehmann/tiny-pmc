mod context;
mod orderings;
mod sub_model_rewards;

use crate::mecs::Mecs;
use crate::sccs::Scc;
pub use context::SubModelConstructionContext;
pub use orderings::{
    Attractor, AttractorChoiceMode, IndexBased, IndexOrderDirection, Legacy, StateOrdering,
    SubModelOrder,
};
use probabilistic_models::base_model::Mdp;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
pub use sub_model_rewards::{RewardsSource, StateAndChoiceRewards, UnitStateRewards};
use typed_index_collections::{Index, RawIndex, To1};

pub struct SubModel<StateIdx: Index, NewSI: Index, NewCI: Index, NewBI: Index> {
    pub mdp: Mdp<NewSI, NewCI, NewBI>,
    /// The value that external states (i.e. those outside the SCC) contribute to a choice.
    pub choice_exit_values: To1<NewCI, f64>,
    pub to_old_state_index: To1<NewSI, StateIdx>,
}

impl<StateIdx: Index, NewSI: Index, NewCI: Index, NewBI: Index>
    SubModel<StateIdx, NewSI, NewCI, NewBI>
{
    pub fn empty() -> Self {
        Self {
            mdp: Mdp::default(),
            choice_exit_values: To1::new(),
            to_old_state_index: To1::new(),
        }
    }

    pub fn from_scc<
        M: ReadStateSpace<StateIndex = StateIdx>
            + ReadPredecessors<
                StateIdx = StateIdx,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
        ScI: Index,
        ScEI: Index,
        Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
        O: StateOrdering,
    >(
        model: &M,
        scc: Scc<'_, ScI, ScEI, M::StateIndex>,
        mecs: &Mecs<M::StateIndex, M::ChoiceIndex>,
        values: &To1<M::StateIndex, f64>,
        rewards: &Rew,
        ordering: &O,
    ) -> Self {
        let mut context = SubModelConstructionContext::new(model, ordering);
        let mut sub_model = SubModel::empty();
        sub_model.rebuild_from_scc(model, scc, mecs, values, rewards, ordering, &mut context);
        sub_model
    }

    pub fn rebuild_from_scc<
        M: ReadStateSpace<StateIndex = StateIdx>
            + ReadPredecessors<
                StateIdx = StateIdx,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
        ScI: Index,
        ScEI: Index,
        Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
        O: StateOrdering,
    >(
        &mut self,
        model: &M,
        scc: Scc<'_, ScI, ScEI, M::StateIndex>,
        mecs: &Mecs<M::StateIndex, M::ChoiceIndex>,
        values: &To1<M::StateIndex, f64>,
        rewards: &Rew,
        ordering: &O,
        context: &mut SubModelConstructionContext<
            M::StateIndex,
            O::Context<M::StateIndex, M::ChoiceIndex>,
        >,
    ) {
        self.to_old_state_index.clear();
        ordering.compute_ordering(
            model,
            scc,
            mecs,
            values,
            rewards,
            &mut context.ordering,
            &mut self.to_old_state_index,
        );
        for (new_state, &state) in self.to_old_state_index.enumerate() {
            context.to_new_state_index[state] = Some(new_state.raw().as_usize());
        }

        self.mdp.clear();
        self.choice_exit_values.clear();

        for (new_state, &state) in self.to_old_state_index.enumerate() {
            self.mdp.add_state(new_state);
            for member in mecs.states_merged_into(state) {
                let state_reward = if rewards.has_state_rewards() {
                    rewards.state_reward(member)
                } else {
                    0.0
                };
                for choice in model.choices_of_state(member) {
                    if mecs.is_internal_choice(choice) {
                        continue;
                    }
                    let choice_index = self.mdp.add_choice();
                    let mut to_self = 0.0;
                    let mut exit_value = 0.0;
                    for branch in model.branches_of_choice(choice) {
                        let mut destination = model.branch_destination(branch);
                        if let Some(representative) = mecs.representative(destination) {
                            destination = representative;
                        }
                        let p = model.branch_probability(branch);

                        if destination == state {
                            to_self += p;
                        } else if let Some(target) = context.to_new_state_index[destination] {
                            // We first add the actual probability. After the loop, we then scale
                            // the probability to account for removed self loops.
                            let target = NewSI::from_raw(NewSI::RawType::from_usize(target));
                            self.mdp.add_branch(p, target);
                        } else {
                            exit_value += p * values[destination];
                        }
                    }

                    let reward = if rewards.has_choice_rewards() {
                        state_reward + rewards.choice_reward(choice)
                    } else {
                        state_reward
                    };

                    // Preserve actions that are just a self-loop (relevant at least for minimum
                    //  reachability probability).
                    if to_self == 1.0 {
                        self.mdp.add_branch(1.0, new_state);
                        assert_eq!(exit_value, 0.0);
                        self.choice_exit_values.add_checked(choice_index, reward);
                    } else {
                        let scale_factor = 1.0 / (1.0 - to_self);
                        for branch in self.mdp.branches_of_choice(choice_index) {
                            self.mdp.branch_probabilities[branch] *= scale_factor;
                        }
                        self.choice_exit_values
                            .add_checked(choice_index, (exit_value + reward) * scale_factor);
                    }
                }
            }
        }

        context.reset(&self.to_old_state_index);
    }
}

#[cfg(test)]
mod tests {
    use super::{IndexBased, IndexOrderDirection, SubModel, SubModelConstructionContext};
    use crate::mecs::Mecs;
    use crate::sccs::{ExcludeStatesAndChoices, SccEntryIndex, SccIndex, Sccs};
    use crate::value_iteration::precomputed_states::S0S1;
    use probabilistic_models::mdp;
    use probabilistic_models::{BranchIndex, ChoiceIndex, Model, PredecessorIndex, StateIndex};
    use typed_index_collections::{Csr, Index, To1};

    #[test]
    fn loop_removal_and_rescaling() {
        // States 0 and 1 form an SCC, state 2 is the goal state.
        mdp!(mdp = {
            s0 -> 0.25: s0 & 0.25: s1 & 0.5: s2,
            s1 -> 1.0: s0,
            s2 -> 1.0: s2
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();

        let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> = Sccs::compute(
            &model,
            &S0S1::new(
                To1::with_entries(vec![false, false, false]),
                To1::with_entries(vec![false, false, true]),
            ),
        );
        let values = To1::with_entries(vec![0.0, 0.0, 1.0]);
        let ordering = IndexBased::new(IndexOrderDirection::BackToFront);
        let mut context = SubModelConstructionContext::new(&model, &ordering);

        let mut sub_model: SubModel<_, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> =
            SubModel::empty();
        sub_model.rebuild_from_scc(
            &model,
            sccs.scc_of_state(StateIndex::from_raw(0)).unwrap(),
            &Mecs::empty(),
            &values,
            &(),
            &ordering,
            &mut context,
        );

        assert_eq!(
            sub_model.to_old_state_index,
            To1::with_entries(vec![StateIndex::from_raw(1), StateIndex::from_raw(0)])
        );
        assert_eq!(
            sub_model.mdp.state_to_choice,
            Csr::with_entries(vec![ChoiceIndex::from_raw(1), ChoiceIndex::from_raw(2)])
        );
        assert_eq!(
            sub_model.mdp.choice_to_branch,
            Csr::with_entries(vec![BranchIndex::from_raw(1), BranchIndex::from_raw(2)])
        );
        assert_eq!(
            sub_model.mdp.branch_destinations,
            To1::with_entries(vec![StateIndex::from_raw(1), StateIndex::from_raw(0)])
        );
        assert_eq!(
            sub_model.mdp.branch_probabilities,
            To1::with_entries(vec![1.0, 0.25 * (1.0 / 0.75)])
        );
        assert_eq!(
            sub_model.choice_exit_values,
            To1::with_entries(vec![0.0, 0.5 * 1.0 * (1.0 / 0.75)])
        );
    }

    #[test]
    fn external_branch_values() {
        // State 0 forms an SCC of its own and leaves it into state 1 and into the goal state 2.
        mdp!(mdp = {
            s0 -> 0.5: s0 & 0.25: s1 & 0.25: s2,
            s1 -> 1.0: s1,
            s2 -> 1.0: s2
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();

        let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> = Sccs::compute(
            &model,
            &S0S1::new(
                To1::with_entries(vec![false, false, false]),
                To1::with_entries(vec![false, false, true]),
            ),
        );
        let values = To1::with_entries(vec![0.0, 0.6, 1.0]);
        let ordering = IndexBased::new(IndexOrderDirection::BackToFront);
        let mut context = SubModelConstructionContext::new(&model, &ordering);

        let mut sub_model: SubModel<_, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> =
            SubModel::empty();
        sub_model.rebuild_from_scc(
            &model,
            sccs.scc_of_state(StateIndex::from_raw(0)).unwrap(),
            &Mecs::empty(),
            &values,
            &(),
            &ordering,
            &mut context,
        );

        assert_eq!(
            sub_model.to_old_state_index,
            To1::with_entries(vec![StateIndex::from_raw(0)])
        );
        assert_eq!(
            sub_model.mdp.state_to_choice,
            Csr::with_entries(vec![ChoiceIndex::from_raw(1)])
        );
        // Both remaining branches leave the sub-model, so the only choice has no branches left.
        assert_eq!(
            sub_model.mdp.choice_to_branch,
            Csr::with_entries(vec![BranchIndex::from_raw(0)])
        );
        assert_eq!(
            sub_model.choice_exit_values,
            To1::with_entries(vec![(0.25 * 0.6 + 0.25 * 1.0) * (1.0 / 0.5)])
        );
    }

    #[test]
    fn full_self_loop_preserved() {
        mdp!(mdp = {
            s0 -> 1.0: s0,
            s0 -> 0.25: s0 & 0.25: s1 & 0.5: s2,
            s1 -> 1.0: s0,
            s2 -> 1.0: s2
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();

        let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> = Sccs::compute(
            &model,
            &S0S1::new(
                To1::with_entries(vec![false, false, false]),
                To1::with_entries(vec![false, false, true]),
            ),
        );
        let values = To1::with_entries(vec![0.0, 0.0, 1.0]);
        let ordering = IndexBased::new(IndexOrderDirection::BackToFront);
        let mut context = SubModelConstructionContext::new(&model, &ordering);

        let mut sub_model: SubModel<_, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> =
            SubModel::empty();
        sub_model.rebuild_from_scc(
            &model,
            sccs.scc_of_state(StateIndex::from_raw(0)).unwrap(),
            &Mecs::empty(),
            &values,
            &(),
            &ordering,
            &mut context,
        );

        assert_eq!(
            sub_model.to_old_state_index,
            To1::with_entries(vec![StateIndex::from_raw(1), StateIndex::from_raw(0)])
        );
        assert_eq!(
            sub_model.mdp.state_to_choice,
            Csr::with_entries(vec![ChoiceIndex::from_raw(1), ChoiceIndex::from_raw(3)])
        );
        // The p=1 self loop is not removed by the self-loop removal (otherwise, it would produce
        // incorrect probabilities for minimal reachability).
        assert_eq!(
            sub_model.mdp.choice_to_branch,
            Csr::with_entries(vec![
                BranchIndex::from_raw(1),
                BranchIndex::from_raw(2),
                BranchIndex::from_raw(3)
            ])
        );
        assert_eq!(
            sub_model.mdp.branch_destinations,
            To1::with_entries(vec![
                StateIndex::from_raw(1),
                StateIndex::from_raw(1),
                StateIndex::from_raw(0)
            ])
        );
        assert_eq!(
            sub_model.mdp.branch_probabilities,
            To1::with_entries(vec![1.0, 1.0, 0.25 * (1.0 / 0.75)])
        );
        assert_eq!(
            sub_model.choice_exit_values,
            To1::with_entries(vec![0.0, 0.0, 0.5 * 1.0 * (1.0 / 0.75)])
        );
    }

    #[test]
    fn context_reuse() {
        // State 0 leaves its own SCC into the SCC of state 1, which is built first. State 1
        // leaves its SCC into the goal state 2.
        mdp!(mdp = {
            s0 -> 0.5: s0 & 0.5: s1,
            s1 -> 0.5: s1 & 0.5: s2,
            s2 -> 1.0: s2
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();

        let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> = Sccs::compute(
            &model,
            &S0S1::new(
                To1::with_entries(vec![false, false, false]),
                To1::with_entries(vec![false, false, true]),
            ),
        );
        let values = To1::with_entries(vec![0.0, 0.6, 1.0]);
        let ordering = IndexBased::new(IndexOrderDirection::BackToFront);
        let mut context = SubModelConstructionContext::new(&model, &ordering);

        let mut sub_models: Vec<
            SubModel<StateIndex<usize>, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>>,
        > = Vec::new();
        for scc in sccs.reverse_topological_ordering() {
            let mut sub_model: SubModel<
                _,
                StateIndex<usize>,
                ChoiceIndex<usize>,
                BranchIndex<usize>,
            > = SubModel::empty();
            sub_model.rebuild_from_scc(
                &model,
                scc,
                &Mecs::empty(),
                &values,
                &(),
                &ordering,
                &mut context,
            );
            sub_models.push(sub_model);
        }

        assert_eq!(
            sub_models[0].to_old_state_index,
            To1::with_entries(vec![StateIndex::from_raw(1)])
        );
        assert_eq!(
            sub_models[1].to_old_state_index,
            To1::with_entries(vec![StateIndex::from_raw(0)])
        );
        // State 1 must not be left in the context after building the first sub-model, as its
        // branch would otherwise be mistaken for a branch within the second sub-model.
        assert_eq!(
            sub_models[1].mdp.choice_to_branch,
            Csr::with_entries(vec![BranchIndex::from_raw(0)])
        );
        assert_eq!(
            sub_models[1].choice_exit_values,
            To1::with_entries(vec![0.5 * 0.6 * (1.0 / 0.5)])
        );
    }

    #[test]
    fn collapsed_mec() {
        // States 0 and 1 form a MEC whose only exit is the last choice of state 1.
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s0,
            s1 -> 0.5: s0 & 0.5: s2,
            s2 -> 1.0: s2
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();

        let precomputed_states = S0S1::new(
            To1::with_entries(vec![false, false, false]),
            To1::with_entries(vec![false, false, true]),
        );
        let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> =
            Sccs::compute(&model, &precomputed_states);
        let mecs = Mecs::compute(
            &model,
            ExcludeStatesAndChoices::new(
                To1::with_entries(vec![false, false, true]),
                To1::with_entries(vec![false; 4]),
            ),
        );
        let representative = mecs.representative(StateIndex::from_raw(0)).unwrap();
        let values = To1::with_entries(vec![0.0, 0.0, 1.0]);
        let ordering = IndexBased::new(IndexOrderDirection::BackToFront);
        let mut context = SubModelConstructionContext::new(&model, &ordering);

        let mut sub_model: SubModel<_, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> =
            SubModel::empty();
        sub_model.rebuild_from_scc(
            &model,
            sccs.scc_of_state(StateIndex::from_raw(0)).unwrap(),
            &mecs,
            &values,
            &(),
            &ordering,
            &mut context,
        );

        // The internal choices are dropped and the exit choice is moved to the representative.
        // Its branch back into the MEC becomes a self-loop, which is removed by rescaling.
        assert_eq!(
            sub_model.to_old_state_index,
            To1::with_entries(vec![representative])
        );
        assert_eq!(
            sub_model.mdp.state_to_choice,
            Csr::with_entries(vec![ChoiceIndex::from_raw(1)])
        );
        assert_eq!(
            sub_model.mdp.choice_to_branch,
            Csr::with_entries(vec![BranchIndex::from_raw(0)])
        );
        assert_eq!(sub_model.choice_exit_values, To1::with_entries(vec![1.0]));
    }

    // TODO: Test submodels with rewards!
}
