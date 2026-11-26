use algebra::{Act, Catenation, Monoid, Semigroup};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Capped(usize);

impl Capped {
    const MOST: usize = 4;
}

impl Act<Joined> for Capped {
    fn acted(self, by: Joined) -> Option<Capped> {
        Some(Capped(self.0 + by.0.len())).filter(|grown| grown.0 <= Capped::MOST)
    }
}

laws::action!(
    bytes_act_on_a_capped_count,
    (0..=Capped::MOST).prop_map(Capped),
    <Capped as Act<Joined>>::acted,
    vec(any::<u8>(), 0..3).prop_map(Joined)
);

#[test]
fn an_optional_target_is_acted_on_when_present_and_stays_absent_otherwise() {
    assert_eq!(Some(Capped(1)).acted(Joined(vec![7])), Some(Some(Capped(2))));
    assert_eq!(Some(Capped(Capped::MOST)).acted(Joined(vec![7])), Some(None));
    assert_eq!(None::<Capped>.acted(Joined(vec![7])), Some(None));
}

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
