use lint_file::Primed;
use lint_state_graph::StateGraph;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const OPEN_AND_CLOSED: &str = "struct Open;\nstruct Closed;\nimpl State for Open {}\nimpl State for Closed {}\n";
const EDGE: &str = "impl Open {\n    pub fn close(self) -> Closed {\n        Closed\n    }\n}\n";
const MUTATES: &str = "impl Open {\n    pub fn tweak(&mut self) {}\n}\n";

const NESTED: &str = "fn f() {\n    mod inner {\n        states::states! {\n            #[derive()]\n            mod door {\n                start Open { shut -> Shut, }\n                terminal Shut {}\n                state Lost {}\n            }\n        }\n    }\n}\n";
const UNREACHABLE: &str = "states! {\n    mod door {\n        start Open { shut -> Shut }\n        state Ajar { shut -> Shut }\n        terminal Shut {}\n    }\n}\n";
const NO_START: &str =
    "states! {\n    mod door {\n        state Open { shut -> Shut }\n        terminal Shut {}\n    }\n}\n";
const NO_TERMINAL: &str = "states! {\n    mod door {\n        start Open {}\n    }\n}\n";
const OTHER_RENAME: &str = "use other::thing as lifecycle;\nlifecycle! {\n    mod door {\n        start Open { shut -> Shut }\n        state Lost {}\n        terminal Shut {}\n    }\n}\n";
const NOT_A_GRAPH: &str = "fn f() {\n    states!(1 + 1);\n}\n";
const IN_TEST_MODULE: &str =
    "#[cfg(test)]\nmod tests {\n    states! {\n        mod door {\n            begin Open {}\n        }\n    }\n}\n";
const OTHER_MACRO: &str = "fn f() {\n    print!(\"start Open\");\n}\n";
const THE_DEFINITION: &str = "macro_rules! states {\n    () => {};\n}\n";

const VALUE_VIEW: &str = "impl Value {\n    pub fn report(&self) -> Report {\n        make()\n    }\n}\n";
const SELF_VIEW: &str = "impl Open {\n    pub fn copy(&self) -> Self {\n        make()\n    }\n}\n";
const OPTION_VIEW: &str = "impl Open {\n    pub fn next(&self) -> Option<Closed> {\n        make()\n    }\n}\n";
const FIELD_VIEW: &str = "impl Open {\n    pub fn next(&self) -> Closed {\n        self.closed\n    }\n}\n";
const OTHER_FIELD_VIEW: &str =
    "impl Open {\n    pub fn next(&self, other: Other) -> Closed {\n        other.closed\n    }\n}\n";
const ALIAS_VERTEX: &str = "type Door = Open;\nimpl State for Door {}\n";
const INLINE_VERTEX: &str = "mod inner {\n    pub struct Open;\n    impl State for Open {}\n    impl Open {\n        pub fn tweak(&mut self) {}\n    }\n}\n";
const INLINE_FIELD: &str = "mod inner {\n    pub struct A {\n        pub depth: u8,\n    }\n}\n";
const FREE_TRANSITION: &str =
    "struct Maker;\nimpl Maker {\n    pub fn close(open: Open) -> Closed {\n        make()\n    }\n}\n";
const TRAIT_TRANSITION: &str =
    "impl From<Open> for Closed {\n    fn from(open: Open) -> Closed {\n        make()\n    }\n}\n";
const TWO_TRAITS: &str = "impl Wrap for Open {\n    fn wrap(self) -> Closed {\n        make()\n    }\n}\nimpl Seal for Open {\n    fn seal(self) -> Closed {\n        make()\n    }\n}\n";

const BOXED_VIEW: &str = "impl Open {\n    pub fn next(&self) -> Box<Closed> {\n        make()\n    }\n}\n";

const ALIASED: &str = "use states::states as lifecycle;\nlifecycle! {\n    mod door {\n        start Open { shut -> Shut }\n        state Lost {}\n        terminal Shut {}\n    }\n}\n";
const GROUPED_ALIAS: &str = "use states::{states as lifecycle, other};\nfn f() {\n    lifecycle! {\n        mod door {\n            start Open { shut -> Shut }\n            state Lost {}\n            terminal Shut {}\n        }\n    }\n}\n";
const BEHIND_CFG_IF: &str = "cfg_if::cfg_if! {\n    if #[cfg(unix)] {\n        states::states! {\n            mod door {\n                start Open { shut -> Shut }\n                state Lost {}\n                terminal Shut {}\n            }\n        }\n    }\n}\n";
const STORED: &str = "states! {\n    pub stored Door in door {\n        start Open as Closing { shut -> Shut }\n        state Lost {}\n        terminal Shut {}\n    }\n}\n";
const BEHIND_MACRO_RULES: &str = "macro_rules! door {\n    () => {\n        states::states! {\n            mod door {\n                start Open { shut -> Shut }\n                state Lost {}\n                terminal Shut {}\n            }\n        }\n    };\n}\ndoor!();\n";

