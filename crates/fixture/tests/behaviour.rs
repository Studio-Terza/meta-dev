use fixture::{Cases, Failed, Fixture, Spelled};
use serde::Deserialize;

#[derive(Fixture, Deserialize, Debug)]
struct Paired {
    printed: Spelled,
}

#[derive(Fixture, Deserialize, Debug)]
struct Unpaired {
    printed: Spelled,
}

#[test]
fn a_case_printing_what_its_golden_holds_has_not_drifted() -> Result<(), Failed> {
    let drifted = Cases::<Paired>::drifted(|case| Ok(case.into_row().printed))?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}

#[test]
fn a_case_with_no_golden_beside_it_is_refused() {
    let refused = Cases::<Unpaired>::drifted(|case| Ok(case.into_row().printed));
    assert!(refused.is_err());
}
