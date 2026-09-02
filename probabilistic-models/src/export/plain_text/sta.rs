use crate::Model;
use crate::base_model::BaseModel;
use crate::traits::ReadStateSpace;
use crate::valuations::{Type, ValuationBits, Valuations};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use typed_index_collections::Index;

impl<
    SI: Index,
    CI: Index,
    CEI: Index,
    VI: Index,
    M: BaseModel,
    Ini,
    ChLabel,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
    P,
> Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Valuations<SI, CI, CEI, VI>, P>
{
    #[must_use]
    pub fn sta_file(
        &self,
    ) -> StaFile<'_, M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Valuations<SI, CI, CEI, VI>, P>
    {
        StaFile { model: self }
    }
}

pub struct StaFile<'a, M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, P> {
    model: &'a Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, P>,
}
impl<
    SI: Index,
    CI: Index,
    CEI: Index,
    VI: Index,
    M: BaseModel<StateIdx = SI>,
    Ini,
    ChLabel,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
    P,
> StaFile<'_, M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Valuations<SI, CI, CEI, VI>, P>
{
    pub fn write_to_file(&self, destination: impl AsRef<Path>) -> std::io::Result<()> {
        write!(File::create(destination)?, "{}", self)
    }
}

impl<
    SI: Index,
    CI: Index,
    CEI: Index,
    VI: Index,
    M: BaseModel<StateIdx = SI>,
    Ini,
    ChLabel,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
    P,
> Display
    for StaFile<'_, M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Valuations<SI, CI, CEI, VI>, P>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let classes = self.model.state_valuations.classes();
        assert_eq!(
            classes.len(),
            1,
            "Can only print `.sta` file for models with a single valuation class"
        );
        let class = self.model.state_valuations.class(classes.index(0));
        write!(f, "(")?;
        let mut first = true;
        for variable in class.entries() {
            if !first {
                write!(f, ",")?;
            }
            write!(f, "{}", variable.name)?;
            first = false;
        }
        writeln!(f, ")")?;

        for state in self.model.states() {
            write!(f, "{}:(", state.raw())?;
            let valuation = self.model.state_valuations.entry(state);
            let mut first = true;
            for (variable_index, variable) in class.entries().enumerate() {
                if !first {
                    write!(f, ",")?;
                }
                match variable.variable_type {
                    Type::Bool => {
                        write!(f, "{}", valuation.evaluate_bool(variable_index))?;
                    }
                    Type::Int => {
                        write!(f, "{}", valuation.evaluate_int(variable_index))?;
                    }
                    Type::Uint => {
                        write!(f, "{}", valuation.evaluate_int(variable_index))?;
                    }
                    Type::Double => {
                        write!(f, "{}", valuation.evaluate_double(variable_index))?;
                    }
                    Type::Rational => {
                        let (num, denom) = valuation.evaluate_rational(variable_index);
                        write!(f, "{num}/{denom}")?;
                    }
                    Type::String => {
                        write!(f, "\"{}\"", valuation.evaluate_string(variable_index))?;
                    }
                }
                first = false;
            }
            writeln!(f, ")")?;
        }
        Ok(())
    }
}
