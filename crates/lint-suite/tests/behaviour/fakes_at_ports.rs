use lint_fakes_at_ports::FakesAtPorts;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const PORTED: &str = "mod ports {\n    pub trait Clock {}\n}\nstruct FakeClock;\nimpl ports::Clock for FakeClock {}\n";
const IMPORTED: &str = "use ci::ports::Ports;\nstruct Fake;\nimpl Ports for Fake {}\n";
const UNPORTED: &str =
    "struct FakeClock;\nimpl FakeClock {\n    fn new() -> FakeClock {\n        FakeClock\n    }\n}\n";
const LOCAL_TRAIT: &str = "trait Clock {}\nstruct StubClock;\nimpl Clock for StubClock {}\n";
const STD_TRAITS: &str = "use std::fmt;\nuse std::io::Write;\n#[derive(Debug, Clone)]\nstruct SpyLog;\nimpl ports::Log for SpyLog {}\nimpl fmt::Display for SpyLog {\n    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {\n        Ok(())\n    }\n}\nimpl Write for SpyLog {}\nimpl Drop for SpyLog {\n    fn drop(&mut self) {}\n}\nimpl Default for SpyLog {\n    fn default() -> SpyLog {\n        SpyLog\n    }\n}\nimpl core::hash::Hash for SpyLog {}\n";
const EXTRA: &str = "struct MockClock;\nimpl ports::Clock for MockClock {}\nimpl serde::Serialize for MockClock {}\n";
const PORTS_CRATE: &str = "struct DummyClock;\nimpl meta_ports::Clock for DummyClock {}\n";
const LOOKALIKE_CRATES: &str =
    "struct FakeA;\nimpl reports::Clock for FakeA {}\nstruct FakeB;\nimpl transports::Clock for FakeB {}\n";
const WORDS: &str = "struct Fakery;\nstruct Spyglass;\nstruct MockingBird;\nstruct SpyCamera;\nstruct HTTPStub;\nenum Dummy {}\nstruct Stubborn;\nstruct FAKEClock;\nstruct Clock2Spy;\n";
const GENERIC: &str = "struct FakeStore<T>(T);\nimpl<T> ports::Store for FakeStore<T> {}\n";
const RENAMED: &str = "use ci::ports::Ports as Effects;\nstruct Fake;\nimpl Effects for Fake {}\n";
const GLOB: &str = "use ci::ports::*;\nstruct Fake;\nimpl Ports for Fake {}\n";
const NESTED: &str = "mod helpers {\n    pub struct FakeClock;\n}\nunion DummyBits {\n    a: u8,\n}\n";
const DECLARED: &str = "struct FakeClock;\n";
const IMPLEMENTED: &str = "impl ports::Clock for super::FakeClock {}\n";
const SHIPPED_PORT: &str = "pub trait Clock {}\npub trait Calendar {}\n";
const SHIPPED_OTHER: &str = "pub trait Other {}\n";
const THROUGH_RE_EXPORT: &str =
    "struct FakeClock;\nimpl x::Clock for FakeClock {}\nstruct FakeOther;\nimpl x::Other for FakeOther {}\n";
const INLINE_SHIPPED_PORT: &str = "pub mod ports {\n    pub trait Clock {}\n}\npub use ports::Clock;\n";

const CASES: [Case; 19] = [
    Case::new(
        Title::new("implements a trait of a ports module"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(PORTED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("implements an imported port"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(IMPORTED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("implements no trait"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(UNPORTED))],
        Flagged::new(&["tests/a.rs:1"]),
    ),
    Case::new(
        Title::new("implements a trait outside ports"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(LOCAL_TRAIT))],
        Flagged::new(&["tests/a.rs:2", "tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("derives and std traits"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(STD_TRAITS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a port and another trait"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(EXTRA))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("a crate whose name ends in ports"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(PORTS_CRATE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("crates whose names only contain ports"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(LOOKALIKE_CRATES))],
        Flagged::new(&["tests/a.rs:1", "tests/a.rs:2", "tests/a.rs:3", "tests/a.rs:4"]),
    ),
    Case::new(
        Title::new("whole words only"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(WORDS))],
        Flagged::new(&["tests/a.rs:4", "tests/a.rs:5", "tests/a.rs:6", "tests/a.rs:8", "tests/a.rs:9"]),
    ),
    Case::new(
        Title::new("generic double"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(GENERIC))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("renamed port"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(RENAMED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("glob imported port"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(GLOB))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("shipped code"),
        &[Source::new(Located::new("src/a.rs"), Text::new(UNPORTED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inside a module, and a union"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(NESTED))],
        Flagged::new(&["tests/a.rs:2", "tests/a.rs:4"]),
    ),
    Case::new(
        Title::new("example"),
        &[Source::new(Located::new("examples/a.rs"), Text::new(UNPORTED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("implemented in another file of the same test target"),
        &[
            Source::new(Located::new("tests/c/main.rs"), Text::new(DECLARED)),
            Source::new(Located::new("tests/c/impls.rs"), Text::new(IMPLEMENTED)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("implemented in another test target"),
        &[
            Source::new(Located::new("tests/a.rs"), Text::new(DECLARED)),
            Source::new(Located::new("tests/b.rs"), Text::new(IMPLEMENTED)),
        ],
        Flagged::new(&["tests/a.rs:1"]),
    ),
    Case::new(
        Title::new("a port re-exported by its crate"),
        &[
            Source::new(Located::new("crates/x/src/ports.rs"), Text::new(SHIPPED_PORT)),
            Source::new(Located::new("crates/x/src/other.rs"), Text::new(SHIPPED_OTHER)),
            Source::new(Located::new("crates/x/tests/a.rs"), Text::new(THROUGH_RE_EXPORT)),
        ],
        Flagged::new(&["crates/x/tests/a.rs:3", "crates/x/tests/a.rs:4"]),
    ),
    Case::new(
        Title::new("a port declared in an inline ports module of a crate"),
        &[
            Source::new(Located::new("crates/x/src/lib.rs"), Text::new(INLINE_SHIPPED_PORT)),
            Source::new(Located::new("crates/x/tests/a.rs"), Text::new(THROUGH_RE_EXPORT)),
        ],
        Flagged::new(&["crates/x/tests/a.rs:3", "crates/x/tests/a.rs:4"]),
    ),
];

#[test]
fn test_doubles_implement_only_ports() {
    assert_eq!(Sources::mismatches::<FakesAtPorts>(&CASES), Lines::default());
}
