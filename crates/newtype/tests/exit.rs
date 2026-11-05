use laws::proptest::option;
use laws::proptest::prelude::{any, prop_assert_eq};
use newtype::Exit;

laws::table!(an_exit_succeeds_only_on_code_zero, |code: &Option<i32>| Exit::from(*code), [
    Some(0) => Exit::Succeeded,
    Some(1) => Exit::Failed,
    Some(-1) => Exit::Failed,
    Some(101) => Exit::Failed,
    None => Exit::Failed,
]);

laws::law!(an_exit_succeeds_exactly_when_the_code_is_zero, option::of(any::<i32>()), code, {
    prop_assert_eq!(Exit::from(code) == Exit::Succeeded, code == Some(0));
});
