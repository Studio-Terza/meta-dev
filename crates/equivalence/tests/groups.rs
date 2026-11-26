use std::path::Path;

use equivalence::{Groups, Holes, Keyed, Nested};
use graph::Site;

#[derive(Clone, Copy, Debug)]
struct Item(u8);

impl Keyed for Item {
    type Key = u8;

    fn key(&self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
struct Span {
    length: u8,
    from: u8,
    site: Site<'static>,
}

impl AsRef<Site<'static>> for Span {
    fn as_ref(&self) -> &Site<'static> {
        &self.site
    }
}

impl Keyed for Span {
    type Key = u8;

    fn key(&self) -> u8 {
        self.length
    }
}

impl Nested for Span {
    fn lies_within<'outer>(&self, outer: &'outer Span) -> Option<&'outer Span> {
        Some(outer)
            .filter(|starting| starting.from <= self.from)
            .filter(|covering| self.from + self.length <= covering.from + covering.length)
    }
}

struct Grouping;

impl Grouping {
    const FILES: [&str; 2] = ["a.rs", "b.rs"];

    fn sizes(keys: &[u8]) -> Vec<usize> {
        Groups::grouped(keys.iter().copied().map(Item)).listed().iter().map(|group| group.members().len()).collect()
    }

    fn outermost(spans: &[u8]) -> Vec<u8> {
        let found = spans.as_chunks::<3>().0.iter().filter_map(|[length, from, file]| {
            let path = Path::new(Self::FILES.get(usize::from(*file))?);
            Some(Span { length: *length, from: *from, site: Site::of_file(path) })
        });
        Groups::grouped(found).outermost(|length| *length).iter().map(|group| *group.key()).collect()
    }

    fn holes(sequences: &[&[u8]]) -> String {
        Holes::between(sequences).to_string()
    }
}

laws::table!(
    a_group_has_two_members_or_more,
    Grouping::sizes,
    [
        [1, 2, 2, 3] => vec![2],
        [1, 2, 3] => Vec::<usize>::new(),
        [4, 4, 4, 5, 5] => vec![3, 2],
    ]
);

laws::table!(
    holes_count_every_position_any_sequence_disagrees_at_including_extra_and_missing_ones,
    Grouping::holes,
    [
        [&[1, 2][..], &[1, 3][..]] => "1",
        [&[1, 2][..], &[1, 2, 3][..]] => "1",
        [&[][..], &[1][..]] => "1",
        [&[1][..], &[1][..]] => "0",
    ]
);

laws::table!(
    a_group_inside_a_larger_group_is_dropped_whatever_order_the_keys_come_in,
    Grouping::outermost,
    [
        [3, 1, 0, 3, 21, 0, 10, 0, 0, 10, 20, 0] => vec![10],
        [3, 1, 0, 3, 21, 0, 10, 40, 0, 10, 60, 0] => vec![10, 3],
        [3, 1, 0, 3, 21, 0, 10, 0, 1, 10, 20, 1] => vec![10, 3],
        [10, 0, 0, 10, 20, 0, 8, 40, 0, 8, 60, 0, 3, 41, 0, 3, 61, 0] => vec![10, 8],
    ]
);
