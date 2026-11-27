use std::env;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use std::ops::Deref;

use file_doubles::Memory;
use file_ports::Bytes;
use newtype::{Arbitrary, Newtype};
use proptest::sample::Index;
use proptest::test_runner::Config;

#[derive(Debug)]
pub struct Campaign;

#[derive(Newtype, Arbitrary, Clone, Debug)]
#[newtype(strategy = proptest::collection::vec(proptest::arbitrary::any::<Of>(), 0..8))]
pub struct Few<Of: proptest::arbitrary::Arbitrary + fmt::Debug + 'static>(Vec<Of>);

impl<Of: proptest::arbitrary::Arbitrary + fmt::Debug + 'static> Deref for Few<Of> {
    type Target = [Of];

    fn deref(&self) -> &[Of] {
        &self.0
    }
}

#[derive(Arbitrary, Clone, Debug)]
pub enum Input {
    Bytes(Placed, Vec<u8>),
    Manifest(Vec<u8>),
    Damaged(Placed, Few<Item>, Index, u8, Damage),
    Deep(Placed, Shell, u8, Expression),
    Long(Placed, Link, u16),
}

#[derive(Arbitrary, Clone, Copy, Debug)]
pub enum Placed {
    Library,
    Binary,
    Test,
}

#[derive(Arbitrary, Clone, Copy, Debug)]
pub enum Damage {
    Kept,
    Cut,
    Flipped,
    Inserted,
}

#[derive(Arbitrary, Clone, Copy, Debug)]
pub enum Shell {
    Parenthesised,
    Braced,
    Bracketed,
    Negated,
    Borrowed,
    Matched,
}

#[derive(Arbitrary, Clone, Copy, Debug)]
pub enum Link {
    Method,
    Added,
    Tried,
    Cast,
    Field,
}

#[derive(Arbitrary, Clone, Copy, Debug)]
pub enum Leaf {
    Zero,
    Named,
    Text,
    Character,
    Truth,
    Unit,
    Nothing,
    Built,
    Started,
    Removed,
    Asked,
    Parsed,
    Ported,
}

#[derive(Arbitrary, Clone, Copy, Debug)]
pub enum Kind {
    Byte,
    Owned,
    Borrowed,
    Bytes,
    Maybe,
    Dynamic,
    Pair,
    Array,
    Flag,
    Fallible,
}

#[derive(Arbitrary, Clone, Copy, Debug)]
pub enum Wrapping {
    Listed,
    Mutable,
    Paired,
}

#[derive(Arbitrary, Clone, Debug)]
pub enum Ty {
    Wrapped(Kind, Few<Wrapping>),
}

#[derive(Arbitrary, Clone, Copy, Debug)]
pub enum Layer {
    Called(Leaf),
    Method(Leaf),
    Branched,
    Matched,
    Looped,
    Closed,
    Unsafe,
    Listed,
}

#[derive(Arbitrary, Clone, Debug)]
pub enum Expression {
    Layered(Leaf, Few<Layer>),
}

#[derive(Arbitrary, Clone, Debug)]
pub enum Statement {
    Said(Expression),
    Bound(Leaf, Expression),
}

#[derive(Arbitrary, Clone, Copy, Debug)]
pub enum Attribute {
    Bare,
    Test,
    Gated,
    Allowed,
    Derived,
    Used,
}

#[derive(Arbitrary, Clone, Copy, Debug)]
pub enum Visible {
    Private,
    Public,
    Crate,
}

#[derive(Arbitrary, Clone, Copy, Debug)]
pub enum Flat {
    Imported,
    Renamed,
    Globbed,
    Reexported,
    Commented,
    Documented,
    Inner,
    Macro,
    Union,
    Alias,
}

#[derive(Arbitrary, Clone, Debug)]
pub enum Declared {
    Function(Leaf, Ty, Ty, Few<Statement>),
    Structure(Leaf, Ty),
    Enumerated(Leaf, Leaf, Ty),
    Constant(Leaf, Ty, Expression),
    Flat(Flat),
}

#[derive(Arbitrary, Clone, Copy, Debug)]
pub enum Container {
    Module(Leaf),
    Tests,
    Implemented(Leaf, Leaf),
    Trait(Leaf),
}

#[derive(Arbitrary, Clone, Debug)]
pub enum Item {
    Placed(Attribute, Visible, Declared, Few<Container>),
}

