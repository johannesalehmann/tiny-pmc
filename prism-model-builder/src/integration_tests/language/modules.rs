use super::*;

#[test]
fn multiple_modules_interleaving() {
    let model = build!(
        "mdp module a x: [0..1] init 0; [] x=0 -> (x'=1); endmodule module b y: [0..1] init 0; [] y=0 -> (y'=1); endmodule",
        .with_mdp_model_type()
    )
    .unwrap();
    assert_eq!(
        valuations(&model),
        ["x=0,y=0", "x=1,y=0", "x=0,y=1", "x=1,y=1"]
    );
    mdp!(expected = {
        s00 -> 1.0: s10,
        s00 -> 1.0: s01,
        s10 -> 1.0: s11,
        s01 -> 1.0: s11,
        s11 -> 1.0: s11,
    });
    assert_eq!(transitions(&model), transitions(&expected));
}

#[test]
fn synchronisation_two_modules() {
    let model = build!(
        "mdp module a x: [0..2] init 0; [s] x=0 -> 0.5: (x'=1) + 0.5: (x'=2); endmodule module b y: [0..2] init 0; [s] y=0 -> 0.25: (y'=1) + 0.75: (y'=2); endmodule",
        .with_mdp_model_type()
    )
    .unwrap();
    assert_eq!(
        valuations(&model),
        ["x=0,y=0", "x=1,y=1", "x=1,y=2", "x=2,y=1", "x=2,y=2"]
    );
    mdp!(expected = {
        s00 -> 0.125: s11 & 0.375: s12 & 0.125: s21 & 0.375: s22,
        s11 -> 1.0: s11,
        s12 -> 1.0: s12,
        s21 -> 1.0: s21,
        s22 -> 1.0: s22,
    });
    assert_eq!(transitions(&model), transitions(&expected));
}

#[test]
fn synchronisation_one_guard_false() {
    let model = build!(
        "mdp module a x: [0..1] init 0; [s] x=0 -> (x'=1); endmodule module b y: [0..1] init 0; [s] y=1 -> (y'=0); [] y=0 -> (y'=1); endmodule",
        .with_mdp_model_type()
    )
    .unwrap();
    // In `x=0,y=0`, only the local command of `b` is enabled.
    assert_eq!(
        valuations(&model),
        ["x=0,y=0", "x=0,y=1", "x=1,y=0", "x=1,y=1"]
    );
    mdp!(expected = {
        s00 -> 1.0: s01,
        s01 -> 1.0: s10,
        s10 -> 1.0: s11,
        s11 -> 1.0: s11,
    });
    assert_eq!(transitions(&model), transitions(&expected));
}

#[test]
fn synchronisation_multiple_commands_per_module() {
    let model = build!(
        "mdp module a x: [0..2] init 0; [s] x=0 -> (x'=1); [s] x=0 -> (x'=2); endmodule module b y: [0..1] init 0; [s] y=0 -> (y'=1); endmodule",
        .with_mdp_model_type()
    )
    .unwrap();
    assert_eq!(valuations(&model), ["x=0,y=0", "x=1,y=1", "x=2,y=1"]);
    mdp!(expected = {
        s00 -> 1.0: s11,
        s00 -> 1.0: s21,
        s11 -> 1.0: s11,
        s21 -> 1.0: s21,
    });
    assert_eq!(transitions(&model), transitions(&expected));
}

#[test]
fn action_in_single_module() {
    let model = build!(
        "mdp module a x: [0..1] init 0; [go] x=0 -> (x'=1); endmodule module b y: [0..1] init 0; [] y=0 -> (y'=1); endmodule",
        .with_mdp_model_type()
    )
    .unwrap();
    assert_eq!(
        valuations(&model),
        ["x=0,y=0", "x=0,y=1", "x=1,y=0", "x=1,y=1"]
    );
    // Choices of actions come after the local commands.
    mdp!(expected = {
        s00 -> 1.0: s01,
        s00 -> 1.0: s10,
        s01 -> 1.0: s11,
        s10 -> 1.0: s11,
        s11 -> 1.0: s11,
    });
    assert_eq!(transitions(&model), transitions(&expected));
}

#[test]
fn global_variables() {
    let model = build!(
        "mdp global g: [0..2] init 0; module a [] g=0 -> (g'=1); endmodule module b [] g=1 -> (g'=2); endmodule",
        .with_mdp_model_type()
    )
    .unwrap();
    assert_eq!(valuations(&model), ["g=0", "g=1", "g=2"]);
    mdp!(expected = { g0 -> 1.0: g1, g1 -> 1.0: g2, g2 -> 1.0: g2 });
    assert_eq!(transitions(&model), transitions(&expected));
}

#[test]
fn module_renaming() {
    const SOURCE: &str = r"
        mdp
        module ma
            x: [0..1] init 0;
            [a] x=0 -> (x'=1);
        endmodule
        module mb = ma [x=y, a=b] endmodule
        module mc
            z: [0..1] init 0;
            [b] x=1 & z=0 -> (z'=1);
        endmodule
    ";
    let model = build!(SOURCE, .with_mdp_model_type()).unwrap();
    // `[a]` is only in `ma` (so `mb` does not take part), `[b]` synchronises `mb` and `mc`. The
    // renamed variable `y` comes last.
    assert_eq!(
        valuations(&model),
        ["x=0,z=0,y=0", "x=1,z=0,y=0", "x=1,z=1,y=1"]
    );
    mdp!(expected = {
        s000 -> 1.0: s100,
        s100 -> 1.0: s111,
        s111 -> 1.0: s111,
    });
    assert_eq!(transitions(&model), transitions(&expected));
}
