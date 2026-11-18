use algebra::{Monoid, Semigroup};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct Joined(Vec<u8>);

impl Semigroup for Joined {
    fn combine(self, other: Joined) -> Joined {
        Joined(self.0.into_iter().chain(other.0).collect())
    }
}

impl Monoid for Joined {}

#[test]
fn concatenating_nothing_gives_the_identity_and_keeps_order() {
    assert_eq!(Joined::concat([]), Joined(Vec::new()));
    assert_eq!(Joined::concat([Joined(vec![1]), Joined(vec![2, 3])]), Joined(vec![1, 2, 3]));
}
