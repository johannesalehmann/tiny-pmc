use crate::mecs::Mecs;
use crate::sccs::Scc;
use crate::sub_model::{RewardsSource, StateOrdering};
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::marker::PhantomData;
use typed_index_collections::{Index, IndexRange, To1};

// The attractor algorithm computes which fraction of probability mass is known for each *choice*.
// This enum specifies how to use the choice values to state values.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AttractorChoiceMode {
    BestChoice,
    WorstChoice,
    AverageChoice,
}

pub struct Attractor {
    choice_mode: AttractorChoiceMode,
}

impl Attractor {
    pub fn new(choice_mode: AttractorChoiceMode) -> Self {
        Self { choice_mode }
    }
}

pub struct AttractorContext<SI: Index, CI: Index> {
    queue: BinaryHeap<(Priority, SI)>,
    known_mass: To1<CI, f64>,
    non_self_loop_mass: To1<CI, f64>,
    phantom_data: PhantomData<(SI, CI)>,
}

impl<SI: Index, CI: Index> AttractorContext<SI, CI> {
    fn state_priority(&self, choices: IndexRange<CI>, mode: AttractorChoiceMode) -> f64 {
        let mut value = 0.0;
        let mut count = 0;

        if mode == AttractorChoiceMode::WorstChoice {
            value = f64::MAX;
        }

        for choice in choices {
            if self.non_self_loop_mass[choice] == 0.0 {
                continue;
            }
            count += 1;
            let new_value = self.known_mass[choice] / self.non_self_loop_mass[choice];
            match mode {
                AttractorChoiceMode::BestChoice => {
                    if new_value > value {
                        value = new_value;
                    }
                }
                AttractorChoiceMode::WorstChoice => {
                    if new_value < value {
                        value = new_value;
                    }
                }
                AttractorChoiceMode::AverageChoice => value += new_value,
            }
        }
        if mode == AttractorChoiceMode::AverageChoice && count > 0 {
            value = value / count as f64;
        }

        value
    }
}

impl StateOrdering for Attractor {
    type Context<SI: Index, CI: Index> = AttractorContext<SI, CI>;

    fn create_context<M: ReadStateSpace>(
        &self,
        model: &M,
    ) -> Self::Context<M::StateIndex, M::ChoiceIndex> {
        todo!()
    }

    fn compute_ordering<
        M: ReadStateSpace<StateIndex = SI, ChoiceIndex = CI, BranchIndex = BI>
            + ReadPredecessors<StateIdx = SI, ChoiceIdx = CI, BranchIdx = BI>,
        SI: Index,
        CI: Index,
        BI: Index,
        ScI: Index,
        ScEI: Index,
        NewSI: Index,
        Rew: RewardsSource<SI, CI>,
    >(
        &self,
        model: &M,
        scc: Scc<'_, ScI, ScEI, SI>,
        mecs: &Mecs<SI, CI>,
        values: &To1<SI, f64>,
        rewards: &Rew,
        context: &mut Self::Context<SI, CI>,
        to_old_state_index: &mut To1<NewSI, SI>,
    ) {
        for state in scc.states() {
            for choice in model.choices_of_state(state) {
                let mut known = 0.0;
                let mut to_self = 0.0;
                for branch in model.branches_of_choice(choice) {
                    let dest = model.branch_destination(branch);
                    if dest == state
                        || mecs
                            .representative(dest)
                            .map(|r| Some(r) == mecs.representative(state))
                            .unwrap_or(false)
                    {
                        to_self += model.branch_probability(branch);
                    } else if !scc.contains(dest) {
                        known += model.branch_probability(branch);
                    }
                }
                context.known_mass[choice] = known;
                context.non_self_loop_mass[choice] = 1.0 - to_self;
            }
            context.queue.push((
                Priority::new(
                    context.state_priority(model.choices_of_state(state), self.choice_mode),
                ),
                state,
            ));
        }
        todo!()
    }
}

struct Priority {
    value: f64,
}

impl Priority {
    pub fn new(value: f64) -> Self {
        assert!(value.is_finite());
        Self { value }
    }
}

impl PartialEq for Priority {
    fn eq(&self, other: &Self) -> bool {
        self.value.total_cmp(&other.value).is_eq()
    }
}

impl Eq for Priority {}
impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.value.total_cmp(&other.value))
    }
}
impl Ord for Priority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.total_cmp(&other.value)
    }
}
