use crate::base_model::{BaseModel, Mdp};
use crate::traits::ReadStateSpace;
use crate::Model;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use typed_index_collections::{Index, RawIndex};

impl<M: BaseModel, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
    Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
{
    #[must_use]
    pub fn tra_file(&self) -> TraFile<'_, M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals> {
        TraFile { model: self }
    }
}

pub struct TraFile<'a, M: BaseModel, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals> {
    model: &'a Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>,
}

impl<SI: Index, CI: Index, BI: Index, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
    TraFile<'_, Mdp<SI, CI, BI>, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
{
    pub fn write_to_file(&self, destination: impl AsRef<Path>) -> std::io::Result<()> {
        write!(File::create(destination)?, "{}", self)
    }
}

impl<SI: Index, CI: Index, BI: Index, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals> Display
    for TraFile<'_, Mdp<SI, CI, BI>, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.model.states().len())?;
        write!(f, " {}", self.model.choices().len())?;
        writeln!(f, " {}", self.model.branches().len())?;
        let mut prev_state = None;
        let mut prev_choice = Some(CI::from_raw(CI::RawType::zero()));
        let mut choice_counter = 0;
        for (state, choice, branch) in self.model.base.state_choice_branch_triples() {
            if prev_state != Some(state) {
                choice_counter = 0;
            } else if prev_choice != Some(choice) {
                choice_counter += 1;
            }
            let destination = self.model.base.branch_destinations[branch];
            let probability = self.model.base.branch_probabilities[branch];
            writeln!(
                f,
                "{} {} {} {}",
                state.raw(),
                choice_counter,
                destination.raw(),
                probability
            )?;
            prev_state = Some(state);
            prev_choice = Some(choice);
        }
        Ok(())
    }
}
