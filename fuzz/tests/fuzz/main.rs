mod campaign;
mod crash;
mod input;
mod pending;
mod shell;
mod syntax;

use campaign::Campaign;
use crash::Crash;
use input::{Input, Role};
use proptest::collection::vec;
use proptest::prelude::{Strategy, any};
use proptest::sample::{Index, select};
use syntax::Syntax;

const MAX_BYTES: usize = 512;
const NO_BYTES: usize = 0;
const TRUNCATE: u8 = 0;
const FLIP: u8 = 1;
const INSERT: u8 = 2;
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
                let mut bytes = text.into_bytes();
                let at = place.index(bytes.len().saturating_add(1));
                match kind {
                    TRUNCATE => bytes.truncate(at),
                    FLIP => {
                        if let Some(found) = bytes.get_mut(at) {
                            *found ^= byte | 1;
                        }
                    }
                    INSERT => bytes.insert(at, byte),
                    _ => {}
                }
                Input::new(role, bytes)
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
