use proptest::collection::vec;
use proptest::prelude::{BoxedStrategy, Strategy, prop_oneof};
use proptest::sample::select;

use crate::shell::Shell;

const NAMES: [&str; 24] = [
    "new",
    "start",
    "stop",
    "is_ready",
    "has_items",
    "from_str",
    "fmt",
    "encode",
    "decode",
    "to_text",
    "from_text",
    "main",
    "tests",
    "ports",
    "bits",
    "Foo",
    "Bar",
    "Baz",
    "T",
    "x",
    "value",
    "run",
    "open",
    "close",
];
const TYPES: [&str; 14] = [
    "u8",
    "usize",
    "String",
    "&str",
    "&'a str",
    "Vec<u8>",
    "Option<Foo>",
    "Box<dyn Bar>",
    "(u8, u8)",
    "[u8; 4]",
    "fn(u8) -> u8",
    "bool",
    "*const u8",
    "Result<(), Foo>",
];
const LEAVES: [&str; 18] = [
    "0",
    "1u8",
    "0xff",
    "1.5",
    "\"text\"",
    "b\"bytes\"",
    "'c'",
    "true",
    "()",
    "x",
    "None",
    "Self::new()",
    "std::process::Command::new(\"ls\")",
    "std::fs::remove_dir_all(\"p\")",
    "std::mem::forget(x)",
    "std::env::temp_dir()",
    "x as u8",
    "1 + 2",
];
const MACROS: [&str; 7] = ["vec", "println", "assert_eq", "matches", "format", "roundtrip", "law"];
const ATTRIBUTES: [&str; 9] = [
    "",
    "",
    "#[derive(Debug)]",
    "#[test]",
    "#[ignore]",
    "#[cfg(test)]",
    "#[allow(dead_code)]",
    "#[must_use]",
    "#[derive(Clone, Copy)]",
];
const VISIBILITY: [&str; 4] = ["", "pub ", "pub(crate) ", "pub(super) "];
const GENERICS: [&str; 5] = ["", "<T>", "<'a>", "<'a, T: Bar>", "<const N: usize>"];
const USES: [&str; 8] = [
    "use std::process::Command;",
    "use std::fs::{self, remove_dir_all as gone};",
    "use std::*;",
    "use crate::Foo as Bar;",
    "pub use super::x::y;",
    "use ::std::mem;",
    "use self::{a, b::{c, d}};",
    "extern crate std;",
];
const COMMENTS: [&str; 4] = ["// note", "/* block */", "/// doc", "//! inner"];
const RAW: [&str; 4] = [
    "macro_rules! m { ($x:expr) => { $x + 1 }; }",
    "extern \"C\" { fn f(a: u8) -> u8; }",
    "union U { a: u8, b: u16 }",
    "const _: () = ();",
];
const SHELLS: [Shell; 8] = [
    Shell::new("(", ")"),
    Shell::new("{ ", " }"),
    Shell::new("[", "]"),
    Shell::new("-", ""),
    Shell::new("&", ""),
    Shell::new("!", ""),
    Shell::new("if x { ", " } else { 0 }"),
    Shell::new("match x { _ => ", " }"),
];
const LINKS: [&str; 6] = [".m()", " + 1", "?", " as u8", ".0", " && x"];
const MAX_CHAIN: usize = 30000;
const MAX_LIST: usize = 4;
const DEPTH: u32 = 4;
const SIZE: u32 = 48;
const BRANCH: u32 = 4;
const NO_ITEMS: usize = 0;
const MIN_NESTING: usize = 1;
const MAX_NESTING: usize = 140;
const MAX_ITEMS: usize = 10;

pub struct Syntax;

impl Syntax {
    pub fn file() -> impl Strategy<Value = String> {
        vec(Self::item(), NO_ITEMS..MAX_ITEMS).prop_map(|items| items.join("\n"))
    }

    pub fn nested() -> impl Strategy<Value = String> {
        (select(SHELLS.to_vec()), MIN_NESTING..MAX_NESTING, Self::expression())
            .prop_map(|(shell, depth, core)| format!("fn deep() {{ {} }}", shell.around(&core, depth)))
    }

    pub fn chained() -> impl Strategy<Value = String> {
        (select(LINKS.to_vec()), MIN_NESTING..MAX_CHAIN)
            .prop_map(|(link, length)| format!("fn long() {{ x{} }}", link.repeat(length)))
    }

