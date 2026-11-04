use equivalence::{Groups, Holes, Keyed, Nested};

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
            .filter(|outer| outer.from <= self.from)
            .filter(|outer| self.from + self.length <= outer.from + outer.length)
    }
}

struct Grouping;

impl Grouping {
    fn sizes(keys: &[u8]) -> Vec<usize> {
        Groups::grouped(keys.iter().copied().map(Item)).listed().iter().map(|group| group.members().len()).collect()
    }

    fn outermost(spans: &[u8]) -> Vec<u8> {
        let found = spans.as_chunks::<2>().0.iter().map(|[length, from]| Span { length: *length, from: *from });
        Groups::grouped(found).outermost(|length| *length).listed().iter().map(|group| *group.key()).collect()
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
        [3, 1, 3, 21, 10, 0, 10, 20] => vec![10],
        [3, 1, 3, 21, 10, 40, 10, 60] => vec![10, 3],
    ]
);
