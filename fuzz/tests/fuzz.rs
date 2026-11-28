use fixture::Failed;
use lint::{Cli, Verdict};
use proptest::prelude::{any, prop_assert_eq};
use proptest::test_runner::TestRunner;
use shapes::{Campaign, Input};
use timer_ports::Stopped;

#[test]
fn no_input_panics_the_runner_or_breaks_its_promises() -> Result<(), Failed> {
    TestRunner::default().run(&any::<Input>(), |input| {
        let ran = || {
            let mut err = Vec::new();
            let verdict = Cli::from_args([Campaign::root().into_os_string()]).verdict(
                input.files(),
                Stopped,
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
}