    fn pick(table: &'static [&'static str]) -> BoxedStrategy<String> {
        select(table).prop_map(str::to_owned).boxed()
    }

    fn name() -> BoxedStrategy<String> {
        prop_oneof![Self::pick(&NAMES), "[a-z_][a-z0-9_]{0,8}".prop_map(|text| format!("{text}x"))].boxed()
    }

    fn list(each: BoxedStrategy<String>) -> BoxedStrategy<String> {
        vec(each, NO_ITEMS..MAX_LIST).prop_map(|parts| parts.join(", ")).boxed()
    }

    fn ty() -> BoxedStrategy<String> {
        Self::pick(&TYPES)
            .prop_recursive(DEPTH, SIZE, BRANCH, |inner| {
                prop_oneof![
                    inner.clone().prop_map(|ty| format!("Vec<{ty}>")),
                    inner.clone().prop_map(|ty| format!("&mut {ty}")),
                    inner.clone().prop_map(|ty| format!("Option<{ty}>")),
                    Self::list(inner).prop_map(|tys| format!("({tys})")),
                ]
            })
            .boxed()
    }

    fn expression() -> BoxedStrategy<String> {
        prop_oneof![Self::pick(&LEAVES), Self::name()]
            .prop_recursive(DEPTH, SIZE, BRANCH, |inner| {
                prop_oneof![
                    (Self::name(), Self::list(inner.clone())).prop_map(|(name, args)| format!("{name}({args})")),
                    (inner.clone(), Self::name(), Self::list(inner.clone()))
                        .prop_map(|(on, name, args)| format!("{on}.{name}({args})")),
                    (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("if {a} {{ {b} }} else {{ 0 }}")),
                    (inner.clone(), inner.clone())
                        .prop_map(|(a, b)| format!("match {a} {{ Some(v) if v => {b}, _ => 0 }}")),
                    (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("for i in {a} {{ {b}; }}")),
                    inner.clone().prop_map(|a| format!("loop {{ break {a}; }}")),
                    inner.clone().prop_map(|a| format!("|v: u8| {{ let y = {a}; y }}")),
                    inner.clone().prop_map(|a| format!("unsafe {{ {a} }}")),
                    inner.clone().prop_map(|a| format!("{a}?")),
                    inner.clone().prop_map(|a| format!("&mut *{a}")),
                    (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{a} + {b} * 2")),
                    (Self::pick(&MACROS), Self::list(inner.clone())).prop_map(|(mac, args)| format!("{mac}!({args})")),
                    (Self::name(), inner.clone()).prop_map(|(name, a)| format!("Foo {{ {name}: {a}, ..x }}")),
                    Self::list(inner).prop_map(|args| format!("[{args}]")),
                ]
            })
            .boxed()
    }

    fn body() -> BoxedStrategy<String> {
        let statement = prop_oneof![
            Self::expression().prop_map(|e| format!("{e};")),
            (Self::name(), Self::expression()).prop_map(|(name, e)| format!("let {name} = {e};")),
        ];
        vec(statement, NO_ITEMS..MAX_LIST).prop_map(|parts| parts.join(" ")).boxed()
    }

    fn function() -> BoxedStrategy<String> {
        (
            Self::pick(&ATTRIBUTES),
            Self::pick(&VISIBILITY),
            Self::name(),
            Self::pick(&GENERICS),
            Self::ty(),
            Self::ty(),
            Self::body(),
        )
            .prop_map(|(attribute, vis, name, generics, argument, returned, body)| {
                format!("{attribute} {vis}fn {name}{generics}(a: {argument}) -> {returned} {{ {body} }}")
            })
            .boxed()
    }

    fn declaration() -> BoxedStrategy<String> {
        prop_oneof![
            (Self::pick(&ATTRIBUTES), Self::pick(&VISIBILITY), Self::name(), Self::pick(&GENERICS), Self::ty())
                .prop_map(|(attribute, vis, name, generics, ty)| format!(
                    "{attribute} {vis}struct {name}{generics} {{ {vis}field: {ty} }}"
                )),
            (Self::pick(&ATTRIBUTES), Self::name(), Self::ty())
                .prop_map(|(attribute, name, ty)| format!("{attribute} pub struct {name}({ty}, u8);")),
            (Self::pick(&ATTRIBUTES), Self::name(), Self::name(), Self::ty()).prop_map(
                |(attribute, name, variant, ty)| format!(
                    "{attribute} enum {name} {{ {variant}({ty}), Other {{ a: u8 }}, Unit }}"
                )
            ),
            (Self::name(), Self::ty(), Self::expression())
                .prop_map(|(name, ty, e)| format!("pub const {name}: {ty} = {e};")),
            (Self::name(), Self::ty()).prop_map(|(name, ty)| format!("static mut {name}: {ty} = 0;")),
            (Self::name(), Self::ty()).prop_map(|(name, ty)| format!("type {name}<'a> = {ty};")),
        ]
        .boxed()
    }

    fn members() -> BoxedStrategy<String> {
        vec(Self::function(), NO_ITEMS..MAX_LIST).prop_map(|parts| parts.join("\n")).boxed()
    }

    fn container(items: BoxedStrategy<String>) -> BoxedStrategy<String> {
        let inner = vec(items, NO_ITEMS..MAX_LIST).prop_map(|parts| parts.join("\n"));
        prop_oneof![
            (Self::name(), inner.clone()).prop_map(|(name, body)| format!("mod {name} {{ {body} }}")),
            inner.prop_map(|body| format!("#[cfg(test)] mod tests {{ use super::*; {body} }}")),
            (Self::name(), Self::name(), Self::members())
                .prop_map(|(ty, tr, body)| format!("impl {tr} for {ty} {{ {body} }}")),
            (Self::name(), Self::members()).prop_map(|(ty, body)| format!("impl<T> {ty}<T> {{ {body} }}")),
            (Self::name(), Self::members()).prop_map(|(name, body)| format!("pub trait {name}: Sized {{ {body} }}")),
        ]
        .boxed()
    }

    fn item() -> BoxedStrategy<String> {
        let flat = prop_oneof![
            Self::function(),
            Self::declaration(),
            Self::pick(&USES),
            Self::pick(&COMMENTS),
            Self::pick(&RAW),
        ];
        flat.boxed().prop_recursive(DEPTH, SIZE, BRANCH, Self::container).boxed()
    }
}
