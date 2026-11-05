use stable_ids_patch::{Graph, Op, Patch, Rejection, Ty};

fn setup() -> Graph {
    let mut g = Graph::new();
    g.insert_direct(0, "a", Ty::I32, 1);
    g.insert_direct(1, "b", Ty::U32, 2);
    g
}

fn patch(base: u64, ops: Vec<Op>) -> Patch {
    Patch { base, ops }
}

#[test]
fn persistent_id_survives_insertion_but_positional_id_does_not() {
    let mut g = setup();
    let b = g.ids()[1];
    let positional_before = g.positional_id(b).unwrap();
    g.insert_direct(0, "z", Ty::I32, 0);
    let positional_after = g.positional_id(b).unwrap();

    assert_ne!(positional_before, positional_after, "position-derived id changed");
    assert_eq!(g.node(b).unwrap().name, "b", "persistent id still names the same node");
}

#[test]
fn rename_keeps_id_and_changes_content_hash() {
    let mut g = setup();
    let a = g.ids()[0];
    let hash_before = g.node(a).unwrap().content_hash();
    let base = g.graph_hash();

    let p = patch(
        base,
        vec![Op::Rename { target: a, expect_name: "a".into(), to: "x".into() }],
    );
    g.apply(&p).expect("rename applies");

    assert_eq!(g.node(a).unwrap().name, "x");
    assert_ne!(g.node(a).unwrap().content_hash(), hash_before);
    assert!(g.ids().contains(&a));
}

#[test]
fn stale_base_is_rejected_and_store_is_unchanged() {
    let mut g = setup();
    let before = g.clone();
    let p = patch(0xdead_beef, vec![Op::SetValue { target: g.ids()[0], expect_value: 1, to: 9 }]);

    let err = g.apply(&p).unwrap_err();
    assert!(matches!(err, Rejection::Stale { .. }), "got {err:?}");
    assert_eq!(g, before);
}

#[test]
fn replayed_patch_is_stale_after_it_applied_once() {
    let mut g = setup();
    let a = g.ids()[0];
    let base = g.graph_hash();
    let p = patch(base, vec![Op::SetValue { target: a, expect_value: 1, to: 5 }]);

    g.apply(&p).expect("first application");
    let after_first = g.clone();
    let err = g.apply(&p).unwrap_err();

    assert!(matches!(err, Rejection::Stale { .. }));
    assert_eq!(g, after_first);
}

#[test]
fn all_or_nothing_when_a_later_precondition_fails() {
    let mut g = setup();
    let (a, b) = (g.ids()[0], g.ids()[1]);
    let before = g.clone();
    let p = patch(
        g.graph_hash(),
        vec![
            Op::Rename { target: a, expect_name: "a".into(), to: "x".into() },
            Op::SetValue { target: b, expect_value: 999, to: 3 },
        ],
    );

    let err = g.apply(&p).unwrap_err();
    assert!(matches!(err, Rejection::PreconditionFailed { op: 1, .. }), "got {err:?}");
    assert_eq!(g, before, "first op must not be visible");
}

#[test]
fn validation_rejects_out_of_range_u32_and_leaves_store_unchanged() {
    let mut g = setup();
    let b = g.ids()[1];
    let before = g.clone();
    let p = patch(
        g.graph_hash(),
        vec![Op::SetValue { target: b, expect_value: 2, to: 1 << 40 }],
    );

    let err = g.apply(&p).unwrap_err();
    assert!(
        matches!(err, Rejection::Invalid { op: 0, code: "TYP-U32-RANGE", .. }),
        "got {err:?}"
    );
    assert_eq!(g, before);
}

#[test]
fn dry_run_predicts_apply_and_does_not_mutate() {
    let mut g = setup();
    let a = g.ids()[0];
    let before = g.clone();
    let p = patch(g.graph_hash(), vec![Op::SetValue { target: a, expect_value: 1, to: 7 }]);

    let predicted = g.dry_run(&p).expect("dry run accepts");
    assert_eq!(g, before, "dry run must not write");

    let applied = g.apply(&p).expect("apply accepts");
    assert_eq!(predicted, applied);
}

#[test]
fn deleted_id_is_never_reused() {
    let mut g = setup();
    let b = g.ids()[1];
    let content = g.node(b).unwrap().content_hash();
    let p = patch(g.graph_hash(), vec![Op::Delete { target: b, expect_hash: content }]);
    g.apply(&p).expect("delete applies");

    let new_id = g.insert_direct(1, "c", Ty::I32, 0);
    assert_ne!(new_id, b);
    assert!(g.node(b).is_none());
}
