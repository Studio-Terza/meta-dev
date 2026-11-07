use equivalence::Laws;

struct Agreement;

impl Agreement {
    fn matched(sides: &[&[u8]; 2]) -> bool {
        Laws::Matched.agree(sides[0], sides[1]).is_some()
    }

    fn where_declared(sides: &[&[u8]; 2]) -> bool {
        Laws::WhereDeclared.agree(sides[0], sides[1]).is_some()
    }

    fn kept(laws: &[Laws]) -> usize {
        laws.iter().map(|group| group.kept(&["first law", "second law"]).len()).sum()
    }
}

laws::table!(
    matched_laws_agree_only_when_both_sides_declare_the_same,
    Agreement::matched,
    [
        [&[1], &[1]] => true,
        [&[1], &[2]] => false,
        [&[1], &[]] => false,
        [&[], &[]] => true,
    ]
);

laws::table!(
    laws_where_declared_agree_unless_both_sides_declare_and_differ,
    Agreement::where_declared,
    [
        [&[1], &[1]] => true,
        [&[1], &[2]] => false,
        [&[1], &[]] => true,
        [&[], &[2]] => true,
    ]
);

laws::table!(
    only_matched_laws_enter_the_key,
    Agreement::kept,
    [
        [Laws::Matched] => 2,
        [Laws::WhereDeclared] => 0,
    ]
);
