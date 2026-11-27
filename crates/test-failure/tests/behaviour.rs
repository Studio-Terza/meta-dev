use std::io;

use proptest::prelude::{any, prop_assert_eq, proptest};
use test_failure::Failed;

proptest! {
    #[test]
    fn a_failure_keeps_the_message_it_was_made_from(message in any::<String>()) {
        prop_assert_eq!(Failed::from(io::Error::other(message.clone())), Failed::from(message));
    }
}
