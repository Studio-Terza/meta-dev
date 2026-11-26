use algebra::Act;
use equivalence::{Agreement, Equivalence, Holes, Laws, Pair, Renaming};
use laws::proptest::collection::vec;
use laws::proptest::option::of;
use laws::proptest::sample::select;
use laws::proptest::strategy::{BoxedStrategy, Strategy};

struct Agreeing;

impl Agreeing {
    fn matched(sides: &[&[u8]; 2]) -> Option<Agreement> {
        Laws::Matched.agree(sides[0], sides[1])
    }

    fn where_declared(sides: &[&[u8]; 2]) -> Option<Agreement> {
        Laws::WhereDeclared.agree(sides[0], sides[1])
    }

    fn kept(laws: &[Laws]) -> usize {
        laws.iter().map(|group| group.kept(&["first law", "second law"]).len()).sum()
    }

    fn differing(sequences: &[&[u8]]) -> Option<String> {
        Holes::between(sequences).differing().map(|differing| differing.to_string())
    }
}

struct Drawn;

impl Drawn {
    const NODES: [u8; 4] = [0, 1, 2, 3];
    const LONGEST: usize = 4;

    fn node() -> BoxedStrategy<u8> {
        select(Drawn::NODES.to_vec()).boxed()
    }

    fn pairs() -> BoxedStrategy<Vec<Pair<u8>>> {
        vec((of(Drawn::node()), of(Drawn::node())).prop_map(|(from, to)| Pair::new(from, to)), 0..Drawn::LONGEST)
            .boxed()
    }

    fn renamings() -> BoxedStrategy<Renaming<u8>> {
        (Drawn::node(), Drawn::node(), Drawn::pairs())
            .prop_filter_map("the pairs conflict with the start", |(from, to, pairs)| {
                Renaming::started(from, to).acted(pairs)
            })
            .boxed()
    }
}

laws::table!(
    matched_laws_agree_only_when_both_sides_declare_the_same,
    Agreeing::matched,
    [
        [&[1], &[1]] => Some(Agreement::Same),
        [&[1], &[2]] => None,
        [&[1], &[]] => None,
        [&[], &[]] => Some(Agreement::Same),
    ]
);

laws::table!(
    laws_where_declared_agree_unless_both_sides_declare_and_differ,
    Agreeing::where_declared,
    [
        [&[1], &[1]] => Some(Agreement::Same),
        [&[1], &[2]] => None,
        [&[1], &[]] => Some(Agreement::Undeclared),
        [&[], &[2]] => Some(Agreement::Undeclared),
    ]
);

laws::table!(
    only_matched_laws_enter_the_key,
    Agreeing::kept,
    [
        [Laws::Matched] => 2,
        [Laws::WhereDeclared] => 0,
    ]
);

#[test]
fn an_equivalence_keeps_laws_only_when_it_compares_matched_laws() {
    let laws = ["first law", "second law"];
    let kept = [
        Equivalence::comparing(&[], Laws::Matched).kept(&laws).len(),
        Equivalence::comparing(&[], Laws::WhereDeclared).kept(&laws).len(),
        Equivalence::erasing(&[]).kept(&laws).len(),
    ];
    assert_eq!(kept, [2, 0, 0]);
}

laws::table!(
    only_sequences_that_disagree_somewhere_differ,
    Agreeing::differing,
    [
        [&[1, 2][..], &[1, 3][..]] => Some(String::from("1")),
        [&[1][..], &[1][..]] => None,
    ]
);

laws::action!(pairs_act_on_a_renaming, Drawn::renamings(), <Renaming<u8> as Act<Vec<Pair<u8>>>>::acted, Drawn::pairs());
