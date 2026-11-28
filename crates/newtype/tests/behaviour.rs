use std::{iter, mem};

use design_messages::CheckName;
use file_ports::{Offset, Text};
use fixture::{Cases, Debugged, Failed, Fixture, Properties};
use grammar_spec_line::Attachment;
use invariants::{Ignoring, ignored};
use newtype::{Checked, Counter, Wrapping};
use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};
use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::{Config, TestRunner};
use serde::Deserialize;
use serde_json::Value;
use store_ports::Written;
use transcript_ports::Body;

ignored! {
    enum Ignored {
        Refused => "a text the check refuses is the None the law compares with the refused read",
    }
}

#[derive(Fixture, Deserialize, Debug)]
struct Sampled {}

proptest! {
    #![proptest_config(Properties::settled())]

    #[test]
    fn a_counter_adds_saturating_from_zero(counted in any::<Written>(), other in any::<Written>(), many in any::<Vec<Written>>()) {
        prop_assert_eq!(Written::new(counted.into_inner()), counted);
        prop_assert_eq!(Written::zero().count(), usize::MIN);
        prop_assert_eq!(counted.plus(Written::zero()), counted);
        prop_assert_eq!(counted.plus(other).count(), counted.count().saturating_add(other.count()));
        prop_assert_eq!(counted.incremented(), counted.plus(Written::zero().incremented()));
        prop_assert_eq!(Written::total(many.iter().copied()), many.iter().copied().fold(Written::zero(), Written::plus));
        prop_assert_eq!(Written::new(usize::MAX).incremented(), Written::new(usize::MAX));
    }

    #[test]
    fn a_checked_newtype_builds_reads_and_writes_only_what_its_check_accepts(text in any::<String>()) {
        let built = Body::try_new(text.clone()).ignoring(Ignored::Refused);
        prop_assert_eq!(built.is_some(), Body::checked(&text).is_ok());
        let read = serde_json::from_value::<Body>(Value::String(text.clone())).ignoring(Ignored::Refused);
        prop_assert_eq!(&read, &built);
        prop_assert_eq!(built.map(|made| made.to_string()), Body::checked(&text).ignoring(Ignored::Refused).cloned());
        let named = CheckName::try_new(text.clone()).ignoring(Ignored::Refused);
        let written = named.as_ref().map(serde_json::to_value).transpose()?;
        prop_assert_eq!(written, named.map(|_| Value::String(text)));
    }

    #[test]
    fn a_plain_newtype_reads_and_shows_what_it_holds(text in any::<String>(), offset in any::<Offset>()) {
        prop_assert_eq!(serde_json::from_value::<Text>(Value::String(text.clone()))?, Text::new(text.clone()));
        prop_assert_eq!(Text::new(text.clone()).to_string(), text);
        prop_assert_eq!(offset.to_string(), offset.into_inner().to_string());
    }
}

#[test]
fn checked_generators_draw_only_accepted_values_within_the_rejection_bound() {
    let settled = Properties::settled();
    let mut bounded = TestRunner::new(Config { max_local_rejects: settled.cases, ..settled });
    let drawn = (any::<Body>(), any::<Offset>(), any::<Attachment>());
    let accepted = bounded.run(&drawn, |(body, offset, attachment)| {
        prop_assert!(Body::checked(&body.into_inner()).is_ok());
        prop_assert!(Offset::checked(&offset.into_inner()).is_ok());
        prop_assert!(Attachment::checked(&attachment.into_inner()).is_ok());
        Ok(())
    });
    assert!(accepted.is_ok());
}

#[test]
fn an_enum_generator_draws_the_values_its_golden_lists() -> Result<(), Failed> {
    let drifted = Cases::<Sampled>::drifted(|_case| {
        let settled = Properties::settled();
        let cases = usize::try_from(settled.cases)?;
        let mut runner = TestRunner::new(settled);
        let drawn = iter::repeat_with(|| {
            any::<store_ports::Value>().new_tree(&mut runner).map(|tree| mem::discriminant(&tree.current()))
        })
        .take(cases)
        .collect::<Result<Vec<_>, _>>()?;
        Ok(Debugged(drawn).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
