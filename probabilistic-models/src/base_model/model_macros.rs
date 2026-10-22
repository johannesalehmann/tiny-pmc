#[macro_export]
macro_rules! mdp {
    (
        $mdp:ident = {
            $($state:ident -> $($p:literal: $dest:ident)&* $($deadlock:ident)?),* $(,)?
        }
    ) => {
        $crate::__choice_branch_model!(mdp $mdp = [] {
            $($state -> $($p: $dest)&* $($deadlock)?),*
        });
    };
}

// Like `mdp!`, but first lists the players and then every line names the owner of its state, e.g.
// `players: [p1, p2], s0 (p1) -> 1.0: s1`. Players are indexed in the order in which they are
// listed. All lines of a state must name the same owner.
#[macro_export]
macro_rules! smg {
    (
        $smg:ident = {
            players: [$($player:ident),* $(,)?],
            $($state:ident ($owner:ident) -> $($p:literal: $dest:ident)&* $($deadlock:ident)?),* $(,)?
        }
    ) => {
        $crate::__choice_branch_model!(smg $smg = [$($player),*] {
            $($state ($owner) -> $($p: $dest)&* $($deadlock)?),*
        });
    };
}

// Shared implementation of `mdp!` and `smg!`. The public macros restrict which lines may name an
// owner, so the `mdp` mode never sees owners and the `smg` mode always does.
#[doc(hidden)]
#[macro_export]
macro_rules! __choice_branch_model {
    (@model_type mdp) => { $crate::base_model::Mdp<$crate::Uniform<usize>> };
    (@model_type smg) => { $crate::base_model::StochasticGame<$crate::Uniform<usize>> };

    (@add_players mdp $built:ident, $players:ident) => {};
    (@add_players smg $built:ident, $players:ident) => {
        for player in &$players {
            $built.players.add_player(player.to_string());
        }
    };

    (@add_state mdp $built:ident, $owner_by_state:ident, $name:ident) => {
        $crate::traits::BuildStates::add_state(&mut $built)
    };
    (@add_state smg $built:ident, $owner_by_state:ident, $name:ident) => {
        $built.add_state($owner_by_state[$name])
    };

    (
        $mode:ident $model:ident = [$($player:ident),*] {
            $($state:ident $(($owner:ident))? -> $($p:literal: $dest:ident)&* $($deadlock:ident)?),*
        }
    ) => {
        #[allow(unused_mut)]
        let (mut $model, __model_state_indices) = {
            #[allow(unused_imports)]
            use $crate::Index;
            use $crate::traits::BuildChoiceBranchGraph;
            use $crate::{PlayerIndex, StateIndex, Uniform};
            use ::std::collections::HashMap;

            let mut state_indices: HashMap<&'static str, StateIndex<Uniform<usize>>> = HashMap::new();
            let mut order: Vec<&'static str> = Vec::new();
            $(
                state_indices.entry(stringify!($state)).or_insert_with(|| {
                    let index = StateIndex::from_raw(order.len());
                    order.push(stringify!($state));
                    index
                });
            )*

            #[allow(unused_variables)]
            let players: Vec<&'static str> = vec![$(stringify!($player)),*];
            for (index, player) in players.iter().enumerate() {
                assert!(
                    !players[..index].contains(player),
                    "Player `{player}` is listed more than once."
                );
            }

            #[allow(unused_mut, unused_variables)]
            let mut owner_by_state: HashMap<&'static str, PlayerIndex<Uniform<usize>>> =
                HashMap::new();
            $($(
                let owner_name = stringify!($owner);
                let owner = PlayerIndex::from_raw(
                    players
                        .iter()
                        .position(|player| *player == owner_name)
                        .unwrap_or_else(|| {
                            panic!(
                                "`{}` is owned by `{owner_name}`, which is not listed as a player.",
                                stringify!($state)
                            )
                        }) as u32,
                );
                let previous = *owner_by_state.entry(stringify!($state)).or_insert(owner);
                assert_eq!(
                    previous,
                    owner,
                    "`{}` cannot be owned by both `{}` and `{owner_name}`.",
                    stringify!($state),
                    players[previous.raw() as usize]
                );
            )?)*

            let mut choices_by_state: HashMap<&'static str, Vec<Vec<(f64, StateIndex<Uniform<usize>>)>>> =
                HashMap::new();
            let mut deadlock_states: Vec<&'static str> = Vec::new();
            $(
                let branches: Vec<(f64, StateIndex<Uniform<usize>>)> =
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

            let mut built: $crate::__choice_branch_model!(@model_type $mode) = Default::default();
            $crate::__choice_branch_model!(@add_players $mode built, players);
            for name in &order {
                let state = $crate::__choice_branch_model!(@add_state $mode built, owner_by_state, name);
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
            let $state = __model_state_indices[stringify!($state)];
        )*
    };
}

#[cfg(test)]
mod tests {
    use crate::traits::ReadStateSpace;
    use crate::{Index, StateIndex, Uniform};

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
        let _: StateIndex<Uniform<usize>> = s1;
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
