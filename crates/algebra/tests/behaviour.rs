use algebra::{Monoid, Semigroup};
use laws::proptest::collection::vec;
use laws::proptest::prelude::any;
use laws::proptest::strategy::Strategy;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct Joined(Vec<u8>);

impl Semigroup for Joined {
    fn combine(self, other: Joined) -> Joined {
        Joined(self.0.into_iter().chain(other.0).collect())
    }
}

impl Monoid for Joined {}

laws::monoid!(joined_bytes_form_a_monoid, Joined, vec(any::<u8>(), 0..4).prop_map(Joined));

#[test]
fn concatenating_nothing_gives_the_identity_and_keeps_order() {
    assert_eq!(Joined::concat([]), Joined(Vec::new()));
    assert_eq!(Joined::concat([Joined(vec![1]), Joined(vec![2, 3])]), Joined(vec![1, 2, 3]));
}
