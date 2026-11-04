use equivalence::{Groups, Holes, Keyed};

#[derive(Clone, Copy, Debug)]
struct Item(u8);

impl Keyed for Item {
    type Key = u8;

    fn key(&self) -> u8 {
        self.0
    }
}

struct Grouping;

impl Grouping {
    fn sizes(keys: &[u8]) -> Vec<usize> {
        Groups::grouped(keys.iter().copied().map(Item)).listed().iter().map(|group| group.members().len()).collect()
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
