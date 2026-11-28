use std::env;
use std::ffi::OsString;
use std::iter;
use std::path::Path;
use std::time::{Duration, Instant};

use file_ports::{Bytes, Memory};
use invariants::{Ignoring, ignored};
use lint::{Cli, Verdict};
use proptest::collection::vec;
use proptest::prelude::{BoxedStrategy, Strategy, any, prop_assert_eq, prop_oneof};
use proptest::sample::{Index, select};
use proptest::test_runner::{Config, TestCaseError, TestError, TestRunner};
use timer_ports::Monotonic;

ignored! {
    enum Ignored {
        Unset => "an unset or unreadable FUZZ_SECONDS is the default budget",
    }
}

const ROOT: &str = "/fuzzed";
const MANIFEST: &str = "[package]\nname = \"fuzzed\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";
const FILES: [&str; 3] = ["src/lib.rs", "src/main.rs", "tests/fuzzed.rs"];
const LEAVES: [&str; 13] = [
    "0",
    "x",
    "\"text\"",
    "'c'",
    "true",
    "()",
    "None",
    "Self::new()",
    "std::process::Command::new(\"ls\")",
    "std::fs::remove_dir_all(\"p\")",
    "is_ready",
    "from_str",
    "ports",
];
const TYPES: [&str; 10] = [
    "u8",
    "String",
    "&'a str",
    "Vec<u8>",
    "Option<Foo>",
    "Box<dyn Bar>",
    "(u8, u8)",
    "[u8; 4]",
    "bool",
    "Result<(), Foo>",
];
const FLAT: [&str; 10] = [
    "use std::process::Command;",
    "use std::fs::{self, remove_dir_all as gone};",
    "use std::*;",
    "pub use super::x::y;",
    "// note",
    "/// doc",
    "//! inner",
    "macro_rules! m { ($x:expr) => { $x + 1 }; }",
    "union U { a: u8, b: u16 }",
    "type T<'a> = &'a u8;",
];
const ATTRIBUTES: [&str; 6] = ["", "#[test]", "#[cfg(test)]", "#[allow(dead_code)]", "#[derive(Debug)]", "#[must_use]"];
const VISIBILITY: [&str; 3] = ["", "pub ", "pub(crate) "];
const SHELLS: [[&str; 2]; 6] = [["(", ")"], ["{ ", " }"], ["[", "]"], ["-", ""], ["&", ""], ["match x { _ => ", " }"]];
const LINKS: [&str; 5] = [".m()", " + 1", "?", " as u8", ".0"];
const DEEPEST: usize = 140;
const LONGEST: usize = 30_000;
const BYTES: usize = 512;
const LISTED: usize = 4;
const ITEMS: usize = 10;
const BATCH: u32 = 64;
const SECONDS: &str = "FUZZ_SECONDS";
const DEFAULT_SECONDS: u64 = 5;

#[derive(Clone, Debug)]
struct Input {
    file: &'static str,
    manifest: Vec<u8>,
    source: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
struct Run {
    verdict: Verdict,
    err: Vec<u8>,
}

struct Syntax;

#[test]
fn no_input_panics_the_runner_or_breaks_its_promises() -> Result<(), TestError<Input>> {
    let seconds = env::var(SECONDS).ignoring(Ignored::Unset).and_then(|text| text.parse().ignoring(Ignored::Unset));
    let deadline = Instant::now() + Duration::from_secs(seconds.unwrap_or(DEFAULT_SECONDS));
    let inputs = Input::all();
    iter::repeat_with(Instant::now).take_while(|now| *now < deadline).try_for_each(|_| {
        TestRunner::new(Config { cases: BATCH, failure_persistence: None, ..Config::default() })
            .run(&inputs, Input::checked)
    })
}

impl Input {
    fn sourced(file: &'static str, text: String) -> Input {
        Input { file, manifest: MANIFEST.as_bytes().to_vec(), source: text.into_bytes() }
    }

    fn all() -> BoxedStrategy<Input> {
        let files = select(FILES.to_vec());
        prop_oneof![
            (files.clone(), vec(any::<u8>(), 0..BYTES))
                .prop_map(|(file, source)| Input { source, ..Input::sourced(file, String::new()) }),
            vec(any::<u8>(), 0..BYTES)
                .prop_map(|manifest| Input { manifest, ..Input::sourced("src/lib.rs", String::new()) }),
            (files.clone(), Syntax::file(), any::<Index>(), any::<u8>(), 0..4_u8).prop_map(
                |(file, text, place, byte, kind)| Input {
                    source: Syntax::damaged(text.as_bytes(), place, byte, kind),
                    ..Input::sourced(file, String::new())
                }
            ),
            (files.clone(), select(SHELLS.to_vec()), 1..DEEPEST, Syntax::expression()).prop_map(
                |(file, [open, close], depth, core)| Input::sourced(
                    file,
                    format!("fn deep() {{ {}{core}{} }}", open.repeat(depth), close.repeat(depth))
                )
            ),
            (files, select(LINKS.to_vec()), 1..LONGEST).prop_map(|(file, link, length)| Input::sourced(
                file,
                format!("fn long() {{ x{} }}", link.repeat(length))
            )),
        ]
        .boxed()
    }

    fn run(&self) -> Run {
        let root = Path::new(ROOT);
        let files: Memory = [
            (root.join("Cargo.toml"), Bytes::new(self.manifest.clone())),
            (root.join(self.file), Bytes::new(self.source.clone())),
        ]
        .into_iter()
        .collect();
        let mut err = Vec::new();
        let verdict = Cli::from_args([OsString::from(ROOT)]).verdict(
            &files,
            Path::new(""),
            Monotonic::default(),
            &mut Vec::new(),
            &mut err,
        );
        Run { verdict, err }
    }

