#[macro_export]
macro_rules! mdp {
    (
        $mdp:ident = {
            $($state:ident -> $($p:literal: $dest:ident)&* $($deadlock:ident)?),* $(,)?
        }
    ) => {
        #[allow(unused_mut)]
        let (mut $mdp, __mdp_state_indices) = {
            #[allow(unused_imports)]
            use $crate::Index;
            use $crate::base_model::Mdp;
            use $crate::traits::{BuildChoiceBranchGraph, BuildStates};
            use $crate::{BranchIndex, ChoiceIndex, StateIndex};
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
            let mut deadlock_states: Vec<&'static str> = Vec::new();
            $(
                let branches: Vec<(f64, StateIndex<usize>)> =
                    vec![$( ($p, state_indices[stringify!($dest)]) ),*];
                #[allow(unused_mut, unused_assignments)]
                let mut is_deadlock = false;
                $(
                    assert_eq!(
                        stringify!($deadlock),
                        "deadlock",
                        "Expected `deadlock` or a list of branches after `{} ->`. Use `{} -> 1.0: {}` to express a transition instead",
                        stringify!($state),
                        stringify!($state),
                        stringify!($deadlock)
                    );
                    is_deadlock = true;
                )?
                if is_deadlock {
                    assert!(
                        branches.is_empty(),
                        "`{}` is a deadlock state and cannot have branches.",
                        stringify!($state)
                    );
                    deadlock_states.push(stringify!($state));
                } else {
                    choices_by_state
                        .entry(stringify!($state))
                        .or_insert_with(Vec::new)
                        .push(branches);
                }
            )*
            for state in &deadlock_states {
                assert!(
                    !choices_by_state.contains_key(state),
                    "`{state}` cannot be a deadlock state, as it has choices."
                );
            }

            let mut built: Mdp<StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> =
                Mdp::new();
            for name in &order {
                let state = built.add_state();
                assert_eq!(state, state_indices[name]);
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

#[cfg(test)]
mod tests {
    use crate::traits::ReadStateSpace;
    use crate::{Index, StateIndex};

    #[test]
    fn deadlock_states() {
        mdp!(model = {
            s0 -> 0.5: s1 & 0.5: s3,
            s1 -> deadlock,
            s2 -> 1.0: s3,
            s2 ->,
            s3 -> deadlock
        });
        // States are numbered in order of appearance, deadlock states included.
        assert_eq!(model.states().len(), 4);
        assert_eq!((s0.raw(), s1.raw(), s2.raw(), s3.raw()), (0, 1, 2, 3));
        assert_eq!(model.choices_of_state(s0).len(), 1);
        assert_eq!(model.choices_of_state(s1).len(), 0);
        // `s2 ->` is an additional choice without branches
        assert_eq!(model.choices_of_state(s2).len(), 2);
        assert_eq!(model.choices_of_state(s3).len(), 0);
        assert_eq!(model.choices().len(), 3);
        // Deadlock states can be used as destinations.
        assert_eq!(
            model.branch_destination(model.branches().into_iter().next().unwrap()),
            s1
        );
    }

    #[test]
    fn deadlock_state_first_with_trailing_comma() {
        mdp!(model = { s0 -> deadlock, s1 -> 1.0: s0, });
        let _: StateIndex<usize> = s1;
        assert_eq!((s0.raw(), s1.raw()), (0, 1));
        assert_eq!(model.choices_of_state(s0).len(), 0);
        assert_eq!(model.choices_of_state(s1).len(), 1);
    }

    #[test]
    fn only_deadlock_state() {
        mdp!(model = { s0 -> deadlock });
        assert_eq!(model.states().len(), 1);
        assert_eq!(model.choices().len(), 0);
    }

    #[test]
    #[should_panic(expected = "`s0` cannot be a deadlock state")]
    fn deadlock_state_with_choices() {
        mdp!(_model = { s0 -> 1.0: s0, s0 -> deadlock });
    }

    #[test]
    #[should_panic(expected = "`s0` cannot be a deadlock state")]
    fn deadlock_state_with_choices_after() {
        mdp!(_model = { s0 -> deadlock, s0 -> 1.0: s0 });
    }

    #[test]
    #[should_panic(expected = "Expected `deadlock` or a list of branches after `s0 ->`")]
    fn misspelled_deadlock() {
        mdp!(_model = { s0 -> deadlok });
    }

    #[test]
    #[should_panic(expected = "`s1` is a deadlock state and cannot have branches")]
    fn deadlock_with_branches() {
        mdp!(_model = { s0 -> deadlock, s1 -> 1.0: s0 deadlock });
    }
}
