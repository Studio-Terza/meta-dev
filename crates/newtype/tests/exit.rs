use laws::proptest::option;
use laws::proptest::prelude::{any, prop_assert, prop_assert_eq};
use newtype::Exit;
use std::num::NonZeroI32;

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
