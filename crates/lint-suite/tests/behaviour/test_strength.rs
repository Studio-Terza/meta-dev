use lint_fixtures::{Located, Source, Sources, Text};
use lint_test_strength::TestStrength;
use lint_transition_graph::{ChangeLint, Extraction, TypeSystem};

struct Weakened;

impl Weakened {
    fn report(before: &[Source], after: &[Source]) -> String {
        let (before, after) = (Sources::parsed(before), Sources::parsed(after));
        let (before_extraction, after_extraction) =
            (Extraction::from(before.as_slice()), Extraction::from(after.as_slice()));
        TestStrength::report_change(
            &TypeSystem::built_from(&before, &before_extraction),
            &TypeSystem::built_from(&after, &after_extraction),
        )
        .to_string()
    }

    fn lost(before: &[Source], after: &[Source]) -> usize {
        Self::report(before, after).matches("lost a check").count()
    }

    fn reported(pair: [&str; 2]) -> String {
        Self::report(&[Self::module(pair[0])], &[Self::module(pair[1])])
    }

    fn mentions(row: &[&str]) -> bool {
        Some(row)
            .filter(|row| row.len() == 3)
            .and_then(|row| row.first().zip(row.get(1)).zip(row.last()))
            .is_some_and(|((before, after), needle)| Self::reported([before, after]).contains(needle))
    }

    fn between(before: &[Source], after: &[Source]) -> bool {
        Self::lost(before, after) > 0
    }

    fn in_one_module(pair: &[&str]) -> bool {
        Some(pair)
            .filter(|pair| pair.len() == 2)
            .and_then(|pair| pair.first().zip(pair.last()))
            .is_some_and(|(before, after)| Self::between(&[Self::module(before)], &[Self::module(after)]))
    }

    fn module(body: &str) -> Source {
        Self::placed("crates/x/src/lib.rs", body)
    }

    fn placed(path: &'static str, body: &str) -> Source {
        Self::at(path, &format!("#[cfg(test)]\nmod tests {{\n{body}\n}}\n"))
    }

    fn at(path: &'static str, text: &str) -> Source {
        Source::new(Located::new(path), Text::new(Box::leak(text.to_owned().into_boxed_str())))
    }
}

const EQUALS: &str = "#[test]\nfn adds() {\n    assert_eq!(f(1), 2);\n}\n";
const BOUNDED: &str = "#[test]\nfn adds() {\n    assert!(f(1) > 0);\n}\n";
const SPELLED_OUT: &str = "#[test]\nfn adds() {\n    assert!(f(1) == 2);\n}\n";
const TWO_ROWS: &str = "laws::table!(adds, f, [1 => 2, 2 => 3]);\n";
const ONE_ROW: &str = "laws::table!(adds, f, [1 => 2]);\n";
const THREE_ROWS: &str = "laws::table!(adds, f, [1 => 2, 2 => 3, 3 => 4]);\n";
const LOOPED: &str = "const ROWS: [(u8, u8); 3] = [(1, 2), (2, 3), (3, 4)];\n#[test]\nfn adds() {\n    for (input, want) in ROWS {\n        assert_eq!(f(input), want);\n    }\n}\n";
const LOOPED_SHORT: &str = "const ROWS: [(u8, u8); 2] = [(1, 2), (2, 3)];\n#[test]\nfn adds() {\n    for (input, want) in ROWS {\n        assert_eq!(f(input), want);\n    }\n}\n";
const ENUMERATED: &str = "#[test]\nfn every_mark_shows() {\n    for mark in Mark::ALL {\n        assert_eq!(mark.show(), mark.name());\n    }\n}\n";
const LISTED: &str = "#[test]\nfn every_mark_shows() {\n    for mark in [Mark::A, Mark::B] {\n        assert_eq!(mark.show(), mark.name());\n    }\n}\n";
const INLINE: &str = "#[test]\nfn adds() {\n    assert_eq!(f(1), 2);\n}\n";
const HELPED: &str = "fn expect(value: u8, want: u8) {\n    assert_eq!(f(value), want);\n}\n#[test]\nfn adds() {\n    expect(1, 2);\n}\n";
const RENAMED: &str = "#[test]\nfn sums() {\n    assert_eq!(f(1), 2);\n}\n";
const SPLIT: &str =
    "#[test]\nfn adds_one() {\n    assert_eq!(f(1), 2);\n}\n#[test]\nfn adds_two() {\n    assert_eq!(f(2), 3);\n}\n";
const JOINED: &str = "#[test]\nfn adds() {\n    assert_eq!(f(1), 2);\n    assert_eq!(f(2), 3);\n}\n";
const EMPTY: &str = "";
const ASSOCIATIVE: &str = "laws::associative!(joins, any::<A>(), A::join);\n";
const COMMUTATIVE: &str = "laws::commutative!(joins, any::<A>(), A::join);\n";
const ARRAY: &str = "#[test]\nfn lists() {\n    assert_eq!(all(), [1, 2, 3]);\n}\n";
const ARRAY_SHORT: &str = "#[test]\nfn lists() {\n    assert_eq!(all(), [1, 2]);\n}\n";
const DIFFERS: &str = "#[test]\nfn differs() {\n    assert_ne!(f(1), f(2));\n}\n";
const BARE: &str = "#[test]\nfn differs() {\n    assert!(f(1));\n}\n";
const TWICE: &str = "#[test]\nfn twice() {\n    assert!(f(1) > 0);\n    assert!(f(1) > 0);\n}\n";
const ONCE: &str = "#[test]\nfn twice() {\n    assert!(f(1) > 0);\n}\n";
const OK_PINNED: &str = "#[test]\nfn parses() {\n    assert_eq!(parse(1).is_ok(), true);\n}\n";
const OK_BARE: &str = "#[test]\nfn parses() {\n    assert!(parse(1).is_ok());\n}\n";
const POSITIVE_THEN_VALUE: &str = "#[test]\nfn mixed() {\n    assert!(f(9) > 0);\n    assert_eq!(f(1), 1);\n}\n";
const VALUE_THEN_POSITIVE: &str = "#[test]\nfn mixed() {\n    assert_eq!(f(1), 1);\n    assert!(f(9) > 0);\n}\n";
const ARRAY_MESSAGED: &str = "#[test]\nfn lists() {\n    assert_eq!(all(), [1], \"x\");\n}\n";
const SAME_SIDES: &str = "#[test]\nfn adds() {\n    assert_eq!(f(1), f(1), \"same\");\n}\n";
const LOOPED_FIRST: &str = "const ROWS: [(u8, u8); 3] = [(1, 2), (2, 3), (3, 4)];\n#[test]\nfn adds() {\n    for (input, want) in ROWS.iter().take(1) {\n        assert_eq!(f(*input), *want);\n    }\n}\n";
const LOOPED_ITER: &str = "const ROWS: [(u8, u8); 3] = [(1, 2), (2, 3), (3, 4)];\n#[test]\nfn adds() {\n    for (input, want) in ROWS.iter() {\n        assert_eq!(f(*input), *want);\n    }\n}\n";
const ROW_AS_RECEIVER: &str = "const ROWS: [(u8, u8); 3] = [(1, 2), (2, 3), (3, 4)];\n#[test]\nfn adds() {\n    assert_eq!(f(1), ROWS.iter().next().unwrap().1);\n}\n";
const UNRELATED_LEN: &str =
    "fn len(_: u8) {\n    assert_eq!(f(1), 2);\n}\n#[test]\nfn adds() {\n    let _ = vec![1].len();\n}\n";