    fn checked(self) -> Result<(), TestCaseError> {
        let run = self.run();
        prop_assert_eq!(&run, &self.run(), "two runs of the runner disagreed");
        prop_assert_eq!(
            run.err.is_empty(),
            run.verdict == Verdict::Clean,
            "a run explains itself exactly when it fails: {}",
            run.err.escape_ascii()
        );
        Ok(())
    }
}

impl Syntax {
    fn pick(table: &'static [&'static str]) -> BoxedStrategy<String> {
        select(table).prop_map(str::to_owned).boxed()
    }

    fn list(each: BoxedStrategy<String>) -> BoxedStrategy<String> {
        vec(each, 0..LISTED).prop_map(|parts| parts.join(", ")).boxed()
    }

    fn name() -> BoxedStrategy<String> {
        prop_oneof![Syntax::pick(&LEAVES), "[a-z_][a-z0-9_]{0,8}x"].boxed()
    }

    fn ty() -> BoxedStrategy<String> {
        Syntax::pick(&TYPES)
            .prop_recursive(4, 48, 4, |inner| {
                prop_oneof![
                    inner.clone().prop_map(|ty| format!("Vec<{ty}>")),
                    inner.clone().prop_map(|ty| format!("&mut {ty}")),
                    Syntax::list(inner).prop_map(|tys| format!("({tys})"))
                ]
            })
            .boxed()
    }

    fn expression() -> BoxedStrategy<String> {
        Syntax::name()
            .prop_recursive(4, 48, 4, |inner| {
                prop_oneof![
                    (Syntax::name(), Syntax::list(inner.clone())).prop_map(|(name, args)| format!("{name}({args})")),
                    (inner.clone(), Syntax::name()).prop_map(|(on, name)| format!("{on}.{name}()")),
                    (inner.clone(), inner.clone())
                        .prop_map(|(test, then)| format!("if {test} {{ {then} }} else {{ 0 }}")),
                    (inner.clone(), inner.clone())
                        .prop_map(|(on, arm)| format!("match {on} {{ Some(v) if v => {arm}, _ => 0 }}")),
                    (inner.clone(), inner.clone()).prop_map(|(over, body)| format!("for i in {over} {{ {body}; }}")),
                    inner.clone().prop_map(|value| format!("|v: u8| {{ let y = {value}; y }}")),
                    inner.clone().prop_map(|value| format!("unsafe {{ {value}? }}")),
                    Syntax::list(inner).prop_map(|args| format!("vec![{args}]")),
                ]
            })
            .boxed()
    }

    fn function() -> BoxedStrategy<String> {
        let statement = prop_oneof![
            Syntax::expression().prop_map(|value| format!("{value};")),
            (Syntax::name(), Syntax::expression()).prop_map(|(name, value)| format!("let {name} = {value};")),
        ];
        let statements = vec(statement, 0..LISTED).prop_map(|parts| parts.join(" "));
        (Syntax::pick(&ATTRIBUTES), Syntax::pick(&VISIBILITY), Syntax::name(), Syntax::ty(), Syntax::ty(), statements)
            .prop_map(|(attribute, vis, name, taken, given, body)| {
                format!("{attribute} {vis}fn {name}<'a>(a: {taken}) -> {given} {{ {body} }}")
            })
            .boxed()
    }

    fn item() -> BoxedStrategy<String> {
        let declaration = prop_oneof![
            (Syntax::pick(&ATTRIBUTES), Syntax::pick(&VISIBILITY), Syntax::name(), Syntax::ty()).prop_map(
                |(attribute, vis, name, ty)| format!("{attribute} {vis}struct {name}<'a> {{ {vis}field: {ty} }}")
            ),
            (Syntax::name(), Syntax::name(), Syntax::ty())
                .prop_map(|(name, variant, ty)| format!("enum {name} {{ {variant}({ty}), Other {{ a: u8 }}, Unit }}")),
            (Syntax::name(), Syntax::ty(), Syntax::expression())
                .prop_map(|(name, ty, value)| format!("pub const {name}: {ty} = {value};")),
        ];
        prop_oneof![Syntax::function(), declaration, Syntax::pick(&FLAT)]
            .prop_recursive(4, 48, 4, |inner| {
                let items = vec(inner, 0..LISTED).prop_map(|parts| parts.join("\n"));
                let members = vec(Syntax::function(), 0..LISTED).prop_map(|parts| parts.join("\n"));
                prop_oneof![
                    (Syntax::name(), items.clone()).prop_map(|(name, body)| format!("mod {name} {{ {body} }}")),
                    items.prop_map(|body| format!("#[cfg(test)] mod tests {{ use super::*; {body} }}")),
                    (Syntax::name(), Syntax::name(), members.clone())
                        .prop_map(|(ty, tr, body)| format!("impl {tr} for {ty} {{ {body} }}")),
                    (Syntax::name(), members).prop_map(|(name, body)| format!("pub trait {name}: Sized {{ {body} }}")),
                ]
            })
            .boxed()
    }

    fn file() -> BoxedStrategy<String> {
        vec(Syntax::item(), 0..ITEMS).prop_map(|items| items.join("\n")).boxed()
    }

    fn damaged(bytes: &[u8], place: Index, byte: u8, kind: u8) -> Vec<u8> {
        let at = place.index(bytes.len().saturating_add(1));
        let kept = bytes.iter().take(at).copied();
        match kind {
            0 => bytes.to_vec(),
            1 => kept.collect(),
            2 => kept
                .chain(bytes.get(at).map(|found| found ^ (byte | 1)))
                .chain(bytes.iter().skip(at.saturating_add(1)).copied())
                .collect(),
            _ => kept.chain([byte]).chain(bytes.iter().skip(at).copied()).collect(),
        }
    }
}
