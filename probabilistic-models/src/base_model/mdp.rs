use crate::choices::{ChoiceToBranch, StateToChoice};
use crate::{BranchIndex, ChoiceIndex, StateIndex};
use typed_index_collections::{
    ChainedCsrIter, Csr, CsrIterator, Index, IndexRange, RawIndex, SemiboundedIndexRange, To1,
};

#[derive(Default)]
pub struct Mdp<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> {
    pub state_to_choice: StateToChoice<StateIdx, ChoiceIdx>,
    pub choice_to_branch: ChoiceToBranch<ChoiceIdx, BranchIdx>,
    pub branch_probabilities: To1<BranchIdx, f64>,
    pub branch_destinations: To1<BranchIdx, StateIdx>,
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> super::ReadStateSpace
    for Mdp<StateIdx, ChoiceIdx, BranchIdx>
{
    type StateIdx = StateIdx;
    type ChoiceIdx = ChoiceIdx;
    type BranchIdx = BranchIdx;

    fn states(&self) -> SemiboundedIndexRange<Self::StateIdx> {
        self.state_to_choice.keys()
    }

    fn choices(&self) -> SemiboundedIndexRange<Self::ChoiceIdx> {
        self.choice_to_branch.keys()
    }

    fn branches(&self) -> SemiboundedIndexRange<Self::BranchIdx> {
        self.choice_to_branch.values()
    }

    fn choices_of_state(&self, state: Self::StateIdx) -> IndexRange<Self::ChoiceIdx> {
        self.state_to_choice.index(state)
    }

    fn branches_of_choice(&self, choice: Self::ChoiceIdx) -> IndexRange<Self::BranchIdx> {
        self.choice_to_branch.index(choice)
    }

    fn branch_probability(&self, branch: Self::BranchIdx) -> f64 {
        self.branch_probabilities[branch]
    }

    fn branch_destination(&self, branch: Self::BranchIdx) -> Self::StateIdx {
        self.branch_destinations[branch]
    }
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> super::BaseModel
    for Mdp<StateIdx, ChoiceIdx, BranchIdx>
{
}

impl Mdp<StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> {
    pub fn with_default_types() -> Self {
        Self::default()
    }
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> Mdp<StateIdx, ChoiceIdx, BranchIdx> {
    pub fn add_state(&mut self, state_index: StateIdx) {
        let next_choice = self.state_to_choice.end();
        self.state_to_choice
            .add_entry(state_index, next_choice, next_choice);
    }
    pub fn add_choice(&mut self) -> ChoiceIdx {
        let choice_index = self.choice_to_branch.keys().end();
        let branch_index = self.choice_to_branch.end();
        self.choice_to_branch
            .add_entry(choice_index, branch_index, branch_index);
        self.state_to_choice
            .extend_last_entry(choice_index + ChoiceIdx::RawType::one());
        choice_index
    }

    pub fn add_branch(&mut self, probability: f64, target: StateIdx) -> BranchIdx {
        let branch_index = self.branch_destinations.add(target);
        self.branch_probabilities.add(probability);
        self.choice_to_branch
            .extend_last_entry(branch_index + BranchIdx::RawType::one());
        branch_index
    }

    pub fn add_choice_from_slice(&mut self, branches: &[(f64, StateIdx)]) -> ChoiceIdx {
        let index = self.add_choice();
        for &(rate_or_probability, target) in branches {
            self.add_branch(rate_or_probability, target);
        }
        index
    }

    pub fn state_choice_pairs(&self) -> StateChoicePairs<'_, StateIdx, ChoiceIdx> {
        StateChoicePairs {
            state_to_choice: &self.state_to_choice,
        }
    }

    pub fn state_choice_branch_triples(
        &self,
    ) -> StateChoiceBranchTriples<'_, StateIdx, ChoiceIdx, BranchIdx> {
        StateChoiceBranchTriples {
            state_to_choice: &self.state_to_choice,
            choice_to_branch: &self.choice_to_branch,
        }
    }
}

pub struct StateChoicePairs<'a, StateIdx: Index, ChoiceIdx: Index> {
    state_to_choice: &'a Csr<StateIdx, ChoiceIdx>,
}
impl<'a, StateIdx: Index, ChoiceIdx: Index> IntoIterator
    for StateChoicePairs<'a, StateIdx, ChoiceIdx>
{
    type Item = (StateIdx, ChoiceIdx);
    type IntoIter = CsrIterator<'a, StateIdx, ChoiceIdx>;

    fn into_iter(self) -> Self::IntoIter {
        self.state_to_choice.into_iter()
    }
}

