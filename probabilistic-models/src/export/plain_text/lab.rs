use crate::Model;
use crate::annotations::AtomicPropositions;
use crate::base_model::BaseModel;
use crate::traits::{ReadInitialStates, ReadStateSpace};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use typed_index_collections::{Index, RawIndex};

impl<
    'a,
    AI: Index,
    SI: Index,
    AEI: Index,
    M: BaseModel<StateIdx = SI>,
    Ini: ReadInitialStates<StateIdx = SI>,
    C,
    B,
    O,
    R,
    A,
    SV,
    P,
> Model<M, Ini, C, B, O, AtomicPropositions<AI, SI, AEI>, R, A, SV, P>
{
    pub fn lab_file(
        &self,
    ) -> LabFile<'_, M, Ini, C, B, O, AtomicPropositions<AI, SI, AEI>, R, A, SV, P> {
        LabFile { model: self }
    }
}

pub struct LabFile<'a, M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, P> {
    model: &'a Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, P>,
}

impl<
    'a,
    AI: Index,
    SI: Index,
    AEI: Index,
    M: BaseModel<StateIdx = SI>,
    Ini: ReadInitialStates<StateIdx = SI>,
    C,
    B,
    O,
    R,
    A,
    SV,
    P,
> LabFile<'a, M, Ini, C, B, O, AtomicPropositions<AI, SI, AEI>, R, A, SV, P>
{
    pub fn write_to_file(&self, destination: impl AsRef<Path>) -> std::io::Result<()> {
        write!(File::create(destination)?, "{}", self)
    }
}

impl<
    'a,
    AI: Index,
    SI: Index,
    AEI: Index,
    M: BaseModel<StateIdx = SI>,
    Ini: ReadInitialStates<StateIdx = SI>,
    C,
    B,
    O,
    R,
    A,
    SV,
    P,
> Display for LabFile<'a, M, Ini, C, B, O, AtomicPropositions<AI, SI, AEI>, R, A, SV, P>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0=\"init\"")?;
        for (index, name) in self.model.atomic_propositions.names().enumerate() {
            write!(f, " {}=\"{}\"", index.raw().as_usize() + 1, name)?;
        }
        writeln!(f)?;
        let mut satisfied = vec![false; self.model.atomic_propositions.len() + 1];
        for state in self.model.states() {
            for entry in &mut satisfied {
                *entry = false;
            }
            let mut any_true = false;
            if self.model.initial.is_initial(state) {
                satisfied[0] = true;
                any_true = true;
            }
            for index in self.model.atomic_propositions.internal_indices() {
                let value = self.model.atomic_propositions[index][state];
                satisfied[index.raw().as_usize() + 1] = value;
                if value {
                    any_true = true;
                }
            }
            if any_true {
                write!(f, "{}:", state.raw())?;
                for (i, value) in satisfied.iter().enumerate() {
                    if *value {
                        write!(f, " {}", i)?;
                    }
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
