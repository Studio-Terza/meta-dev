use fixture::{Cases, Failed, Fixture, Laws, Rows, Spelled};
use serde::Deserialize;

#[derive(Fixture, Deserialize, Debug)]
struct Paired {
    printed: Spelled,
}

#[derive(Fixture, Deserialize, Debug)]
struct Unpaired {
    printed: Spelled,
}

#[derive(Fixture, Deserialize, Debug)]
struct Drifting {
    printed: Spelled,
}

#[derive(Fixture, Deserialize, Debug)]
struct Asked {
    calls: Vec<Spelled>,
}

#[test]
fn a_case_printing_what_its_golden_holds_has_not_drifted() -> Result<(), Failed> {
    let drifted = Cases::<Paired>::drifted(|case| Ok(case.into_row().printed))?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}

#[test]
fn a_case_printing_other_than_its_golden_has_drifted() -> Result<(), Failed> {
    let drifted = Cases::<Drifting>::drifted(|case| Ok(case.into_row().printed))?;
    assert_ne!(drifted, Vec::new());
    Ok(())
}

#[test]
fn a_case_with_no_golden_beside_it_is_refused() {
    let refused = Cases::<Unpaired>::drifted(|case| Ok(case.into_row().printed));
    assert!(refused.is_err());
}

#[test]
fn every_row_is_read_and_an_implementor_disagreeing_with_its_reference_is_refused() -> Result<(), Failed> {
    let asked = Rows::<Asked>::read()?;
    assert_ne!(asked.iter().map(|row| row.calls.len()).sum::<usize>(), usize::default());
    asked.iter().try_for_each(|row| {
        Laws::agreed(&row.calls, Spelled::to_string, Spelled::to_string)?;
        let refused = Laws::agreed(&row.calls, Spelled::to_string, |_call| String::new());
        assert!(refused.is_err());
        Ok(())
    })
}
