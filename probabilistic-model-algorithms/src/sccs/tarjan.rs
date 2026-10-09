use super::{ExclusionCriterion, Sccs};
use probabilistic_models::traits::ReadStateSpace;
use typed_index_collections::{Csr, Index, IndexRangeIterator, RawIndex, To1};

// Used by the optional chain-length computer. Determines how chain length is counted.
#[derive(Clone, Copy, Debug)]
pub enum SccChainWeight {
    CountAllScc,            // Every SCC has value 1
    CountNonSingletonSccs,  // Every non-singleton SCC has value 1
    SizeOfNonSingletonSccs, // The value of each SCC corresponds to its size (except for singletons, which get 0)
}

impl<ScI: Index, ScEI: Index, SI: Index> Sccs<ScI, ScEI, SI> {
    pub fn compute_tarjan<
        M: ReadStateSpace<StateIndex = SI>,
        Ex: ExclusionCriterion<SI, M::ChoiceIndex>,
    >(
        model: &M,
        exclusion_criterion: &Ex,
    ) -> Self {
        Self::compute_tarjan_generic(model, exclusion_criterion, ()).0
    }

    pub fn compute_tarjan_with_longest_chain<
        M: ReadStateSpace<StateIndex = SI>,
        Ex: ExclusionCriterion<SI, M::ChoiceIndex>,
    >(
        model: &M,
        exclusion_criterion: &Ex,
        weight: SccChainWeight,
    ) -> (Self, usize) {
        let tracker = LongestChain {
            weight,
            chain_from: Vec::new(),
            longest: 0,
        };
        let (sccs, tracker) = Self::compute_tarjan_generic(model, exclusion_criterion, tracker);
        (sccs, tracker.longest)
    }

    fn compute_tarjan_generic<
        M: ReadStateSpace<StateIndex = SI>,
        Ex: ExclusionCriterion<SI, M::ChoiceIndex>,
        C: ChainTracker,
    >(
        model: &M,
        exclusion_criterion: &Ex,
        mut chain: C,
    ) -> (Self, C) {
        const UNVISITED: usize = 0;
        // States of the n-th completed SCC store `COMPLETED - n`. These values and `EXCLUDED` are
        // larger than any DFS index, so that edges into excluded states or into completed SCCs
        // never lower a lowlink.
        const EXCLUDED: usize = usize::MAX;
        const COMPLETED: usize = usize::MAX - 1;

        let state_count = model.states().len();
        let mut dfs_index = To1::with_entries(vec![UNVISITED; state_count]);
        let mut scc_entry_count = 0;
        for state in model.states() {
            if exclusion_criterion.is_state_excluded(state) {
                dfs_index[state] = EXCLUDED;
            } else {
                scc_entry_count += 1;
            }
        }

        let mut scc_entries: To1<ScEI, SI> = To1::with_capacity(scc_entry_count);
        let mut scc_bounds = vec![0];
        let mut tarjan_stack = Vec::new();
        let mut dfs_stack = Vec::new();
        let mut next_index = 1;

        for root in model.states() {
            if dfs_index[root] != UNVISITED {
                continue;
            }
            dfs_index[root] = next_index;
            dfs_stack.push(Frame {
                state: root,
                index: next_index,
                low: next_index,
                chain_info: chain.new_frame(),
                choices: model.choices_of_state(root).into_iter(),
                branches: IndexRangeIterator::empty(),
            });
            tarjan_stack.push(root);
            next_index += 1;

            while let Some(frame) = dfs_stack.last_mut() {
                let mut descend_into = None;
                loop {
                    if let Some(branch) = frame.branches.next() {
                        let destination = model.branch_destination(branch);
                        let destination_index = dfs_index[destination];
                        if destination_index == UNVISITED {
                            descend_into = Some(destination);
                            break;
                        }
                        frame.low = frame.low.min(destination_index);
                        if destination_index > state_count && destination_index != EXCLUDED {
                            chain.add_dependency(
                                &mut frame.chain_info,
                                COMPLETED - destination_index,
                            );
                        }
                    } else if let Some(choice) = frame.choices.next() {
                        let choice_branches = model.branches_of_choice(choice);
                        chain.add_choice(&mut frame.chain_info, choice_branches.len());
                        if !exclusion_criterion.is_choice_excluded(choice) {
                            frame.branches = choice_branches.into_iter();
                        }
                    } else {
                        break;
                    }
                }

                match descend_into {
                    Some(destination) => {
                        dfs_index[destination] = next_index;
                        dfs_stack.push(Frame {
                            state: destination,
                            index: next_index,
                            low: next_index,
                            chain_info: chain.new_frame(),
                            choices: model.choices_of_state(destination).into_iter(),
                            branches: IndexRangeIterator::empty(),
                        });
                        tarjan_stack.push(destination);
                        next_index += 1;
                    }
                    None => {
                        let frame = dfs_stack.pop().unwrap();
                        if frame.low == frame.index {
                            let scc = scc_bounds.len() - 1;
                            loop {
                                let member = tarjan_stack.pop().unwrap();
                                dfs_index[member] = COMPLETED - scc;
                                scc_entries.add(member);
                                if member == frame.state {
                                    break;
                                }
                            }
                            scc_bounds.push(scc_entries.len());
                            chain.complete_scc(
                                frame.chain_info,
                                scc_bounds[scc + 1] - scc_bounds[scc],
                            );
                            if let Some(parent) = dfs_stack.last_mut() {
                                chain.add_dependency(&mut parent.chain_info, scc);
                            }
                        } else {
                            let parent = dfs_stack.last_mut().unwrap();
                            parent.low = parent.low.min(frame.low);
                            chain.merge(&mut parent.chain_info, frame.chain_info);
                        }
                    }
                }
            }
        }

        // Tarjan completes SCCs in reverse topological order, so we reverse them (this maintains
        // compatibility with the Kosajaru implementation)
        scc_entries.entries_mut().reverse();
        let mut sccs = Csr::with_capacity(scc_bounds.len() - 1);
        for &bound in scc_bounds.iter().rev().skip(1) {
            sccs.add_entry_unchecked(ScEI::from_raw(ScEI::RawType::from_usize(
                scc_entry_count - bound,
            )));
        }

        let mut state_to_scc = To1::with_entries(vec![None; state_count]);
        for (scc, entries) in sccs.ranges().into_iter().enumerate() {
            for entry in entries {
                state_to_scc[scc_entries[entry]] = Some(scc);
            }
        }

        (
            Self {
                sccs,
                scc_entries,
                state_to_scc,
            },
            chain,
        )
    }
}

