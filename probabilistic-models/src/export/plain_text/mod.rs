use crate::Model;
use crate::base_model::{BaseModel, Mdp};
use crate::traits::ReadStateSpace;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use typed_index_collections::{Index, RawIndex};

impl<M: BaseModel, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
    Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
{
    #[must_use]
    pub fn sta_file(&self) -> TraFile<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals> {
        TraFile { model: self }
    }
}

struct TraFile<'a, M: BaseModel, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals> {
    model: &'a Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>,
}

impl<SI: Index, CI: Index, BI: Index, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
    TraFile<'_, Mdp<SI, CI, BI>, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
{
    pub fn write_to_file(&self, destination: impl AsRef<Path>) -> std::io::Result<()> {
        let mut file = File::create(destination)?;
        write!(file, "{}", self)?;
        Ok(())
    }
}

impl<SI: Index, CI: Index, BI: Index, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals> Display
    for TraFile<'_, Mdp<SI, CI, BI>, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.model.states().len())?;
        write!(f, "{}", self.model.choices().len())?;
        writeln!(f, "{}", self.model.branches().len())?;
        for (state, choice, action) in self.model.base.state_choice_branch_triples() {
            writeln!(f, "{} {} {}", state.raw(), choice.raw(), action.raw())?;
        }
        Ok(())
    }
}