const UNRELATED_METHOD: &str = "struct S;\nimpl S {\n    fn expect(&self) {\n        assert_eq!(f(1), 2);\n    }\n}\n#[test]\nfn adds() {\n    S.expect();\n}\n";
const ASSOCIATED: &str = "struct S;\nimpl S {\n    fn expect(value: u8, want: u8) {\n        assert_eq!(f(value), want);\n    }\n}\n#[test]\nfn adds() {\n    S::expect(1, 2);\n}\n";
const PROPTESTED: &str =
    "proptest! {\n    #[test]\n    fn holds(x in 0..9u8) {\n        prop_assert_eq!(f(x), x + 1);\n    }\n}\n";
const PROPTEST_EMPTIED: &str =
    "proptest! {\n    #[test]\n    fn holds(x in 0..9u8) {\n        let _ = f(x);\n    }\n}\n";

const MACRO_CALL: &str = "#[test]\nfn adds() {\n    assert_eq!(vec![f(1)], [2]);\n}\n";
const MACRO_OTHER_CALL: &str = "#[test]\nfn adds() {\n    assert_eq!(vec![g(1)], [2]);\n}\n";
const TAUTOLOGY_WIDE: &str = "#[test]\nfn adds() {\n    assert_eq!(f(1) + 5, f(1) + 5);\n}\n";
const AFTER_MATCH_PANICS: &str = "#[test]\nfn adds() {\n    match f(0) {\n        0 => panic!(\"x\"),\n        _ => panic!(\"y\"),\n    }\n    assert_eq!(f(1), 2);\n}\n";
const STRATEGY_PAREN: &str =
    "proptest! {\n    #[test]\n    fn holds(x in (0..9u8)) {\n        prop_assert_eq!(f(x), x + 1);\n    }\n}\n";
const OTHER_CALL: &str = "#[test]\nfn adds() {\n    assert_eq!(g(1), 2);\n}\n";
const BOTH_CALLS: &str = "#[test]\nfn adds() {\n    assert_eq!(g(f(1)), 2);\n}\n";
const METHOD_CALL: &str = "#[test]\nfn adds() {\n    assert_eq!(a.run(1), 2);\n}\n";
const OTHER_METHOD: &str = "#[test]\nfn adds() {\n    assert_eq!(a.walk(1), 2);\n}\n";
const TABLE_OTHER_FUNCTION: &str = "laws::table!(adds, g, [1 => 2, 2 => 3]);\n";
const QUALIFIED_CALL: &str = "#[test]\nfn adds() {\n    assert_eq!(other::f(1), 2);\n}\n";
const RANGE_WIDE: &str = "#[test]\nfn spans() {\n    for i in 0..10u8 {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_NARROW: &str = "#[test]\nfn spans() {\n    for i in 0..5u8 {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_SHIFTED: &str =
    "#[test]\nfn spans() {\n    for i in 5..15u8 {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_CLOSED: &str =
    "#[test]\nfn spans() {\n    for i in 0..=9u8 {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_NAMED: &str =
    "const TOP: u8 = 10;\n#[test]\nfn spans() {\n    for i in 0..TOP {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_NESTED: &str = "#[test]\nfn spans() {\n    for i in 0..4u8 {\n        for j in 0..4u8 {\n            assert_eq!(f(i, j), i + j);\n        }\n    }\n}\n";
const RANGE_NESTED_THIN: &str = "#[test]\nfn spans() {\n    for i in 0..4u8 {\n        for j in 0..2u8 {\n            assert_eq!(f(i, j), i + j);\n        }\n    }\n}\n";
const MEMBER_TABLE: &str = "struct S;\nimpl S {\n    const ROWS: [(u8, u8); 3] = [(1, 2), (2, 3), (3, 4)];\n}\n#[test]\nfn adds() {\n    for (input, want) in S::ROWS {\n        assert_eq!(f(input), want);\n    }\n}\n";
const MEMBER_TABLE_SHORT: &str = "struct S;\nimpl S {\n    const ROWS: [(u8, u8); 2] = [(1, 2), (2, 3)];\n}\n#[test]\nfn adds() {\n    for (input, want) in S::ROWS {\n        assert_eq!(f(input), want);\n    }\n}\n";
const MEMBER_TABLE_SELF: &str = "struct S;\nimpl S {\n    const ROWS: [(u8, u8); 3] = [(1, 2), (2, 3), (3, 4)];\n    fn run() {\n        for (input, want) in Self::ROWS {\n            assert_eq!(f(input), want);\n        }\n    }\n}\n#[test]\nfn adds() {\n    S::run();\n}\n";
const MEMBER_TABLE_SELF_SHORT: &str = "struct S;\nimpl S {\n    const ROWS: [(u8, u8); 2] = [(1, 2), (2, 3)];\n    fn run() {\n        for (input, want) in Self::ROWS {\n            assert_eq!(f(input), want);\n        }\n    }\n}\n#[test]\nfn adds() {\n    S::run();\n}\n";
const TWO_DISTINCT_ROWS: &str = "laws::table!(adds, f, [1 => 2, 2 => 3]);\n";
const TWO_DUPLICATE_ROWS: &str = "laws::table!(adds, f, [1 => 2, 1 => 2]);\n";
const TWO_ROWS_ONE_WANT: &str = "laws::table!(adds, f, [1 => 2, 2 => 2]);\n";
const LISTED_DISTINCT: &str =
    "#[test]\nfn adds() {\n    for input in [1, 2] {\n        assert_eq!(f(input), input + 1);\n    }\n}\n";
