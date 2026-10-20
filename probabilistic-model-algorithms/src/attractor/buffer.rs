use probabilistic_models::base_model::{ReadOwners, TwoPlayer};
use probabilistic_models::traits::ReadStateSpace;
use probabilistic_models::{Ix, StateIndex};
use typed_index_collections::To1;

pub struct AttractorBuffer<I: Ix> {
    counts: To1<StateIndex<I>, Count>,
    pub open_list: Vec<StateIndex<I>>,
}

#[derive(Copy, Clone)]
struct Count {
    current: u32,
    default: u32,
}

impl Count {
    fn with_default(default: u32) -> Self {
        Self {
            current: 0,
            default,
        }
    }

    fn reset_reaching_player(&mut self) {
        self.current = 1;
    }
    fn reset_avoiding_player(&mut self) {
        self.current = self.default;
    }
    fn reset_sink_state(&mut self) {
        self.current = self.default + 1;
    }
}

impl<I: Ix> AttractorBuffer<I> {
    pub fn create<M: ReadStateSpace<Ix = I>>(model: &M) -> Self {
        let mut counts = To1::with_capacity(model.states().len());
        for state in model.states() {
            counts.add_checked(
                state,
                Count::with_default(model.choices_of_state(state).len() as u32),
            );
        }
        Self {
            counts,
            open_list: Vec::new(),
        }
    }

    pub fn reset_owner_counts<M: ReadStateSpace<Ix = I> + ReadOwners<OwnerType = TwoPlayer>>(
        &mut self,
        model: &M,
        reaching_player: TwoPlayer,
    ) {
        for state in model.states() {
            if model.state_owner(state) == reaching_player {
                self.reset_reaching_player(state)
            } else {
                self.reset_avoiding_player(state)
            }
        }
    }

    pub fn reset_reaching_player(&mut self, index: StateIndex<I>) {
        self.counts[index].reset_reaching_player()
    }

    pub fn reset_avoiding_player(&mut self, index: StateIndex<I>) {
        self.counts[index].reset_avoiding_player()
    }

    pub fn reset_sink_state(&mut self, index: StateIndex<I>) {
        self.counts[index].reset_sink_state()
    }

    pub fn get_value(&self, index: StateIndex<I>) -> u32 {
        self.counts[index].current
    }

    pub fn set_value(&mut self, index: StateIndex<I>, value: u32) {
        self.counts[index].current = value
    }
}
