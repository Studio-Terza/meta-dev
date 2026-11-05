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
        Self::report(before, after).matches("lost a claim").count()
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
    [TWICE, ONCE] => false,
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
    [EQUALS, BOTH_CALLS] => true,
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
    [EQUALS, AFTER_MAYBE_RETURN] => true,
    [EQUALS, IN_FALSE_BRANCH] => true,
    [EQUALS, IN_ARITHMETIC_FALSE_BRANCH] => true,
    [EQUALS, IN_DEAD_ELSE] => true,
    [EQUALS, IN_LIVE_BRANCH] => true,
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
    [BARE_CONDITION, PATTERNED] => true,
    [SPELLED_DIFFERS, SPELLED_ORDERED] => true,
    [SPELLED_ORDERED, SPELLED_DIFFERS] => true,
    [NAMED_VALUE, UNNAMED_VALUE] => true,
    [UNNAMED_VALUE, NAMED_VALUE] => true,
    [MACRO_VALUES, MACRO_VALUE_SHORT] => true,
    [MACRO_VALUE_SHORT, MACRO_VALUES] => true,
    [EQUALS, BOUND_WANT] => false,
    [CONSTANT_SIDE, ONE_ELEMENT] => true,
    [CONSTANT_REFERENCE, ONE_ELEMENT_REFERENCE] => true,
    [CONSTANT_PARENTHESISED, ONE_ELEMENT_PARENTHESISED] => true,
    [ONE_ELEMENT, CONSTANT_SIDE] => true,
    [EQUALS, CRATE_HELPER] => false,
    [SUPER_SUPER_INLINE, SUPER_SUPER_HELPER] => false,
    [EQUALS, RENAMED_IMPORT] => false,
    [EQUALS, GROUPED_IMPORT] => false,
    [INNER_GLOB_INLINE, INNER_GLOB] => false,
    [EQUALS, RECURSIVE_HELPER] => false,
    [EQUALS, IN_DEAD_WHILE] => true,
    [EQUALS, IN_LIVE_WHILE] => true,
    [EQUALS, IN_LOOP_UNKNOWN] => false,
    [EQUALS, AFTER_CONTINUE] => true,
    [EQUALS, AFTER_BLOCK_RETURN] => true,
    [EQUALS, AFTER_UNSAFE_RETURN] => true,
    [EQUALS, AFTER_PAREN_RETURN] => true,
    [EQUALS, AFTER_MATCH_RETURNS] => true,
    [EQUALS, AFTER_MATCH_PARTLY] => true,
    [EQUALS, AFTER_BOTH_RETURN] => true,
    [EQUALS, AFTER_ELSE_RETURN] => true,
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
    [EQUALS, IF_UNKNOWN_ORDER] => true,
    [EQUALS, IF_OVERFLOWING] => true,
]);

const ROUNDTRIP: &str = "laws::roundtrip!(rt, any::<u8>(), enc, dec);\n";
const DECLARED: &str = "laws::law!(holds, any::<u8>(), f);\n";

