use super::{PermuteStates, PermuteStatesWithContext, StateOrdering};
use crate::base_model::{BaseModel, SinglePlayer, TurnBased};
use crate::traits::ReadStateSpace;
use crate::{Ix, StateIndex};
use typed_index_collections::To1;

impl<I: Ix> PermuteStates for TurnBased<I> {
    fn permute_states(&self, ordering: &StateOrdering<StateIndex<I>>) -> Self {
        let (n, ordering_n) = (self.state_to_owner.len(), ordering.len());
        assert_eq!(
            n, ordering_n,
            "The owners cover {n} states, but the state ordering covers {ordering_n} states."
        );
        let new_owners: Vec<_> = (&ordering.new_to_old)
            .into_iter()
            .map(|&old_state| self.state_to_owner[old_state])
            .collect();
        TurnBased {
            state_to_owner: To1::with_entries(new_owners),
            players: self.players.clone(),
        }
    }
}

// `SinglePlayer` has no index type, so it cannot implement `PermuteStates` (which requires `HasIx`).
impl<I: Ix> PermuteStatesWithContext<I> for SinglePlayer {
    fn permute_states_with_context<Base: ReadStateSpace<Ix = I>>(
        &self,
        _ordering: &StateOrdering<StateIndex<I>>,
        _old_base: &Base,
        _new_base: &Base,
    ) -> Self {
        SinglePlayer
    }
}

impl<G: PermuteStates + ReadStateSpace, P: PermuteStatesWithContext<G::Ix>> PermuteStates
    for BaseModel<G, P>
{
    fn permute_states(&self, ordering: &StateOrdering<StateIndex<G::Ix>>) -> Self {
        let graph = self.graph.permute_states(ordering);
        let players = self
            .players
            .permute_states_with_context(ordering, &self.graph, &graph);
        BaseModel { graph, players }
    }
}