struct Frame<SI, CI: Index, BI: Index, C> {
    state: SI,
    index: usize,
    low: usize,
    chain_info: C,
    choices: IndexRangeIterator<CI>,
    branches: IndexRangeIterator<BI>,
}

trait ChainTracker {
    type Frame: Copy;
    fn new_frame(&self) -> Self::Frame;
    fn add_choice(&self, frame: &mut Self::Frame, branch_count: usize);
    fn add_dependency(&self, frame: &mut Self::Frame, scc: usize);
    fn merge(&self, frame: &mut Self::Frame, same_scc_child: Self::Frame);
    fn complete_scc(&mut self, root_frame: Self::Frame, state_count: usize);
}

impl ChainTracker for () {
    type Frame = ();
    fn new_frame(&self) {}
    fn add_choice(&self, _frame: &mut (), _branch_count: usize) {}
    fn add_dependency(&self, _frame: &mut (), _scc: usize) {}
    fn merge(&self, _frame: &mut (), _same_scc_child: ()) {}
    fn complete_scc(&mut self, _root_frame: (), _state_count: usize) {}
}

struct LongestChain {
    weight: SccChainWeight,
    chain_from: Vec<usize>,
    longest: usize,
}

#[derive(Clone, Copy)]
struct LongestChainFrame {
    longest_dependency: usize,
    size: usize,
}

impl ChainTracker for LongestChain {
    type Frame = LongestChainFrame;

    fn new_frame(&self) -> LongestChainFrame {
        LongestChainFrame {
            longest_dependency: 0,
            size: 1,
        }
    }

    fn add_choice(&self, frame: &mut LongestChainFrame, branch_count: usize) {
        frame.size += 1 + branch_count;
    }

    fn add_dependency(&self, frame: &mut LongestChainFrame, scc: usize) {
        frame.longest_dependency = frame.longest_dependency.max(self.chain_from[scc]);
    }

    fn merge(&self, frame: &mut LongestChainFrame, same_scc_child: LongestChainFrame) {
        frame.longest_dependency = frame
            .longest_dependency
            .max(same_scc_child.longest_dependency);
        frame.size += same_scc_child.size;
    }

    fn complete_scc(&mut self, root_frame: LongestChainFrame, state_count: usize) {
        let weight = match self.weight {
            SccChainWeight::CountAllScc => 1,
            SccChainWeight::CountNonSingletonSccs if state_count > 1 => 1,
            SccChainWeight::SizeOfNonSingletonSccs if state_count > 1 => root_frame.size,
            _ => 0,
        };
        let chain = root_frame.longest_dependency + weight;
        self.chain_from.push(chain);
        self.longest = self.longest.max(chain);
    }
}
