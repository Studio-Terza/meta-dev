use laws::proptest::collection::vec;
use laws::proptest::prelude::{any, prop_assert_eq};
use newtype::{Extent, Filled, Measured};

struct Measures;

impl Measures {
    fn of_text(text: &&str) -> Extent {
        text.extent()
    }

    fn of_owned_text(text: &String) -> Extent {
        text.extent()
    }

    fn of_list(items: &Vec<u8>) -> Extent {
        items.extent()
    }

    fn kept(items: &[u8]) -> Option<Vec<u8>> {
        Filled::new(items.to_vec()).map(Filled::into_inner)
    }

    fn kept_text(text: &&str) -> Option<String> {
        Filled::new(*text).map(Filled::into_inner).map(str::to_owned)
    }
}

laws::table!(an_extent_reads_whether_text_holds_anything, Measures::of_text, ["" => Extent::Empty, "a" => Extent::Filled, " " => Extent::Filled]);

laws::table!(an_extent_reads_owned_text_as_its_slice, Measures::of_owned_text, [String::new() => Extent::Empty, String::from("仙") => Extent::Filled]);

laws::table!(an_extent_reads_whether_a_list_holds_anything, Measures::of_list, [Vec::new() => Extent::Empty, vec![0] => Extent::Filled]);

laws::table!(filled_wraps_only_a_list_that_holds_something, Measures::kept, [[0_u8; 0] => None, [7_u8, 8] => Some(vec![7, 8])]);

laws::table!(filled_wraps_only_text_that_holds_something, Measures::kept_text, ["" => None, "a" => Some(String::from("a")), " " => Some(String::from(" "))]);

laws::law!(an_extent_is_empty_exactly_when_the_list_is_empty, vec(any::<u8>(), 0..4), items, {
    prop_assert_eq!(items.extent() == Extent::Empty, items.is_empty());
});

laws::law!(an_extent_is_empty_exactly_when_the_text_is_empty, any::<String>(), text, {
    prop_assert_eq!(text.extent() == Extent::Empty, text.is_empty());
});

laws::roundtrip!(
    filled_returns_what_it_wrapped_and_drops_only_the_empty,
    vec(any::<u8>(), 0..4),
    |items: &Vec<u8>| Filled::new(items.clone()),
    |filled: &Option<Filled<Vec<u8>>>| filled.clone().map(Filled::into_inner).unwrap_or_default()
);

struct Reads;

impl Reads {
    fn borrowed(items: &[u8]) -> Option<Vec<u8>> {
        Filled::new(items.to_vec()).map(|filled| (&filled).into_iter().chain(filled.iter()).copied().collect())
    }

    fn owned(items: &[u8]) -> Option<Vec<u8>> {
        Filled::new(items.to_vec()).map(|filled| filled.into_iter().collect())
    }

    fn converted(items: &[u8]) -> Option<Vec<u8>> {
        Filled::new(items.to_vec()).map(Vec::from)
    }
}

laws::table!(filled_iterates_by_reference, Reads::borrowed, [[0_u8; 0] => None, [1_u8, 2] => Some(vec![1, 2, 1, 2])]);

laws::table!(filled_iterates_by_value, Reads::owned, [[0_u8; 0] => None, [1_u8, 2] => Some(vec![1, 2])]);

laws::table!(filled_converts_into_a_vec, Reads::converted, [[0_u8; 0] => None, [5_u8, 6] => Some(vec![5, 6])]);