const LISTED_DUPLICATE: &str =
    "#[test]\nfn adds() {\n    for input in [1, 1] {\n        assert_eq!(f(input), input + 1);\n    }\n}\n";
const STRATEGY_WIDE: &str =
    "proptest! {\n    #[test]\n    fn holds(x in 0..9u8) {\n        prop_assert_eq!(f(x), x + 1);\n    }\n}\n";
const STRATEGY_NARROW: &str =
    "proptest! {\n    #[test]\n    fn holds(x in 0..5u8) {\n        prop_assert_eq!(f(x), x + 1);\n    }\n}\n";
const STRATEGY_SHIFTED: &str =
    "proptest! {\n    #[test]\n    fn holds(x in 1..10u8) {\n        prop_assert_eq!(f(x), x + 1);\n    }\n}\n";
const STRATEGY_CLOSED: &str =
    "proptest! {\n    #[test]\n    fn holds(x in 0..=8u8) {\n        prop_assert_eq!(f(x), x + 1);\n    }\n}\n";
const STRATEGY_ANY: &str =
    "proptest! {\n    #[test]\n    fn holds(x in any::<u8>()) {\n        prop_assert_eq!(f(x), x + 1);\n    }\n}\n";
const STRATEGY_TYPED: &str =
    "proptest! {\n    #[test]\n    fn holds(x: u8) {\n        prop_assert_eq!(f(x), x + 1);\n    }\n}\n";
