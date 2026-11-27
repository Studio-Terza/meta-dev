use std::iter;
use std::time::Instant;

use fixture::Failed;
use lint::{Cli, Verdict};
use proptest::prelude::{any, prop_assert_eq};
use proptest::test_runner::TestRunner;
use shapes::{Campaign, Input};
use timer_ports::Stopped;

#[test]
fn no_input_panics_the_runner_or_breaks_its_promises() -> Result<(), Failed> {
    let deadline = Campaign::deadline();
    iter::repeat_with(Instant::now).take_while(|now| *now < deadline).try_for_each(|_| {
        TestRunner::new(Campaign::batch()).run(&any::<Input>(), |input| {
            let files = input.files();
            let ran = || {
                let mut err = Vec::new();
                let verdict = Cli::from_args([Campaign::root().into_os_string()]).verdict(
                    &files,
                    Stopped::default(),
                    &mut Vec::new(),
                    &mut err,
                );
                (verdict, err)
            };
            let (verdict, err) = ran();
            prop_assert_eq!((verdict, err.clone()), ran());
            prop_assert_eq!(err.is_empty(), verdict == Verdict::Clean);
            Ok(())
        })?;
        Ok(())
    })
}