laws::table!(a_lost_claim_is_described_by_its_test_subject_values_and_domain, Weakened::mentions, [
    [EQUALS, EMPTY, "test adds () in tests of x lost a claim: f(1) equals 2;"] => true,
    [DIFFERS, EMPTY, "test differs () in tests of x lost a claim: f(1) differs from f(2);"] => true,
    [BOUNDED, EMPTY, "test adds () in tests of x lost a claim: f(1) lies in 1..="] => true,
    [PATTERNED, EMPTY, "test shapes () in tests of x lost a claim: f(1) matches Some(is anything);"] => true,
    [ASSOCIATIVE, EMPTY, "lost a claim: the associative law of (A::join(), x0) equals true, for x0 256 sampled values of any()"] => true,
    [COMMUTATIVE, EMPTY, "lost a claim: the commutative law of (A::join(), x0) equals true"] => true,
    [ROUNDTRIP, EMPTY, "test rt () in tests of x lost a claim: the roundtrip law of (enc(), dec(), x0) equals true, for x0 256 sampled values of 0..=255"] => true,
    [DECLARED, EMPTY, "test holds () in tests of x lost a claim: the declared law of (f(), x0) equals true"] => true,
    [LAW_ANY, EMPTY, "for x0 256 sampled values of 0..=255"] => true,
    [ENUMERATED, EMPTY, "for x0 every value of Mark :: ALL"] => true,
    [LOOPED, EMPTY, "lost a claim: f(3) equals 4;"] => true,
    [RANGE_WIDE, EMPTY, "test spans () in tests of x lost a claim: f(x0) equals Add(x0, 1), for x0 every value of 0..=9;"] => true,
    [SUPER_SUPER_INLINE, EMPTY, "test adds () in tests in outer in inner of x lost a claim: f(1) equals 2;"] => true,
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
fn the_fix_says_how_to_keep_the_claim() {
    assert!(Weakened::reported([EQUALS, EMPTY]).contains("restore the claim, or state one that entails it"));
}

const DUPLICATED_BEFORE: &str = "#[test]\nfn adds() {\n    assert_eq!(f(1), 2);\n    assert_eq!(f(2), 3);\n}\n";
const DUPLICATED_AFTER: &str = "#[test]\nfn adds() {\n    assert_eq!(f(1), 2);\n    assert_eq!(f(1), 2);\n}\n";
const LISTED_PAIR: &str = "#[test]\nfn lists() {\n    assert_eq!(all(), [1, 2]);\n}\n";
const LISTED_ONE_WITH_MESSAGE: &str = "#[test]\nfn lists() {\n    assert_eq!(all(), [1], \"x\");\n}\n";
const ROW_CONSTANT_LOOP: &str = "const ROWS: [(u8, u8); 3] = [(1, 2), (2, 3), (3, 4)];\n#[test]\nfn adds() {\n    for (input, want) in ROWS {\n        assert_eq!(f(input), want);\n    }\n}\n";
const ROW_CONSTANT_READ: &str = "const ROWS: [(u8, u8); 3] = [(1, 2), (2, 3), (3, 4)];\n#[test]\nfn adds() {\n    assert_eq!(f(1), ROWS.iter().next().unwrap().1);\n}\n";
const LOOP_THOUSAND: &str =
    "#[test]\nfn adds() {\n    for x in 0..1000u32 {\n        assert_eq!(f(x), x + 1);\n    }\n}\n";
const PROPERTY_TWO: &str =
    "proptest! {\n    #[test]\n    fn adds(x in 0..2u32) {\n        prop_assert_eq!(f(x), x + 1);\n    }\n}\n";
const PROPERTY_THOUSAND: &str =
    "proptest! {\n    #[test]\n    fn adds(x in 0..1000u32) {\n        prop_assert_eq!(f(x), x + 1);\n    }\n}\n";
const TEN_ROWS: &str = "#[test]\nfn increments() {\n    for (input, wanted) in [(1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 7), (7, 8), (8, 9), (9, 10), (10, 11)] {\n        assert_eq!(inc(input), wanted);\n    }\n}\n";
const PROPERTY_INCREMENTS: &str = "proptest! {\n    #[test]\n    fn increments(input in 0..2u8) {\n        prop_assert_eq!(inc(input), input + 1);\n    }\n}\n";
const MANY_CASES: &str = "proptest! {\n    #![proptest_config(ProptestConfig::with_cases(10_000))]\n    #[test]\n    fn roundtrips(x in any::<u32>()) {\n        prop_assert_eq!(decode(encode(x)), x);\n    }\n}\n";
const ONE_CASE: &str = "proptest! {\n    #![proptest_config(ProptestConfig::with_cases(1))]\n    #[test]\n    fn roundtrips(x in any::<u32>()) {\n        prop_assert_eq!(decode(encode(x)), x);\n    }\n}\n";
const LONG_VECTORS: &str = "proptest! {\n    #[test]\n    fn sorts(v in prop::collection::vec(any::<u8>(), 0..1000)) {\n        prop_assert!(is_sorted(&sort(v)));\n    }\n}\n";
const SHORT_VECTORS: &str = "proptest! {\n    #[test]\n    fn sorts(v in prop::collection::vec(any::<u8>(), 0..1)) {\n        prop_assert!(is_sorted(&sort(v)));\n    }\n}\n";
const LONG_WORDS: &str = "proptest! {\n    #[test]\n    fn parses(s in \"[a-z]{1,20}\") {\n        prop_assert!(parse(&s).is_ok());\n    }\n}\n";
const ONE_WORD: &str =
    "proptest! {\n    #[test]\n    fn parses(s in \"a\") {\n        prop_assert!(parse(&s).is_ok());\n    }\n}\n";
const SPLIT_KEEPING_NAME: &str =
    "#[test]\nfn adds() {\n    assert_eq!(f(1), 2);\n}\n#[test]\nfn adds_two() {\n    assert_eq!(f(2), 3);\n}\n";
const SEPARATE_TESTS: &str =
    "#[test]\nfn adds_one() {\n    assert_eq!(f(1), 2);\n}\n#[test]\nfn adds_two() {\n    assert_eq!(f(2), 3);\n}\n";
const MERGED_INTO_ONE: &str = "#[test]\nfn adds_one() {\n    assert_eq!(f(1), 2);\n    assert_eq!(f(2), 3);\n}\n";
const LOOP_THREE: &str = "#[test]\nfn adds() {\n    for x in 0..3u8 {\n        assert_eq!(f(x), x + 1);\n    }\n}\n";
const UNROLLED_THREE: &str =
    "#[test]\nfn adds() {\n    assert_eq!(f(0), 1);\n    assert_eq!(f(1), 2);\n    assert_eq!(f(2), 3);\n}\n";
const LOOP_TEN: &str = "#[test]\nfn adds() {\n    for x in 0..10u8 {\n        assert_eq!(f(x), x + 1);\n    }\n}\n";
const SPLIT_LOOPS: &str = "#[test]\nfn adds() {\n    for x in 0..5u8 {\n        assert_eq!(f(x), x + 1);\n    }\n    for x in 5..10u8 {\n        assert_eq!(f(x), x + 1);\n    }\n}\n";
const TWO_BOUNDS: &str = "#[test]\nfn bounds() {\n    assert!(f(1) > 0);\n    assert!(f(1) < 10);\n}\n";
const JOINED_BOUNDS: &str = "#[test]\nfn bounds() {\n    assert!((f(1) > 0) && (f(1) < 10));\n}\n";
const EQUALS_SOME: &str = "#[test]\nfn finds() {\n    assert_eq!(find(1), Some(3));\n}\n";
const IS_SOME: &str = "#[test]\nfn finds() {\n    assert!(find(1).is_some());\n}\n";
const MATCHES_FIVE: &str = "#[test]\nfn finds() {\n    assert!(matches!(find(1), Some(1..=5)));\n}\n";
const MATCHES_NINE: &str = "#[test]\nfn finds() {\n    assert!(matches!(find(1), Some(1..=9)));\n}\n";
const ABOVE_ZERO: &str = "#[test]\nfn adds() {\n    assert!(f(1) > 0);\n}\n";
const AT_LEAST_ZERO: &str = "#[test]\nfn adds() {\n    assert!(f(1) >= 0);\n}\n";
const EQUALS_SWAPPED: &str = "#[test]\nfn adds() {\n    assert_eq!(2, f(1));\n}\n";
const REPINNED: &str = "#[test]\nfn adds() {\n    assert_eq!(f(1), 3);\n}\n";
const LAW_ANY_BYTE: &str = "laws::associative!(joins, any::<u8>(), join);\n";
const LAW_TWO_BYTES: &str = "laws::associative!(joins, 0..2u8, join);\n";
const EMPTY_LOOP_ONLY: &str = "#[test]\nfn adds() {\n    for _ in 0..0u8 {\n        assert_eq!(f(1), 2);\n    }\n}\n";

laws::table!(claims_are_entailed_or_lost, Weakened::in_one_module, [
    [DUPLICATED_BEFORE, DUPLICATED_AFTER] => true,
    [LISTED_PAIR, LISTED_ONE_WITH_MESSAGE] => true,
    [ROW_CONSTANT_LOOP, ROW_CONSTANT_READ] => true,
    [LOOP_THOUSAND, PROPERTY_TWO] => true,
    [LOOP_THOUSAND, PROPERTY_THOUSAND] => true,
    [PROPERTY_TWO, LOOP_THOUSAND] => false,
    [TEN_ROWS, PROPERTY_INCREMENTS] => true,
    [MANY_CASES, ONE_CASE] => true,
    [ONE_CASE, MANY_CASES] => false,
    [LONG_VECTORS, SHORT_VECTORS] => true,
    [LONG_WORDS, ONE_WORD] => true,
    [LONG_VECTORS, LONG_VECTORS] => false,
    [DUPLICATED_BEFORE, SPLIT_KEEPING_NAME] => false,
    [SEPARATE_TESTS, MERGED_INTO_ONE] => false,
    [LOOP_THREE, UNROLLED_THREE] => false,
    [UNROLLED_THREE, LOOP_THREE] => false,
    [LOOP_TEN, SPLIT_LOOPS] => false,
    [SPLIT_LOOPS, LOOP_TEN] => false,
    [LOOP_TEN, LOOP_THREE] => true,
    [TWO_BOUNDS, JOINED_BOUNDS] => false,
    [JOINED_BOUNDS, TWO_BOUNDS] => false,
    [EQUALS_SOME, IS_SOME] => true,
    [IS_SOME, EQUALS_SOME] => false,
    [MATCHES_FIVE, MATCHES_NINE] => true,
    [MATCHES_NINE, MATCHES_FIVE] => false,
    [ABOVE_ZERO, AT_LEAST_ZERO] => true,
    [AT_LEAST_ZERO, ABOVE_ZERO] => false,
    [EQUALS, EQUALS_SWAPPED] => false,
    [EQUALS, SPELLED_OUT] => false,
    [EQUALS, REPINNED] => true,
    [LAW_ANY_BYTE, LAW_TWO_BYTES] => true,
    [LAW_TWO_BYTES, LAW_ANY_BYTE] => false,
    [EMPTY_LOOP_ONLY, EMPTY] => false,
    [EQUALS, EMPTY_LOOP_ONLY] => true,
]);

const CLOSURE_NEVER_CALLED: &str = "#[test]\nfn adds() {\n    let _never = || assert_eq!(f(1), 2);\n}\n";
const NESTED_FN_NEVER_CALLED: &str = "#[test]\nfn adds() {\n    fn never() {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const CATCH_UNWIND: &str =
    "#[test]\nfn adds() {\n    let _swallowed = std::panic::catch_unwind(|| assert_eq!(f(1), 2));\n}\n";
const CFG_ANY_STATEMENT: &str = "#[test]\nfn adds() {\n    #[cfg(any())]\n    assert_eq!(f(1), 2);\n}\n";
const SHOULD_PANIC: &str = "#[test]\n#[should_panic]\nfn adds() {\n    assert_eq!(f(1), 2);\n}\n";
const CFG_ANY_TEST: &str = "#[cfg(any())]\n#[test]\nfn adds() {\n    assert_eq!(f(1), 2);\n}\n";
const LOOP_ONLY_FIRST: &str = "#[test]\nfn adds() {\n    for x in 0..1000u32 {\n        if x == 0 {\n            assert_eq!(f(x), x + 1);\n        }\n    }\n}\n";
const LOOP_BREAKS: &str =
    "#[test]\nfn adds() {\n    for x in 0..1000u32 {\n        assert_eq!(f(x), x + 1);\n        break;\n    }\n}\n";
const LOOP_EVERY_OTHER: &str = "#[test]\nfn adds() {\n    for x in 0..1000u32 {\n        if x % 2 == 0 {\n            assert_eq!(f(x), x + 1);\n        }\n    }\n}\n";
const LOOP_LEAVES_EARLY: &str = "#[test]\nfn adds() {\n    for x in 0..1000u32 {\n        if x == 3 {\n            break;\n        }\n        assert_eq!(f(x), x + 1);\n    }\n}\n";
const CLOSURE_CALLED: &str = "#[test]\nfn adds() {\n    let check = || assert_eq!(f(1), 2);\n    check();\n}\n";
const FOR_EACH: &str = "#[test]\nfn adds() {\n    items.iter().for_each(|x| assert_eq!(f(*x), 2));\n}\n";
const FOR_LOOP: &str = "#[test]\nfn adds() {\n    for x in items {\n        assert_eq!(f(x), 2);\n    }\n}\n";
const FOUND_SOME: &str = "#[test]\nfn finds() {\n    assert!(find().is_some());\n}\n";
const FOUND_UNWRAPPED: &str = "#[test]\nfn finds() {\n    find().unwrap();\n}\n";
const UNWRAPPED_THEN_EQUAL: &str = "#[test]\nfn finds() {\n    let v = find().unwrap();\n    assert_eq!(v, 3);\n}\n";
const EQUAL_SOME_THREE: &str = "#[test]\nfn finds() {\n    assert_eq!(find(), Some(3));\n}\n";
const CFG_MACRO_GUARDED: &str =
    "#[test]\nfn adds() {\n    if cfg!(feature = \"x\") {\n        assert_eq!(f(1), 2);\n    }\n}\n";
const CFG_FEATURE_STATEMENT: &str = "#[test]\nfn adds() {\n    #[cfg(feature = \"x\")]\n    assert_eq!(f(1), 2);\n}\n";
const FRESH_AND_RESET: &str = "#[test]\nfn fresh() {\n    let c = Counter::new();\n    assert_eq!(c.get(), 0);\n}\n#[test]\nfn reset() {\n    let mut c = Counter::new();\n    c.inc();\n    c.reset();\n    assert_eq!(c.get(), 0);\n}\n";
const FRESH_ONLY: &str = "#[test]\nfn fresh() {\n    let c = Counter::new();\n    assert_eq!(c.get(), 0);\n}\n";
const MIDDLE_BAND: &str = "#[test]\nfn middle() {\n    assert!(matches!(f(1), 2..=4));\n}\n";
const HIGH_AND_LOW: &str =
    "#[test]\nfn high() {\n    assert!(f(1) > 5);\n}\n#[test]\nfn low() {\n    assert!(f(1) < 1);\n}\n";
const MIDDLE_COUNTED: &str = "#[test]\nfn middle() {\n    let mut c = Counter::new();\n    c.add(3);\n    assert!(matches!(c.get(), 2..=4));\n}\n";
const HIGH_AND_LOW_COUNTED: &str = "#[test]\nfn high() {\n    let mut c = Counter::new();\n    c.add(9);\n    assert!(c.get() > 5);\n}\n#[test]\nfn low() {\n    let c = Counter::new();\n    assert!(c.get() < 1);\n}\n";
const ACCUMULATED: &str =
    "#[test]\nfn sums() {\n    let mut total = 0;\n    total += f(1);\n    assert_eq!(total, 2);\n}\n";
const ACCUMULATED_UNCHECKED: &str = "#[test]\nfn sums() {\n    let mut total = 0;\n    total += f(1);\n}\n";
const MATCHES_NONE: &str = "#[test]\nfn finds() {\n    assert!(matches!(find(1), None));\n}\n";
const IS_NONE: &str = "#[test]\nfn finds() {\n    assert!(find(1).is_none());\n}\n";

laws::table!(reach_decides_what_a_test_establishes, Weakened::in_one_module, [
    [EQUALS, CLOSURE_NEVER_CALLED] => true,
    [EQUALS, NESTED_FN_NEVER_CALLED] => true,
    [EQUALS, CATCH_UNWIND] => true,
    [EQUALS, CFG_ANY_STATEMENT] => true,
    [EQUALS, SHOULD_PANIC] => true,
    [EQUALS, CFG_ANY_TEST] => true,
    [LOOP_THOUSAND, LOOP_ONLY_FIRST] => true,
    [LOOP_THOUSAND, LOOP_BREAKS] => true,
    [LOOP_THOUSAND, LOOP_EVERY_OTHER] => true,
    [LOOP_THOUSAND, LOOP_LEAVES_EARLY] => true,
    [CLOSURE_CALLED, EQUALS] => false,
    [FOR_EACH, FOR_LOOP] => false,
    [FOUND_SOME, FOUND_UNWRAPPED] => false,
    [UNWRAPPED_THEN_EQUAL, EQUAL_SOME_THREE] => false,
    [EQUALS, CFG_MACRO_GUARDED] => true,
    [EQUALS, CFG_FEATURE_STATEMENT] => true,
    [CFG_MACRO_GUARDED, CFG_MACRO_GUARDED] => false,
    [FRESH_AND_RESET, FRESH_ONLY] => true,
    [MIDDLE_BAND, HIGH_AND_LOW] => true,
    [MIDDLE_COUNTED, HIGH_AND_LOW_COUNTED] => true,
    [ACCUMULATED, ACCUMULATED_UNCHECKED] => true,
    [MATCHES_NONE, EMPTY] => true,
    [IS_NONE, MATCHES_NONE] => false,
    [MATCHES_NONE, IS_NONE] => false,
]);
