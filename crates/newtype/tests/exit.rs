use laws::proptest::option;
use laws::proptest::prelude::{any, prop_assert, prop_assert_eq};
use newtype::Exit;
use std::num::NonZeroI32;
use std::process::{ExitCode, ExitStatus};

const SEVEN: NonZeroI32 = NonZeroI32::new(7).unwrap();
const BYTE_MAX: NonZeroI32 = NonZeroI32::new(255).unwrap();
const PAST_BYTE: NonZeroI32 = NonZeroI32::new(256).unwrap();
const MINUS_ONE: NonZeroI32 = NonZeroI32::new(-1).unwrap();
const NINE: NonZeroI32 = NonZeroI32::new(9).unwrap();
const EXCUSED: [NonZeroI32; 2] = [NonZeroI32::new(3).unwrap(), NonZeroI32::new(4).unwrap()];

laws::table!(an_exit_succeeds_only_on_code_zero, |code: &Option<i32>| Some(Exit::from(*code)), [
    Some(0) => Some(Exit::Succeeded),
    Some(1) => NonZeroI32::new(1).map(Exit::Failed),
    Some(-1) => NonZeroI32::new(-1).map(Exit::Failed),
    Some(101) => NonZeroI32::new(101).map(Exit::Failed),
    None => Some(Exit::Signalled),
    Some(2) => NonZeroI32::new(2).map(Exit::Failed),
]);

laws::law!(an_exit_succeeds_exactly_when_the_code_is_zero, option::of(any::<i32>()), code, {
    prop_assert_eq!(Exit::from(code) == Exit::Succeeded, code == Some(0));
});

laws::law!(a_missing_code_never_equals_success_or_a_code, any::<i32>(), code, {
    prop_assert!(Exit::from(None::<i32>) != Exit::from(Some(code)));
});

laws::table!(a_default_exit_status_succeeds, |_: &u8| Some(Exit::from(ExitStatus::default())), [0 => Some(Exit::Succeeded)]);

laws::table!(an_exit_maps_to_a_process_code, |exit: &Exit| Some(ExitCode::from(*exit)), [
    Exit::Succeeded => Some(ExitCode::from(0)),
    Exit::FAILURE => Some(ExitCode::from(1)),
    Exit::Failed(SEVEN) => Some(ExitCode::from(7)),
    Exit::Failed(BYTE_MAX) => Some(ExitCode::from(255)),
    Exit::Failed(PAST_BYTE) => Some(ExitCode::from(255)),
    Exit::Failed(NonZeroI32::MAX) => Some(ExitCode::from(255)),
    Exit::Failed(MINUS_ONE) => Some(ExitCode::from(1)),
    Exit::Failed(NonZeroI32::MIN) => Some(ExitCode::from(1)),
    Exit::Signalled => Some(ExitCode::from(128)),
]);

laws::table!(only_the_excused_failures_succeed, |code: &i32| Some(Exit::from(Some(*code)).excusing(&EXCUSED)), [
    3 => Some(Exit::Succeeded),
    4 => Some(Exit::Succeeded),
    0 => Some(Exit::Succeeded),
    1 => Some(Exit::FAILURE),
    5 => NonZeroI32::new(5).map(Exit::Failed),
]);

laws::table!(a_signal_is_never_excused, |_: &i32| Some(Exit::Signalled.excusing(&EXCUSED)), [0 => Some(Exit::Signalled)]);

laws::table!(a_step_follows_only_a_success, |exit: &Exit| Some(exit.followed_by(|| Ok::<_, ()>(Exit::FAILURE))), [
    Exit::Succeeded => Some(Ok(Exit::FAILURE)),
    Exit::Signalled => Some(Ok(Exit::Signalled)),
    Exit::Failed(NINE) => Some(Ok(Exit::Failed(NINE))),
]);
