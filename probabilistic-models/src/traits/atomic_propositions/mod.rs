use crate::Model;
use crate::annotations::AtomicPropositions;
use typed_index_collections::Index;

pub trait ReadAtomicPropositions {
    type StateIdx: Index;
    type APIdx: Index;

    fn is_atomic_proposition_set(
        &self,
        state: Self::StateIdx,
        atomic_proposition: Self::APIdx,
    ) -> bool;
}

macro_rules! derive_read_atomic_propositions {
    ($subcomponent:ident) => {
        fn is_atomic_proposition_set(
            &self,
            state: Self::StateIdx,
            atomic_proposition: Self::APIdx,
        ) -> bool {
            self.$subcomponent
                .is_atomic_proposition_set(state, atomic_proposition)
        }
    };
}
pub(crate) use derive_read_atomic_propositions;

impl<AI: Index, SI: Index, AEI: Index> ReadAtomicPropositions for AtomicPropositions<AI, SI, AEI> {
    type StateIdx = SI;
    type APIdx = AI;

    fn is_atomic_proposition_set(
        &self,
        state: Self::StateIdx,
        atomic_proposition: Self::APIdx,
    ) -> bool {
        self.entries()[atomic_proposition][state]
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs: ReadAtomicPropositions, Rew, Ann, StateVals, Preds>
    ReadAtomicPropositions
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type StateIdx = APs::StateIdx;
    type APIdx = APs::APIdx;

    derive_read_atomic_propositions!(atomic_propositions);
}
