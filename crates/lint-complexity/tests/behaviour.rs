use lint_complexity::Complexity;
use lint_fixtures::{Located, Source, Sources, Text};

struct Scored;

impl Scored {
    fn of(text: &str) -> String {
        let sources = [Source::new(Located::new("a.rs"), Text::new(Box::leak(text.to_owned().into_boxed_str())))];
        Sources::report::<Complexity>(&sources).to_string()
    }

    fn highest_is(pair: &[&str]) -> bool {
        match pair {
            [text, expected] => Self::of(text).contains(&format!("highest {expected}\n")),
            _ => false,
        }
    }

    fn lists(pair: &[&str]) -> bool {
        match pair {
            [text, expected] => Self::of(text).contains(expected),
            _ => false,
        }
    }
}

const BAND_BODIES: &str = "fn a() { f(); }\nfn b(x: u8) { if x > 1 { f(); } }\nfn c(x: u8) { if x > 1 { f(); } if x > 2 { f(); } if x > 3 { f(); } }\nfn d(x: u8) { match x { 0 => f(), 1 => f(), 2 => f(), 3 => f(), 4 => f(), _ => f() } }\nfn e(x: u8) { match x { 0 => f(), 1 => f(), 2 => f(), 3 => f(), 4 => f(), 5 => f(), 6 => f(), 7 => f(), 8 => f(), _ => f() } }\nfn h(x: u8) { if x > 1 { f(); } if x > 2 { f(); } }\nfn i(x: u8) { match x { 0 => f(), 1 => f(), 2 => f(), 3 => f(), 4 => f(), 5 => f(), _ => f() } }\nfn j(x: u8) { match x { 0 => f(), 1 => f(), 2 => f(), 3 => f(), 4 => f(), 5 => f(), 6 => f(), 7 => f(), 8 => f(), 9 => f(), _ => f() } }\nfn g(x: u8) { match x { 0 => f(), 1 => f(), 2 => f(), 3 => f(), 4 => f(), 5 => f(), 6 => f(), 7 => f(), 8 => f(), 9 => f(), 10 => f(), _ => f() } }\n";

laws::table!(
    each_branch_adds_one_to_the_highest_score_of_a_function,
    Scored::highest_is,
    [
        ["fn a() { f(); g(); }", "1 in a"] => true,
        ["fn a(x: u8) { if x > 1 { f(); } }", "2 in a"] => true,
        ["fn a(x: u8) { if x > 1 { f(); } else if x > 2 { g(); } }", "3 in a"] => true,
        ["fn a(x: u8) { while x > 1 { f(); } }", "2 in a"] => true,
        ["fn a(x: Vec<u8>) { for y in x { f(y); } }", "2 in a"] => true,
        ["fn a(x: u8) -> Option<u8> { let y = f(x)?; Some(y) }", "2 in a"] => true,
        ["fn a(x: bool, y: bool) -> bool { x && y }", "2 in a"] => true,
        ["fn a(x: bool, y: bool) -> bool { x || y }", "2 in a"] => true,
        ["fn a(x: u8, y: u8) -> u8 { x + y }", "1 in a"] => true,
        ["fn a(x: u8) { match x { 0 => f(), 1 => g(), _ => h() } }", "3 in a"] => true,
        ["fn a(x: u8) { match x { _ => h() } }", "1 in a"] => true,
        ["fn a(x: u8) { match x {} }", "1 in a"] => true,
        ["fn a() { loop { f(); } }", "1 in a"] => true,
        ["fn a(x: u8) { let g = |y: u8| if y > x { 1 } else { 2 }; g(x); }", "2 in a"] => true,
        ["struct S;\nimpl S { fn m(&self, x: u8) { if x > 1 { f(); } } }", "2 in m"] => true,
        ["trait T { fn p(&self, x: u8) { if x > 1 { f(); } } fn q(&self); }", "2 in p"] => true,
        ["fn a() { f(); }\n#[cfg(test)]\nmod tests { fn b(x: u8) { if x > 1 { f(); } } }\n#[test]\nfn c(x: u8) { if x > 1 { f(); } }", "1 in a"] => true,
    ]
);

laws::table!(
    the_distribution_lists_each_crate_by_band_and_leaves_out_empty_ones,
    Scored::lists,
    [
        [BAND_BODIES, "9 functions by complexity, 1: 1, 2-3: 2, 4-6: 2, 7-10: 2, 11+: 2; highest 12 in g"] => true,
        ["fn a() { fn b(x: u8) { if x > 1 { f(); } } b(1); }", "2 functions by complexity, 1: 1, 2-3: 1; highest 2 in b"] => true,
        ["trait T { fn q(&self); }", "functions"] => false,
        ["struct S;", "functions"] => false,
    ]
);

#[test]
fn each_crate_gets_its_own_line_and_a_crate_without_functions_none() {
    let sources = [
        Source::new(Located::new("crates/x/src/lib.rs"), Text::new("fn a() { f(); }")),
        Source::new(Located::new("crates/y/src/lib.rs"), Text::new("struct S;")),
        Source::new(Located::new("crates/z/src/lib.rs"), Text::new("fn b(x: u8) { if x > 1 { f(); } }")),
        Source::new(Located::new("crates/w/tests/t.rs"), Text::new("fn c(x: u8) { if x > 1 { f(); } }")),
    ];
    let report = Sources::report::<Complexity>(&sources);
    assert_eq!(report.violations().len(), 2);
    let text = report.to_string();
    assert!(text.contains("crate x: 1 functions by complexity, 1: 1; highest 1 in a"), "{text}");
    assert!(text.contains("crate z: 1 functions by complexity, 2-3: 1; highest 2 in b"), "{text}");
}