const STRATEGY_PAIR: &str = "proptest! {\n    #[test]\n    fn holds(x in 0..9u8, y in 0..9u8) {\n        prop_assert_eq!(f(x, y), x + y);\n    }\n}\n";
const STRATEGY_PAIR_NARROW: &str = "proptest! {\n    #[test]\n    fn holds(x in 0..9u8, y in 0..3u8) {\n        prop_assert_eq!(f(x, y), x + y);\n    }\n}\n";
const LAW_ANY: &str = "laws::associative!(joins, any::<u8>(), A::join);\n";
const LAW_NARROWED: &str = "laws::associative!(joins, 0..9u8, A::join);\n";
const FOREIGN_HELPER: &str = "mod other {\n    pub fn expect(value: u8, want: u8) {\n        assert_eq!(f(value), want);\n    }\n}\nfn expect(_: u8, _: u8) {}\n#[test]\nfn adds() {\n    expect(1, 2);\n}\n";
const OWN_HELPER: &str = "mod other {\n    pub fn expect(_: u8, _: u8) {}\n}\nfn expect(value: u8, want: u8) {\n    assert_eq!(f(value), want);\n}\n#[test]\nfn adds() {\n    expect(1, 2);\n}\n";
const PATH_HELPER: &str = "mod other {\n    pub fn expect(value: u8, want: u8) {\n        assert_eq!(f(value), want);\n    }\n}\nfn expect(_: u8, _: u8) {}\n#[test]\nfn adds() {\n    other::expect(1, 2);\n}\n";
const IMPORTED_HELPER: &str = "mod other {\n    pub fn expect(value: u8, want: u8) {\n        assert_eq!(f(value), want);\n    }\n}\nuse other::expect;\n#[test]\nfn adds() {\n    expect(1, 2);\n}\n";
const GLOB_HELPER: &str = "mod other {\n    pub fn expect(value: u8, want: u8) {\n        assert_eq!(f(value), want);\n    }\n}\nuse other::*;\n#[test]\nfn adds() {\n    expect(1, 2);\n}\n";
const SELF_HELPER: &str = "fn expect(value: u8, want: u8) {\n    assert_eq!(f(value), want);\n}\n#[test]\nfn adds() {\n    self::expect(1, 2);\n}\n";
const SUPER_HELPER: &str = "fn expect(value: u8, want: u8) {\n    assert_eq!(f(value), want);\n}\nmod inner {\n    #[test]\n    fn adds() {\n        super::expect(1, 2);\n    }\n}\n";
const SUPER_HELPER_INLINE: &str = "mod inner {\n    #[test]\n    fn adds() {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const SIBLING_HELPER: &str = "fn expect(value: u8, want: u8) {\n    assert_eq!(f(value), want);\n}\nmod inner {\n    fn expect(_: u8, _: u8) {}\n    #[test]\n    fn adds() {\n        expect(1, 2);\n    }\n}\n";
const SELF_TYPE_HELPER: &str = "struct S;\nimpl S {\n    fn expect(value: u8, want: u8) {\n        assert_eq!(f(value), want);\n    }\n    fn run() {\n        Self::expect(1, 2);\n    }\n}\n#[test]\nfn adds() {\n    S::run();\n}\n";
const OTHER_TYPE_HELPER: &str = "struct S;\nstruct T;\nimpl T {\n    fn expect(value: u8, want: u8) {\n        assert_eq!(f(value), want);\n    }\n}\nimpl S {\n    fn expect(_: u8, _: u8) {}\n}\n#[test]\nfn adds() {\n    S::expect(1, 2);\n}\n";
const ONE_VALUE: &str = "#[test]\nfn adds() {\n    assert_eq!(f(), 2);\n}\n";
const TAUTOLOGY: &str = "#[test]\nfn adds() {\n    assert_eq!(7, 7);\n}\n";
const TAUTOLOGY_BOUND: &str = "#[test]\nfn adds() {\n    let want = 2;\n    assert_eq!(want, 2);\n}\n";
const ONE_VALUE_SPELLED: &str = "#[test]\nfn adds() {\n    assert!(f() == 2);\n}\n";
const TAUTOLOGY_SPELLED: &str = "#[test]\nfn adds() {\n    assert!(7 == 7);\n}\n";
const TAUTOLOGY_BOOLEANS: &str = "#[test]\nfn adds() {\n    assert_eq!(true, true);\n}\n";
const TAUTOLOGY_DIFFERS: &str = "#[test]\nfn adds() {\n    assert_ne!(f(), 2);\n}\n";
const TAUTOLOGY_TWICE: &str = "#[test]\nfn adds() {\n    assert_ne!(7, 7);\n}\n";
const AFTER_RETURN: &str = "#[test]\nfn adds() {\n    return;\n    assert_eq!(f(1), 2);\n}\n";
const AFTER_PANIC: &str = "#[test]\nfn adds() {\n    panic!(\"stop\");\n    assert_eq!(f(1), 2);\n}\n";
const AFTER_SURE_RETURN: &str =
    "#[test]\nfn adds() {\n    if true {\n        return;\n    }\n    assert_eq!(f(1), 2);\n}\n";
const AFTER_MAYBE_RETURN: &str =
    "#[test]\nfn adds() {\n    if f(0) > 1 {\n        return;\n    }\n    assert_eq!(f(1), 2);\n}\n";
const IN_FALSE_BRANCH: &str = "#[test]\nfn adds() {\n    if false {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IN_ARITHMETIC_FALSE_BRANCH: &str =
    "const LIMIT: u8 = 3;\n#[test]\nfn adds() {\n    if LIMIT > 4 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IN_DEAD_ELSE: &str = "#[test]\nfn adds() {\n    if 1 < 2 {\n        let _ = f(1);\n    } else {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IN_LIVE_BRANCH: &str = "#[test]\nfn adds() {\n    if f(0) < 2 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IN_LIVE_ELSE: &str = "#[test]\nfn adds() {\n    if true {\n        assert_eq!(f(1), 2);\n    } else {\n        let _ = f(1);\n    }\n}\n";
const IN_EMPTY_LOOP: &str = "#[test]\nfn adds() {\n    for _ in 0..0u8 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const AFTER_BREAK: &str =
    "#[test]\nfn adds() {\n    for _ in [1, 2] {\n        break;\n        assert_eq!(f(1), 2);\n    }\n}\n";

const RANGE_SUMMED: &str =
    "#[test]\nfn spans() {\n    for i in 0..(2 + 8u8) {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_DIFFERENCE: &str =
    "#[test]\nfn spans() {\n    for i in 0..(12 - 2u8) {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_PRODUCT: &str =
    "#[test]\nfn spans() {\n    for i in 0..(2 * 5u8) {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_QUOTIENT: &str =
    "#[test]\nfn spans() {\n    for i in 0..(20 / 2u8) {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_REMAINDER: &str =
    "#[test]\nfn spans() {\n    for i in 0..(10 % 11u8) {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_CAST: &str =
    "#[test]\nfn spans() {\n    for i in 0..(10 as u8) {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_PAREN: &str =
    "#[test]\nfn spans() {\n    for i in (0..10u8) {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_LOCAL: &str =
    "#[test]\nfn spans() {\n    let top = 10u8;\n    for i in 0..top {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_TWICE: &str = "const HALF: u8 = 5;\n#[test]\nfn spans() {\n    for i in 0..HALF + HALF {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_OPEN: &str = "#[test]\nfn spans() {\n    for i in ..10u8 {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_NEGATIVE: &str =
    "#[test]\nfn spans() {\n    for i in -5..5i32 {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_NATURAL: &str =
    "#[test]\nfn spans() {\n    for i in 0..10i32 {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_SINGLE_FIVE: &str =
    "#[test]\nfn spans() {\n    for i in 5..=5u8 {\n        assert_eq!(f(i), i);\n    }\n}\n";
const RANGE_SINGLE_SEVEN: &str =
    "#[test]\nfn spans() {\n    for i in 7..=7u8 {\n        assert_eq!(f(i), i);\n    }\n}\n";
const RANGE_UNKNOWN: &str =
    "#[test]\nfn spans() {\n    for i in 0..count() {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const RANGE_CYCLE: &str =
    "#[test]\nfn spans() {\n    let top = top;\n    for i in 0..top {\n        assert_eq!(f(i), i + 1);\n    }\n}\n";
const REFERENCED: &str = "const ROWS: [(u8, u8); 3] = [(1, 2), (2, 3), (3, 4)];\n#[test]\nfn adds() {\n    for (input, want) in &ROWS {\n        assert_eq!(f(*input), *want);\n    }\n}\n";
const PARENTHESISED: &str = "const ROWS: [(u8, u8); 3] = [(1, 2), (2, 3), (3, 4)];\n#[test]\nfn adds() {\n    for (input, want) in (ROWS) {\n        assert_eq!(f(input), want);\n    }\n}\n";
const MACRO_LIST: &str =
    "#[test]\nfn adds() {\n    for input in vec![1, 2, 3] {\n        assert_eq!(f(input), input + 1);\n    }\n}\n";
const MACRO_LIST_SHORT: &str =
    "#[test]\nfn adds() {\n    for input in vec![1, 2] {\n        assert_eq!(f(input), input + 1);\n    }\n}\n";
const PLAIN_LOOP: &str =
    "#[test]\nfn adds() {\n    for input in inputs() {\n        assert_eq!(f(input), input + 1);\n    }\n}\n";
const ENUMERATED_LOWER: &str =
    "#[test]\nfn adds() {\n    for input in Mark::all {\n        assert_eq!(f(input), input + 1);\n    }\n}\n";
const ENUMERATED_UPPER_OWNER: &str =
    "#[test]\nfn adds() {\n    for input in MARK::ALL {\n        assert_eq!(f(input), input + 1);\n    }\n}\n";
const TWO_TABLES: &str = "laws::table!(adds, f, [1 => 2, 2 => 3]);\nlaws::table!(subs, g, [3 => 2, 4 => 3]);\n";
const TWO_TABLES_FIRST_SHORT: &str = "laws::table!(adds, f, [1 => 2]);\nlaws::table!(subs, g, [3 => 2, 4 => 3]);\n";
const LAW_WITH_BLOCK: &str = "laws::law!(holds, any::<u8>(), f, {\n    assert_eq!(f(1), 2);\n});\n";
const LAW_WITHOUT_BLOCK: &str = "laws::law!(holds, any::<u8>(), f);\n";
const WRAPPED: &str = "#[test]\nfn adds() {\n    fixture!(assert_eq!(f(1), 2));\n}\n";
const PATTERNED: &str = "#[test]\nfn shapes() {\n    assert!(matches!(f(1), Some(_)));\n}\n";
const BARE_CONDITION: &str = "#[test]\nfn shapes() {\n    assert!(f(1));\n}\n";
const SPELLED_DIFFERS: &str = "#[test]\nfn differs() {\n    assert!(f(1) != f(2));\n}\n";
const SPELLED_ORDERED: &str = "#[test]\nfn differs() {\n    assert!(f(1) > f(2));\n}\n";
const NAMED_VALUE: &str = "#[test]\nfn names() {\n    assert_eq!(f(), Mark::Open);\n}\n";
const UNNAMED_VALUE: &str = "#[test]\nfn names() {\n    assert_eq!(f(), other);\n}\n";
const MACRO_VALUES: &str = "#[test]\nfn lists() {\n    assert_eq!(f(), vec![1, 2, 3]);\n}\n";
const MACRO_VALUE_SHORT: &str = "#[test]\nfn lists() {\n    assert_eq!(f(), vec![1]);\n}\n";
const BOUND_WANT: &str = "#[test]\nfn adds() {\n    let want = 2;\n    assert_eq!(f(1), want);\n}\n";
const CONSTANT_SIDE: &str = "const WANT: [u8; 3] = [1, 2, 3];\n#[test]\nfn lists() {\n    assert_eq!(f(), WANT);\n}\n";
const CONSTANT_REFERENCE: &str =
    "const WANT: [u8; 3] = [1, 2, 3];\n#[test]\nfn lists() {\n    assert_eq!(f(), &WANT);\n}\n";
const CONSTANT_PARENTHESISED: &str =
    "const WANT: [u8; 3] = [1, 2, 3];\n#[test]\nfn lists() {\n    assert_eq!(f(), (WANT));\n}\n";
const ONE_ELEMENT: &str = "#[test]\nfn lists() {\n    assert_eq!(f(), [1]);\n}\n";
const ONE_ELEMENT_REFERENCE: &str = "#[test]\nfn lists() {\n    assert_eq!(f(), &[1]);\n}\n";
const ONE_ELEMENT_PARENTHESISED: &str = "#[test]\nfn lists() {\n    assert_eq!(f(), ([1]));\n}\n";
const CRATE_HELPER: &str = "fn expect(value: u8, want: u8) {\n    assert_eq!(f(value), want);\n}\n#[test]\nfn adds() {\n    crate::tests::expect(1, 2);\n}\n";
const SUPER_SUPER_HELPER: &str = "fn expect(value: u8, want: u8) {\n    assert_eq!(f(value), want);\n}\nmod outer {\n    mod inner {\n        #[test]\n        fn adds() {\n            super::super::expect(1, 2);\n        }\n    }\n}\n";
const SUPER_SUPER_INLINE: &str = "mod outer {\n    mod inner {\n        #[test]\n        fn adds() {\n            assert_eq!(f(1), 2);\n        }\n    }\n}\n";
const RENAMED_IMPORT: &str = "mod other {\n    pub fn expect(value: u8, want: u8) {\n        assert_eq!(f(value), want);\n    }\n}\nuse other::expect as check;\n#[test]\nfn adds() {\n    check(1, 2);\n}\n";
const GROUPED_IMPORT: &str = "mod other {\n    pub fn expect(value: u8, want: u8) {\n        assert_eq!(f(value), want);\n    }\n    pub fn spare() {}\n}\nuse other::{spare, expect};\n#[test]\nfn adds() {\n    expect(1, 2);\n}\n";
const INNER_GLOB: &str = "fn expect(value: u8, want: u8) {\n    assert_eq!(f(value), want);\n}\nmod inner {\n    use super::*;\n    #[test]\n    fn adds() {\n        expect(1, 2);\n    }\n}\n";
const INNER_GLOB_INLINE: &str = "mod inner {\n    #[test]\n    fn adds() {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const RECURSIVE_HELPER: &str = "fn expect(value: u8, want: u8) {\n    expect(value, want);\n    assert_eq!(f(value), want);\n}\n#[test]\nfn adds() {\n    expect(1, 2);\n}\n";
const IN_DEAD_WHILE: &str = "#[test]\nfn adds() {\n    while false {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IN_LIVE_WHILE: &str = "#[test]\nfn adds() {\n    while f(0) < 3 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IN_LOOP_UNKNOWN: &str = "#[test]\nfn adds() {\n    while true {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const AFTER_CONTINUE: &str =
    "#[test]\nfn adds() {\n    for _ in [1, 2] {\n        continue;\n        assert_eq!(f(1), 2);\n    }\n}\n";
const AFTER_BLOCK_RETURN: &str = "#[test]\nfn adds() {\n    {\n        return;\n    }\n    assert_eq!(f(1), 2);\n}\n";
const AFTER_UNSAFE_RETURN: &str =
    "#[test]\nfn adds() {\n    unsafe {\n        return;\n    }\n    assert_eq!(f(1), 2);\n}\n";
const AFTER_PAREN_RETURN: &str = "#[test]\nfn adds() {\n    (return);\n    assert_eq!(f(1), 2);\n}\n";
const AFTER_MATCH_RETURNS: &str = "#[test]\nfn adds() {\n    match f(0) {\n        0 => return,\n        _ => return,\n    }\n    assert_eq!(f(1), 2);\n}\n";
const AFTER_MATCH_PARTLY: &str = "#[test]\nfn adds() {\n    match f(0) {\n        0 => return,\n        _ => {}\n    }\n    assert_eq!(f(1), 2);\n}\n";
const AFTER_BOTH_RETURN: &str = "#[test]\nfn adds() {\n    if f(0) > 1 {\n        return;\n    } else {\n        return;\n    }\n    assert_eq!(f(1), 2);\n}\n";
const AFTER_ELSE_RETURN: &str = "#[test]\nfn adds() {\n    if f(0) > 1 {\n        let _ = f(2);\n    } else {\n        return;\n    }\n    assert_eq!(f(1), 2);\n}\n";
const AFTER_DEAD_RETURN: &str =
    "#[test]\nfn adds() {\n    if false {\n        return;\n    }\n    assert_eq!(f(1), 2);\n}\n";
const AFTER_DEAD_THEN_ELSE_RETURN: &str = "#[test]\nfn adds() {\n    if 2 < 1 {\n        let _ = f(2);\n    } else {\n        return;\n    }\n    assert_eq!(f(1), 2);\n}\n";
const IF_EQUAL: &str = "#[test]\nfn adds() {\n    if 1 == 2 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_UNEQUAL: &str = "#[test]\nfn adds() {\n    if 1 != 1 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_LESS: &str = "#[test]\nfn adds() {\n    if 2 < 1 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_AT_MOST: &str = "#[test]\nfn adds() {\n    if 2 <= 1 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_GREATER: &str = "#[test]\nfn adds() {\n    if 1 > 2 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_AT_LEAST: &str = "#[test]\nfn adds() {\n    if 1 >= 2 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_NEGATED: &str = "#[test]\nfn adds() {\n    if !true {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_NEGATED_LIVE: &str = "#[test]\nfn adds() {\n    if !false {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_PARENTHESISED: &str = "#[test]\nfn adds() {\n    if (false) {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_NAMED_FLAG: &str =
    "const ENABLED: bool = false;\n#[test]\nfn adds() {\n    if ENABLED {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_NAMED_NUMBER: &str =
    "#[test]\nfn adds() {\n    let limit = 3;\n    if limit > 4 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_LIVE_EQUAL: &str = "#[test]\nfn adds() {\n    if 2 == 2 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_LIVE_LESS_EQUAL: &str = "#[test]\nfn adds() {\n    if 2 <= 2 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_LIVE_GREATER_EQUAL: &str = "#[test]\nfn adds() {\n    if 3 >= 2 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_LIVE_UNEQUAL: &str = "#[test]\nfn adds() {\n    if 1 != 2 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_LIVE_GREATER: &str = "#[test]\nfn adds() {\n    if 3 > 2 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_LIVE_LESS: &str = "#[test]\nfn adds() {\n    if 1 < 2 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_UNKNOWN_ORDER: &str = "#[test]\nfn adds() {\n    if 1 & 2 > 0 {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const IF_OVERFLOWING: &str = "#[test]\nfn adds() {\n    if 170141183460469231731687303715884105727 + 1 > 0 {\n        assert_eq!(f(1), 2);\n    }\n}\n";

laws::table!(a_change_is_flagged_exactly_when_a_test_loses_strength, Weakened::in_one_module, [
    [EQUALS, EQUALS] => false,
    [EQUALS, BOUNDED] => true,
    [EQUALS, SPELLED_OUT] => false,
    [SPELLED_OUT, EQUALS] => false,
    [TWO_ROWS, ONE_ROW] => true,
    [ONE_ROW, TWO_ROWS] => false,
    [TWO_ROWS, THREE_ROWS] => false,
    [LOOPED, LOOPED_SHORT] => true,
    [ENUMERATED, LISTED] => true,
    [INLINE, HELPED] => false,
    [EQUALS, RENAMED] => false,
    [EQUALS, EMPTY] => true,
    [JOINED, SPLIT] => false,
    [ASSOCIATIVE, COMMUTATIVE] => true,
    [ARRAY, ARRAY_SHORT] => true,
    [DIFFERS, BARE] => true,
    [TWICE, ONCE] => true,
    [OK_PINNED, OK_BARE] => false,
    [POSITIVE_THEN_VALUE, VALUE_THEN_POSITIVE] => false,
    [ARRAY_SHORT, ARRAY_MESSAGED] => true,
    [EQUALS, SAME_SIDES] => true,
    [LOOPED, LOOPED_FIRST] => true,
    [LOOPED, LOOPED_ITER] => false,
    [LOOPED, ROW_AS_RECEIVER] => true,
    [EQUALS, UNRELATED_LEN] => true,
    [EQUALS, UNRELATED_METHOD] => true,
    [INLINE, ASSOCIATED] => false,
    [PROPTESTED, PROPTESTED] => false,
    [PROPTESTED, PROPTEST_EMPTIED] => true,
    [MACRO_CALL, MACRO_OTHER_CALL] => true,
    [EQUALS, TAUTOLOGY_WIDE] => true,
    [EQUALS, AFTER_MATCH_PANICS] => true,
    [STRATEGY_WIDE, STRATEGY_PAREN] => false,
    [STRATEGY_PAREN, STRATEGY_NARROW] => true,
    [EQUALS, OTHER_CALL] => true,
    [OTHER_CALL, EQUALS] => true,
    [EQUALS, BOTH_CALLS] => false,
    [BOTH_CALLS, EQUALS] => true,
    [METHOD_CALL, OTHER_METHOD] => true,
    [METHOD_CALL, METHOD_CALL] => false,
    [TWO_ROWS, TABLE_OTHER_FUNCTION] => true,
    [EQUALS, QUALIFIED_CALL] => true,
    [RANGE_WIDE, RANGE_NARROW] => true,
    [RANGE_WIDE, RANGE_SHIFTED] => true,
    [RANGE_WIDE, RANGE_CLOSED] => false,
    [RANGE_WIDE, RANGE_NAMED] => false,
    [RANGE_NARROW, RANGE_WIDE] => false,
    [RANGE_NESTED, RANGE_NESTED_THIN] => true,
    [MEMBER_TABLE, MEMBER_TABLE_SHORT] => true,
    [MEMBER_TABLE, MEMBER_TABLE] => false,
    [MEMBER_TABLE_SHORT, MEMBER_TABLE] => false,
    [MEMBER_TABLE_SELF, MEMBER_TABLE_SELF_SHORT] => true,
    [TWO_DISTINCT_ROWS, TWO_DUPLICATE_ROWS] => true,
    [TWO_ROWS_ONE_WANT, TWO_DUPLICATE_ROWS] => true,
    [TWO_DUPLICATE_ROWS, TWO_DISTINCT_ROWS] => false,
    [LISTED_DISTINCT, LISTED_DUPLICATE] => true,
    [STRATEGY_WIDE, STRATEGY_NARROW] => true,
    [STRATEGY_WIDE, STRATEGY_SHIFTED] => true,
    [STRATEGY_WIDE, STRATEGY_CLOSED] => false,
    [STRATEGY_NARROW, STRATEGY_WIDE] => false,
    [STRATEGY_ANY, STRATEGY_WIDE] => true,
    [STRATEGY_WIDE, STRATEGY_ANY] => false,
    [STRATEGY_TYPED, STRATEGY_ANY] => false,
    [STRATEGY_ANY, STRATEGY_TYPED] => false,
    [STRATEGY_TYPED, STRATEGY_WIDE] => true,
    [STRATEGY_PAIR, STRATEGY_PAIR_NARROW] => true,
    [LAW_ANY, LAW_NARROWED] => true,
    [LAW_NARROWED, LAW_ANY] => false,
    [EQUALS, FOREIGN_HELPER] => true,
    [EQUALS, OWN_HELPER] => false,
    [EQUALS, PATH_HELPER] => false,
    [EQUALS, IMPORTED_HELPER] => false,
    [EQUALS, GLOB_HELPER] => false,
    [EQUALS, SELF_HELPER] => false,
    [SUPER_HELPER_INLINE, SUPER_HELPER] => false,
    [SUPER_HELPER_INLINE, SIBLING_HELPER] => true,
    [EQUALS, SELF_TYPE_HELPER] => false,
    [EQUALS, OTHER_TYPE_HELPER] => true,
    [ONE_VALUE, TAUTOLOGY] => true,
    [ONE_VALUE, TAUTOLOGY_BOUND] => true,
    [ONE_VALUE_SPELLED, TAUTOLOGY_SPELLED] => true,
    [TAUTOLOGY_DIFFERS, TAUTOLOGY_TWICE] => true,
    [TAUTOLOGY_BOOLEANS, TAUTOLOGY] => false,
    [TAUTOLOGY, ONE_VALUE] => false,
    [EQUALS, AFTER_RETURN] => true,
    [EQUALS, AFTER_PANIC] => true,
    [EQUALS, AFTER_SURE_RETURN] => true,
    [EQUALS, AFTER_MAYBE_RETURN] => false,
    [EQUALS, IN_FALSE_BRANCH] => true,
    [EQUALS, IN_ARITHMETIC_FALSE_BRANCH] => true,
    [EQUALS, IN_DEAD_ELSE] => true,
    [EQUALS, IN_LIVE_BRANCH] => false,
    [EQUALS, IN_LIVE_ELSE] => false,
    [EQUALS, IN_EMPTY_LOOP] => true,
    [EQUALS, AFTER_BREAK] => true,
    [RANGE_WIDE, RANGE_SUMMED] => false,
    [RANGE_WIDE, RANGE_DIFFERENCE] => false,
    [RANGE_WIDE, RANGE_PRODUCT] => false,
    [RANGE_WIDE, RANGE_QUOTIENT] => false,
    [RANGE_WIDE, RANGE_REMAINDER] => false,
    [RANGE_WIDE, RANGE_CAST] => false,
    [RANGE_WIDE, RANGE_PAREN] => false,
    [RANGE_WIDE, RANGE_LOCAL] => false,
    [RANGE_WIDE, RANGE_TWICE] => false,
    [RANGE_WIDE, RANGE_OPEN] => false,
    [RANGE_WIDE, RANGE_UNKNOWN] => true,
    [RANGE_WIDE, RANGE_CYCLE] => true,
    [RANGE_NEGATIVE, RANGE_NATURAL] => true,
    [RANGE_NATURAL, RANGE_NEGATIVE] => true,
    [RANGE_SINGLE_FIVE, RANGE_SINGLE_SEVEN] => true,
    [RANGE_SINGLE_FIVE, RANGE_SINGLE_FIVE] => false,
    [IN_EMPTY_LOOP, EMPTY] => false,
    [LOOPED, REFERENCED] => false,
    [LOOPED_SHORT, REFERENCED] => false,
    [REFERENCED, LOOPED_SHORT] => true,
    [LOOPED, PARENTHESISED] => false,
    [PARENTHESISED, LOOPED_SHORT] => true,
    [MACRO_LIST, MACRO_LIST_SHORT] => true,
    [MACRO_LIST_SHORT, MACRO_LIST] => false,
    [LISTED_DISTINCT, PLAIN_LOOP] => true,
    [ENUMERATED, ENUMERATED_LOWER] => true,
    [ENUMERATED, ENUMERATED_UPPER_OWNER] => true,
    [TWO_TABLES, TWO_TABLES_FIRST_SHORT] => true,
    [TWO_TABLES_FIRST_SHORT, TWO_TABLES] => false,
    [LAW_WITH_BLOCK, LAW_WITHOUT_BLOCK] => true,
    [LAW_WITHOUT_BLOCK, LAW_WITH_BLOCK] => false,
    [EQUALS, WRAPPED] => false,
    [WRAPPED, EMPTY] => true,
    [PATTERNED, BARE_CONDITION] => true,
    [BARE_CONDITION, PATTERNED] => false,
    [SPELLED_DIFFERS, SPELLED_ORDERED] => true,
    [SPELLED_ORDERED, SPELLED_DIFFERS] => false,
    [NAMED_VALUE, UNNAMED_VALUE] => true,
    [UNNAMED_VALUE, NAMED_VALUE] => false,
    [MACRO_VALUES, MACRO_VALUE_SHORT] => true,
    [MACRO_VALUE_SHORT, MACRO_VALUES] => false,
    [EQUALS, BOUND_WANT] => false,
    [CONSTANT_SIDE, ONE_ELEMENT] => true,
    [CONSTANT_REFERENCE, ONE_ELEMENT_REFERENCE] => true,
    [CONSTANT_PARENTHESISED, ONE_ELEMENT_PARENTHESISED] => true,
    [ONE_ELEMENT, CONSTANT_SIDE] => false,
    [EQUALS, CRATE_HELPER] => false,
    [SUPER_SUPER_INLINE, SUPER_SUPER_HELPER] => false,
    [EQUALS, RENAMED_IMPORT] => false,
    [EQUALS, GROUPED_IMPORT] => false,
    [INNER_GLOB_INLINE, INNER_GLOB] => false,
    [EQUALS, RECURSIVE_HELPER] => false,
    [EQUALS, IN_DEAD_WHILE] => true,
    [EQUALS, IN_LIVE_WHILE] => false,
    [EQUALS, IN_LOOP_UNKNOWN] => false,
    [EQUALS, AFTER_CONTINUE] => true,
    [EQUALS, AFTER_BLOCK_RETURN] => true,
    [EQUALS, AFTER_UNSAFE_RETURN] => true,
    [EQUALS, AFTER_PAREN_RETURN] => true,
    [EQUALS, AFTER_MATCH_RETURNS] => true,
    [EQUALS, AFTER_MATCH_PARTLY] => false,
    [EQUALS, AFTER_BOTH_RETURN] => true,
    [EQUALS, AFTER_ELSE_RETURN] => false,
    [EQUALS, AFTER_DEAD_RETURN] => false,
    [EQUALS, AFTER_DEAD_THEN_ELSE_RETURN] => true,
    [EQUALS, IF_EQUAL] => true,
    [EQUALS, IF_UNEQUAL] => true,
    [EQUALS, IF_LESS] => true,
    [EQUALS, IF_AT_MOST] => true,
    [EQUALS, IF_GREATER] => true,
    [EQUALS, IF_AT_LEAST] => true,
    [EQUALS, IF_NEGATED] => true,
    [EQUALS, IF_NEGATED_LIVE] => false,
    [EQUALS, IF_PARENTHESISED] => true,
    [EQUALS, IF_NAMED_FLAG] => true,
    [EQUALS, IF_NAMED_NUMBER] => true,
    [EQUALS, IF_LIVE_EQUAL] => false,
    [EQUALS, IF_LIVE_LESS_EQUAL] => false,
    [EQUALS, IF_LIVE_GREATER_EQUAL] => false,
    [EQUALS, IF_LIVE_UNEQUAL] => false,
    [EQUALS, IF_LIVE_GREATER] => false,
    [EQUALS, IF_LIVE_LESS] => false,
    [EQUALS, IF_UNKNOWN_ORDER] => false,
    [EQUALS, IF_OVERFLOWING] => false,
]);

const ROUNDTRIP: &str = "laws::roundtrip!(rt, any::<u8>(), enc, dec);\n";
const DECLARED: &str = "laws::law!(holds, any::<u8>(), f);\n";

laws::table!(a_lost_check_is_described_by_its_test_relation_values_and_domain, Weakened::mentions, [
    [EQUALS, EMPTY, "adds () in tests of x lost a check: an equality pinning 2 values over listed domain of 1 cases (1 distinct, 0 values) calling f"] => true,
    [DIFFERS, EMPTY, "differs () in tests of x lost a check: an inequality pinning 2 values over listed domain of 1 cases (1 distinct, 0 values)"] => true,
    [BOUNDED, EMPTY, "adds () in tests of x lost a check: a condition pinning 2 values over listed domain of 1 cases (1 distinct, 0 values)"] => true,
    [PATTERNED, EMPTY, "shapes () in tests of x lost a check: a pattern match pinning 2 values over listed domain of 1 cases (1 distinct, 0 values)"] => true,
    [ASSOCIATIVE, EMPTY, "joins () in tests of x lost a check: the associative law pinning 0 values over generated domain of 1 cases (1 distinct, 0 values)"] => true,
    [COMMUTATIVE, EMPTY, "joins () in tests of x lost a check: the commutative law pinning 0 values over generated domain of 1 cases (1 distinct, 0 values)"] => true,
    [ROUNDTRIP, EMPTY, "rt () in tests of x lost a check: the roundtrip law pinning 0 values over generated domain of 256 cases (256 distinct, 0 values) over 0..=255"] => true,
    [DECLARED, EMPTY, "holds () in tests of x lost a check: the declared law pinning 0 values over generated domain of 256 cases (256 distinct, 0 values) over 0..=255"] => true,
    [LAW_ANY, EMPTY, "joins () in tests of x lost a check: the associative law pinning 0 values over generated domain of 256 cases (256 distinct, 0 values) over 0..=255"] => true,
    [ENUMERATED, EMPTY, "every_mark_shows () in tests of x lost a check: an equality pinning 0 values over enumerated domain of 1 cases (1 distinct, 0 values)"] => true,
    [LOOPED, EMPTY, "adds () in tests of x lost a check: an equality pinning 0 values over listed domain of 3 cases (3 distinct, 6 values)"] => true,
    [RANGE_WIDE, EMPTY, "spans () in tests of x lost a check: an equality pinning 1 values over listed domain of 10 cases (10 distinct, 0 values) over 0..=9"] => true,
    [SUPER_SUPER_INLINE, EMPTY, "adds () in tests in outer in inner of x lost a check: an equality pinning 2 values over listed domain of 1 cases (1 distinct, 0 values)"] => true,
]);

#[test]
fn a_test_moved_to_another_package_is_flagged() {
    assert!(Weakened::between(
        &[Weakened::placed("crates/x/src/lib.rs", EQUALS)],
        &[Weakened::placed("crates/y/src/lib.rs", EQUALS)],
    ));
}

#[test]
fn a_test_moved_to_another_file_of_its_package_is_not_flagged() {
    assert_eq!(
        Weakened::lost(
            &[Weakened::placed("crates/x/src/a.rs", EQUALS)],
            &[Weakened::placed("crates/x/src/b.rs", EQUALS)],
        ),
        0
    );
}

#[test]
fn an_assertion_moved_into_a_helper_in_another_file_is_flagged() {
    assert!(Weakened::between(
        &[Weakened::placed("crates/x/src/lib.rs", EQUALS)],
        &[
            Weakened::placed("crates/x/src/lib.rs", "#[test]\nfn adds() {\n    expect(1, 2);\n}\n"),
            Weakened::at(
                "crates/x/src/support.rs",
                "pub fn expect(value: u8, want: u8) {\n    assert_eq!(f(value), want);\n}\n"
            ),
        ],
    ));
}

#[test]
fn the_fix_names_the_spec_detail_that_exempts_a_narrowing() {
    assert!(
        Weakened::reported([EQUALS, EMPTY])
            .contains("owner-approved spec detail that names the test as <package>::<test>")
    );
}