pub struct StateChoiceBranchTriples<'a, StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> {
    state_to_choice: &'a Csr<StateIdx, ChoiceIdx>,
    choice_to_branch: &'a Csr<ChoiceIdx, BranchIdx>,
}

impl<'a, StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> IntoIterator
    for StateChoiceBranchTriples<'a, StateIdx, ChoiceIdx, BranchIdx>
{
    type Item = (StateIdx, ChoiceIdx, BranchIdx);
    type IntoIter = StateChoiceBranchTriplesIterator<'a, StateIdx, ChoiceIdx, BranchIdx>;

    fn into_iter(self) -> Self::IntoIter {
        StateChoiceBranchTriplesIterator {
            iterator: self
                .state_to_choice
                .chain(self.choice_to_branch)
                .into_iter(),
        }
    }
}

pub struct StateChoiceBranchTriplesIterator<'a, StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index>
{
    iterator: ChainedCsrIter<
        StateIdx,
        ChoiceIdx,
        BranchIdx,
        CsrIterator<'a, StateIdx, ChoiceIdx>,
        CsrIterator<'a, ChoiceIdx, BranchIdx>,
    >,
}

impl<'a, StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> Iterator
    for StateChoiceBranchTriplesIterator<'a, StateIdx, ChoiceIdx, BranchIdx>
{
    type Item = (StateIdx, ChoiceIdx, BranchIdx);

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator
            .next()
            .map(|((state, choice), branch)| (state, choice, branch))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterator.size_hint()
    }
}

#[macro_export]
macro_rules! mdp {
    ($mdp:ident = {$($state:ident -> $($p:literal: $dest:ident)&*),* $(,)?}) => {
        #[allow(unused_mut)]
        let (mut $mdp, __mdp_state_indices) = {
            use $crate::Index;
            use $crate::StateIndex;
            use $crate::base_model::Mdp;
            use ::std::collections::HashMap;

            let mut state_indices: HashMap<&'static str, StateIndex<usize>> = HashMap::new();
            let mut order: Vec<&'static str> = Vec::new();
            $(
                state_indices.entry(stringify!($state)).or_insert_with(|| {
                    let index = StateIndex::from_raw(order.len());
                    order.push(stringify!($state));
                    index
                });
            )*

            let mut choices_by_state: HashMap<&'static str, Vec<Vec<(f64, StateIndex<usize>)>>> =
                HashMap::new();
            $(
                choices_by_state
                    .entry(stringify!($state))
                    .or_insert_with(Vec::new)
                    .push(vec![$( ($p, state_indices[stringify!($dest)]) ),*]);
            )*

            let mut built = Mdp::with_default_types();
            for name in &order {
                built.add_state(state_indices[name]);
                if let Some(choices) = choices_by_state.get(name) {
                    for choice in choices {
                        built.add_choice_from_slice(choice);
                    }
                }
            }
            (built, state_indices)
        };
        $(
            #[allow(unused_variables)]
            let $state = __mdp_state_indices[stringify!($state)];
        )*
    };
}

fn test() {
    mdp!(model = {
        s1 -> 0.4: s1 & 0.6: s2,
        s2 -> 0.4: s1 & 0.6: s2,
        a -> 0.4: s1 & 0.6: s2,
        b -> 0.4: s1 & 0.6: s2,
        c -> 0.4: s1 & 0.6: s2,
        d -> 0.4: s1 & 0.6: s2
    });
}