impl Campaign {
    const ROOT: &'static str = "/fuzzed";
    const MANIFEST: &'static str = "[package]\nname = \"fuzzed\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";
    const SECONDS: &'static str = "FUZZ_SECONDS";
    const DEFAULT_SECONDS: u64 = 5;
    const BATCH: u32 = 64;

    #[must_use]
    pub fn root() -> PathBuf {
        PathBuf::from(Campaign::ROOT)
    }

    #[must_use]
    pub fn deadline() -> Instant {
        let given = env::var(Campaign::SECONDS).into_iter().find_map(|text| text.parse::<u64>().into_iter().next());
        Instant::now() + Duration::from_secs(given.unwrap_or(Campaign::DEFAULT_SECONDS))
    }

    #[must_use]
    pub fn batch() -> Config {
        Config { cases: Campaign::BATCH, failure_persistence: None, ..Config::default() }
    }
}

impl Input {
    #[must_use]
    pub fn files(&self) -> Memory {
        let root = Campaign::root();
        let manifest = match self {
            Input::Manifest(bytes) => Bytes::new(bytes.clone()),
            Input::Bytes(..) | Input::Damaged(..) | Input::Deep(..) | Input::Long(..) => {
                Bytes::from(Campaign::MANIFEST)
            }
        };
        let (placed, source) = match self {
            Input::Bytes(placed, bytes) => (*placed, bytes.clone()),
            Input::Manifest(_) => (Placed::Library, Vec::new()),
            Input::Damaged(placed, items, at, byte, damage) => {
                let text: String = items.iter().map(|item| [item.to_string(), "\n".to_owned()].concat()).collect();
                (*placed, damage.applied(text.as_bytes(), *at, *byte))
            }
            Input::Deep(placed, shell, depth, core) => {
                let [open, close] = shell.ends();
                let times = usize::from(*depth).saturating_add(1);
                (*placed, format!("fn deep() {{ {}{core}{} }}", open.repeat(times), close.repeat(times)).into_bytes())
            }
            Input::Long(placed, link, length) => {
                (*placed, format!("fn long() {{ x{} }}", link.spelled().repeat(usize::from(*length))).into_bytes())
            }
        };
        [(root.join("Cargo.toml"), manifest), (root.join(placed.path()), Bytes::new(source))].into_iter().collect()
    }
}

impl Placed {
    fn path(self) -> &'static Path {
        Path::new(match self {
            Placed::Library => "src/lib.rs",
            Placed::Binary => "src/main.rs",
            Placed::Test => "tests/fuzzed.rs",
        })
    }
}

impl Damage {
    fn applied(self, bytes: &[u8], place: Index, byte: u8) -> Vec<u8> {
        let at = place.index(bytes.len().saturating_add(1));
        let kept = bytes.iter().take(at).copied();
        match self {
            Damage::Kept => bytes.to_vec(),
            Damage::Cut => kept.collect(),
            Damage::Flipped => kept
                .chain(bytes.get(at).map(|found| found ^ (byte | 1)))
                .chain(bytes.iter().skip(at.saturating_add(1)).copied())
                .collect(),
            Damage::Inserted => kept.chain([byte]).chain(bytes.iter().skip(at).copied()).collect(),
        }
    }
}

impl Shell {
    fn ends(self) -> [&'static str; 2] {
        match self {
            Shell::Parenthesised => ["(", ")"],
            Shell::Braced => ["{ ", " }"],
            Shell::Bracketed => ["[", "]"],
            Shell::Negated => ["-", ""],
            Shell::Borrowed => ["&", ""],
            Shell::Matched => ["match x { _ => ", " }"],
        }
    }
}

impl Link {
    fn spelled(self) -> &'static str {
        match self {
            Link::Method => ".m()",
            Link::Added => " + 1",
            Link::Tried => "?",
            Link::Cast => " as u8",
            Link::Field => ".0",
        }
    }
}

impl Display for Leaf {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Leaf::Zero => "0",
            Leaf::Named => "x",
            Leaf::Text => "\"text\"",
            Leaf::Character => "'c'",
            Leaf::Truth => "true",
            Leaf::Unit => "()",
            Leaf::Nothing => "None",
            Leaf::Built => "Self::new()",
            Leaf::Started => "std::process::Command::new(\"ls\")",
            Leaf::Removed => "std::fs::remove_dir_all(\"p\")",
            Leaf::Asked => "is_ready",
            Leaf::Parsed => "from_str",
            Leaf::Ported => "ports",
        })
    }
}

