use algebra::{Catenation, Monoid, Semigroup};

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

#[test]
fn a_catenation_yields_its_items_in_order_however_deeply_it_is_joined() {
    let deep = (0..20_000).fold(Catenation::default(), |joined, item| Catenation::from(vec![item]).combine(joined));
    let items: Vec<u32> = deep.into_iter().collect();
    assert_eq!(items, (0..20_000).rev().collect::<Vec<u32>>());
    let concatenated =
        Catenation::concat([Catenation::from(vec![1]), Catenation::default(), Catenation::from(vec![2, 3])]);
    assert_eq!(concatenated.into_iter().collect::<Vec<u8>>(), vec![1, 2, 3]);
}
