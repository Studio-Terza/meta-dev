use port_core::{RootRefused, Rooting, Run};

#[test]
fn a_sole_run_refuses_others_and_shared_runs_refuse_a_sole_one() {
    let sole = Run::root(Rooting::Sole);
    assert!(sole.is_ok());
    assert_eq!(Run::root(Rooting::Shared).err(), Some(RootRefused));
    assert_eq!(Run::root(Rooting::Sole).err(), Some(RootRefused));
    drop(sole);
    let first = Run::root(Rooting::Shared);
    let second = Run::root(Rooting::Shared);
    assert!(first.is_ok());
    assert!(second.is_ok());
    assert_eq!(Run::root(Rooting::Sole).err(), Some(RootRefused));
    drop(first);
    assert_eq!(Run::root(Rooting::Sole).err(), Some(RootRefused));
    drop(second);
    assert!(Run::root(Rooting::Sole).is_ok());
}

#[test]
fn a_refused_root_says_a_sole_run_is_the_reason() {
    assert!(RootRefused.to_string().contains("sole run"));
}