impl Display for Ty {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let Ty::Wrapped(kind, wrappings) = self;
        let core = match kind {
            Kind::Byte => "u8",
            Kind::Owned => "String",
            Kind::Borrowed => "&'a str",
            Kind::Bytes => "Vec<u8>",
            Kind::Maybe => "Option<Foo>",
            Kind::Dynamic => "Box<dyn Bar>",
            Kind::Pair => "(u8, u8)",
            Kind::Array => "[u8; 4]",
            Kind::Flag => "bool",
            Kind::Fallible => "Result<(), Foo>",
        };
        let wrapped = wrappings.iter().fold(core.to_owned(), |inner, wrapping| match wrapping {
            Wrapping::Listed => format!("Vec<{inner}>"),
            Wrapping::Mutable => format!("&mut {inner}"),
            Wrapping::Paired => format!("({inner}, u8)"),
        });
        formatter.write_str(&wrapped)
    }
}

impl Display for Expression {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let Expression::Layered(core, layers) = self;
        let layered = layers.iter().fold(core.to_string(), |inner, layer| match layer {
            Layer::Called(name) => format!("{name}({inner})"),
            Layer::Method(name) => format!("{inner}.{name}()"),
            Layer::Branched => format!("if x {{ {inner} }} else {{ 0 }}"),
            Layer::Matched => format!("match x {{ Some(v) if v => {inner}, _ => 0 }}"),
            Layer::Looped => format!("for i in x {{ {inner}; }}"),
            Layer::Closed => format!("|v: u8| {{ let y = {inner}; y }}"),
            Layer::Unsafe => format!("unsafe {{ {inner}? }}"),
            Layer::Listed => format!("vec![{inner}]"),
        });
        formatter.write_str(&layered)
    }
}

impl Display for Statement {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Said(value) => write!(formatter, "{value};"),
            Statement::Bound(name, value) => write!(formatter, "let {name} = {value};"),
        }
    }
}

impl Display for Attribute {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Attribute::Bare => "",
            Attribute::Test => "#[test]",
            Attribute::Gated => "#[cfg(test)]",
            Attribute::Allowed => "#[allow(dead_code)]",
            Attribute::Derived => "#[derive(Debug)]",
            Attribute::Used => "#[must_use]",
        })
    }
}

impl Display for Visible {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Visible::Private => "",
            Visible::Public => "pub ",
            Visible::Crate => "pub(crate) ",
        })
    }
}

impl Display for Flat {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Flat::Imported => "use std::process::Command;",
            Flat::Renamed => "use std::fs::{self, remove_dir_all as gone};",
            Flat::Globbed => "use std::*;",
            Flat::Reexported => "pub use super::x::y;",
            Flat::Commented => "// note",
            Flat::Documented => "/// doc",
            Flat::Inner => "//! inner",
            Flat::Macro => "macro_rules! m { ($x:expr) => { $x + 1 }; }",
            Flat::Union => "union U { a: u8, b: u16 }",
            Flat::Alias => "type T<'a> = &'a u8;",
        })
    }
}

impl Display for Item {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let Item::Placed(attribute, visible, declared, containers) = self;
        let core = match declared {
            Declared::Function(name, taken, given, body) => {
                let statements: Vec<String> = body.iter().map(ToString::to_string).collect();
                format!("{attribute} {visible}fn {name}<'a>(a: {taken}) -> {given} {{ {} }}", statements.join(" "))
            }
            Declared::Structure(name, ty) => {
                format!("{attribute} {visible}struct {name}<'a> {{ {visible}field: {ty} }}")
            }
            Declared::Enumerated(name, variant, ty) => {
                format!("enum {name} {{ {variant}({ty}), Other {{ a: u8 }}, Unit }}")
            }
            Declared::Constant(name, ty, value) => format!("pub const {name}: {ty} = {value};"),
            Declared::Flat(flat) => flat.to_string(),
        };
        let contained = containers.iter().fold(core, |inner, container| match container {
            Container::Module(name) => format!("mod {name} {{ {inner} }}"),
            Container::Tests => format!("#[cfg(test)] mod tests {{ use super::*; {inner} }}"),
            Container::Implemented(ty, implemented) => format!("impl {implemented} for {ty} {{ {inner} }}"),
            Container::Trait(name) => format!("pub trait {name}: Sized {{ {inner} }}"),
        });
        formatter.write_str(&contained)
    }
}
