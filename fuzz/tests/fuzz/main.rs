mod crash;
mod input;
mod outcome;
mod ports;
mod shell;
mod syntax;

use crash::Crash;
use input::{Input, Role};
use ports::campaign::Campaign;
use proptest::collection::vec;
use proptest::prelude::{Strategy, any};
use proptest::sample::{Index, select};
use syntax::Syntax;

const MAX_BYTES: usize = 512;
const NO_BYTES: usize = 0;
const MUTATIONS: u8 = 3;

struct Targets;

impl Targets {
    fn roles() -> impl Strategy<Value = Role> {
        select(Role::SOURCES.to_vec())
    }

    fn bytes() -> impl Strategy<Value = Input> {
        (Self::roles(), vec(any::<u8>(), NO_BYTES..MAX_BYTES)).prop_map(|(role, bytes)| Input::new(role, bytes))
    }

    fn manifests() -> impl Strategy<Value = Input> {
        vec(any::<u8>(), NO_BYTES..MAX_BYTES).prop_map(|bytes| Input::new(Role::Manifest, bytes))
    }

    fn syntax() -> impl Strategy<Value = Input> {
        (Self::roles(), Syntax::file()).prop_map(|(role, text)| Input::new(role, text.into_bytes()))
    }

    fn nested() -> impl Strategy<Value = Input> {
        (Self::roles(), Syntax::nested()).prop_map(|(role, text)| Input::new(role, text.into_bytes()))
    }

    fn chained() -> impl Strategy<Value = Input> {
        (Self::roles(), Syntax::chained()).prop_map(|(role, text)| Input::new(role, text.into_bytes()))
    }

    fn damaged() -> impl Strategy<Value = Input> {
        (Self::roles(), Syntax::file(), any::<Index>(), any::<u8>(), 0..MUTATIONS).prop_map(
            |(role, text, place, byte, kind)| {
                let bytes = text.into_bytes();
                let at = place.index(bytes.len().saturating_add(1));
                let mutated = Mutation::ALL
                    .iter()
                    .skip(usize::from(kind))
                    .take(1)
                    .fold(bytes, |bytes, mutation| mutation.applied(&bytes, at, byte));
                Input::new(role, mutated)
            },
        )
    }
}

#[test]
fn arbitrary_bytes_in_a_source_file_do_not_panic_the_runner() -> Result<(), Crash> {
    Campaign::run("bytes", Targets::bytes())
}

#[test]
fn arbitrary_bytes_in_a_manifest_do_not_panic_the_runner() -> Result<(), Crash> {
    Campaign::run("manifest", Targets::manifests())
}

#[test]
fn arbitrary_valid_syntax_does_not_panic_the_runner() -> Result<(), Crash> {
    Campaign::run("syntax", Targets::syntax())
}

#[test]
fn damaged_valid_syntax_does_not_panic_the_runner() -> Result<(), Crash> {
    Campaign::run("damaged", Targets::damaged())
}

#[test]
fn deeply_nested_syntax_does_not_panic_the_runner() -> Result<(), Crash> {
    Campaign::run("nested", Targets::nested())
}

#[test]
fn long_chains_do_not_panic_the_runner() -> Result<(), Crash> {
    Campaign::run("chained", Targets::chained())
}

#[derive(Clone, Copy)]
enum Mutation {
    Truncate,
    Flip,
    Insert,
}

impl Mutation {
    const ALL: [Mutation; 3] = [Mutation::Truncate, Mutation::Flip, Mutation::Insert];

    fn applied(self, bytes: &[u8], at: usize, byte: u8) -> Vec<u8> {
        match self {
            Mutation::Truncate => bytes.iter().take(at).copied().collect(),
            Mutation::Flip => Self::replaced(bytes, at, bytes.get(at).map(|found| found ^ (byte | 1)), 1),
            Mutation::Insert => Self::replaced(bytes, at, Some(byte), 0),
        }
    }

    fn replaced(bytes: &[u8], at: usize, put: Option<u8>, removed: usize) -> Vec<u8> {
        bytes
            .iter()
            .take(at)
            .copied()
            .chain(put)
            .chain(bytes.iter().skip(at.saturating_add(removed)).copied())
            .collect()
    }
}
