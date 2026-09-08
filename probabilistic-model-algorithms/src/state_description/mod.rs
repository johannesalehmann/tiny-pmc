use probabilistic_models::traits::{ReadAtomicPropositions, ReadStateSpace};
use typed_index_collections::{Index, RawIndex, To1};

pub enum StateDescription<
    'a,
    M: ReadStateSpace + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>,
> {
    AtomicProposition { ap_index: M::APIdx, model: &'a M },
    Flags(To1<<M as ReadStateSpace>::StateIdx, bool>),
    Conjunction(Box<Self>, Box<Self>),
    Disjunction(Box<Self>, Box<Self>),
    Negation(Box<Self>),
}

impl<'a, M: ReadStateSpace + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>>
    StateDescription<'a, M>
{
    pub fn len(&self) -> usize {
        match self {
            StateDescription::AtomicProposition { model, .. } => model.states().len(),
            StateDescription::Flags(flags) => flags.len(),
            StateDescription::Conjunction(lhs, rhs) => {
                let lhs_len = lhs.len();
                let rhs_len = rhs.len();
                assert_eq!(
                    lhs_len, rhs_len,
                    "Left-hand side and right-hand side of state description conjunction have different lengths"
                );
                lhs_len
            }
            StateDescription::Disjunction(lhs, rhs) => {
                let lhs_len = lhs.len();
                let rhs_len = rhs.len();
                assert_eq!(
                    lhs_len, rhs_len,
                    "Left-hand side and right-hand side of state description disjunction have different lengths"
                );
                lhs_len
            }
            StateDescription::Negation(inner) => inner.len(),
        }
    }

    pub fn is_set(&self, state: <M as ReadStateSpace>::StateIdx) -> bool {
        match self {
            StateDescription::AtomicProposition { ap_index, model } => {
                model.is_atomic_proposition_set(state, *ap_index)
            }
            StateDescription::Flags(flags) => flags[state],
            StateDescription::Conjunction(lhs, rhs) => lhs.is_set(state) && rhs.is_set(state),
            StateDescription::Disjunction(lhs, rhs) => lhs.is_set(state) || rhs.is_set(state),
            StateDescription::Negation(inner) => !inner.is_set(state),
        }
    }

    pub fn write_flags(&self, flags: &mut To1<<M as ReadStateSpace>::StateIdx, bool>) {
        // We enforce this check because `copy_from_other` also requires this:
        assert_eq!(
            flags.len(),
            self.len(),
            "Input `flags` of `write_flags` does not have the correct length"
        );
        match self {
            StateDescription::Flags(own_flags) => flags.copy_from_other(own_flags),
            _ => {
                flags.clear();
                for state in 0..self.len() {
                    let state = <M as ReadStateSpace>::StateIdx::from_raw(
                        <<M as ReadStateSpace>::StateIdx as Index>::RawType::from_usize(state),
                    );
                    flags.add(self.is_set(state));
                }
            }
        }
    }
}