const CASES: [Case; 13] = [
    Case::new(
        Title::new("public edge"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(EDGE)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a value's view"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(VALUE_VIEW)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a view returning Self"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(SELF_VIEW)),
        ],
        Flagged::new(&["b.rs:2"]),
    ),
    Case::new(
        Title::new("a view returning an option"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(OPTION_VIEW)),
        ],
        Flagged::new(&["b.rs:2"]),
    ),
    Case::new(
        Title::new("a view of a field"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(FIELD_VIEW)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a view of another value's field"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(OTHER_FIELD_VIEW)),
        ],
        Flagged::new(&["b.rs:2"]),
    ),
    Case::new(
        Title::new("a vertex behind an alias"),
        &[
            Source::new(Located::new("a.rs"), Text::new(ALIAS_VERTEX)),
            Source::new(Located::new("b.rs"), Text::new(MUTATES)),
        ],
        Flagged::new(&["b.rs:2"]),
    ),
    Case::new(
        Title::new("a vertex in an inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE_VERTEX))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("a field in an inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE_FIELD))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("a free transition"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(FREE_TRANSITION)),
        ],
        Flagged::new(&["b.rs:3"]),
    ),
    Case::new(
        Title::new("a trait transition"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(TRAIT_TRANSITION)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("one edge through two traits"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(TWO_TRAITS)),
        ],
        Flagged::new(&["b.rs:2"]),
    ),
    Case::new(
        Title::new("a view returning another wrapper"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(BOXED_VIEW)),
        ],
        Flagged::new(&[]),
    ),
];

#[test]
fn state_graph_rules_are_flagged() {
    assert_eq!(Sources::mismatches::<Primed<StateGraph>>(&CASES), Lines::default());
}

laws::table!(declared_graphs_that_hold_pass, Sources::flagged::<Primed<StateGraph>>, [
    [Source::new(Located::new("a.rs"), Text::new(IN_TEST_MODULE))] => Lines::default(),
    [Source::new(Located::new("x/tests/a.rs"), Text::new(UNREACHABLE))] => Lines::default(),
    [Source::new(Located::new("a.rs"), Text::new(OTHER_MACRO))] => Lines::default(),
    [Source::new(Located::new("a.rs"), Text::new(OTHER_RENAME))] => Lines::default(),
    [Source::new(Located::new("a.rs"), Text::new(THE_DEFINITION))] => Lines::default(),
]);

laws::table!(declared_graphs_that_break_are_flagged, Sources::flagged::<Primed<StateGraph>>, [
    [Source::new(Located::new("a.rs"), Text::new(NESTED))] => Lines::from(["a.rs:8", "a.rs:8"]),
    [Source::new(Located::new("a.rs"), Text::new(NO_START))] => Lines::from(["a.rs:3", "a.rs:4"]),
    [Source::new(Located::new("a.rs"), Text::new(NO_TERMINAL))] => Lines::from(["a.rs:3"]),
    [Source::new(Located::new("a.rs"), Text::new(NOT_A_GRAPH))] => Lines::from(["a.rs:2"]),
    [Source::new(Located::new("a.rs"), Text::new(ALIASED))] => Lines::from(["a.rs:5", "a.rs:5"]),
    [Source::new(Located::new("a.rs"), Text::new(STORED))] => Lines::from(["a.rs:4", "a.rs:4"]),
    [Source::new(Located::new("a.rs"), Text::new(GROUPED_ALIAS))] => Lines::from(["a.rs:6", "a.rs:6"]),
    [Source::new(Located::new("a.rs"), Text::new(BEHIND_CFG_IF))] => Lines::from(["a.rs:6", "a.rs:6"]),
    [Source::new(Located::new("a.rs"), Text::new(BEHIND_MACRO_RULES))] => Lines::from(["a.rs:6", "a.rs:6"]),
]);
