use laws::proptest::option;
use laws::proptest::prelude::{any, prop_assert_eq};
use newtype::Exit;
use std::num::NonZeroI32;

laws::table!(an_exit_succeeds_only_on_code_zero, |code: &Option<i32>| Exit::from(*code), [
    Some(0) => Exit::Succeeded,
    Some(1) => Exit::Failed(NonZeroI32::new(1)),
    Some(-1) => Exit::Failed(NonZeroI32::new(-1)),
    Some(101) => Exit::Failed(NonZeroI32::new(101)),
    None => Exit::Failed(None),
    Some(2) => Exit::Failed(NonZeroI32::new(2)),
]);

laws::law!(an_exit_succeeds_exactly_when_the_code_is_zero, option::of(any::<i32>()), code, {
    prop_assert_eq!(Exit::from(code) == Exit::Succeeded, code == Some(0));
});

laws::law!(a_failed_exit_keeps_the_code_it_was_given, option::of(any::<i32>()), code, {
    prop_assert_eq!(Exit::from(code).code().map(NonZeroI32::get), code.filter(|code| *code != 0));
});

laws::table!(two_failing_codes_stay_distinct, |code: &i32| Exit::from(Some(*code)).code(), [
    1 => NonZeroI32::new(1),
    2 => NonZeroI32::new(2),
    0 => None,
]);
