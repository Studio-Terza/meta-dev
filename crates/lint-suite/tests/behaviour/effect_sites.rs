use lint_core::Lint;
use lint_effect_sites::{EffectSites, EffectsInPorts};
use lint_fixtures::{Located, Source, Sources, Text};

use crate::shown::{Presence, Shown};

struct Effects;

impl Effects {
    fn of(sources: &[Source]) -> String {
        EffectSites::report(&Sources::parsed(sources)).to_string()
    }

    fn at(path: &'static str, text: &str) -> Source {
        Source::new(Located::new(path), Text::new(Box::leak(text.to_owned().into_boxed_str())))
    }
}

const EVERY_KIND: &str = "use std::{env, fs, io, process, thread};\nuse std::time::{Instant, SystemTime};\nfn f() {\n    fs::read_to_string(\"a\");\n    std::fs::File::open(\"a\");\n    env::var(\"A\");\n    SystemTime::now();\n    Instant::now();\n    thread::sleep(d);\n    std::net::TcpStream::connect(\"a\");\n    process::Command::new(\"ls\");\n    rand::random();\n    io::stdout();\n    println!(\"a\");\n}\n";
const IN_PORTS: &str = "mod ports {\n    pub fn f() {\n        std::fs::write(\"a\", \"b\");\n    }\n}\nfn g() {\n    std::fs::write(\"a\", \"b\");\n}\n";
const IN_TESTS: &str = "fn f() {}\n#[cfg(test)]\nmod tests {\n    fn g() {\n        println!(\"a\");\n        std::env::var(\"A\");\n    }\n}\n#[cfg(test)]\nfn h() {\n    std::fs::read(\"a\");\n}\nstruct S;\n#[cfg(test)]\nimpl S {\n    fn i() {\n        std::env::args();\n    }\n}\nimpl S {\n    #[cfg(test)]\n    fn j() {\n        std::env::args();\n    }\n}\n";
const PRINTED_IN_MODULES: &str = "mod a {\n    fn f() {\n        println!(\"a\");\n    }\n}\nmod ports {\n    fn g() {\n        eprintln!(\"b\");\n    }\n}\n";
const FOREIGN: &str =
    "use clap::Command;\nfn f() {\n    Command::new(\"ls\");\n    cache.read();\n    Self::open();\n}\n";

laws::table!(
    effect_sites_are_counted_per_crate_and_listed_outside_ports,
    Shown::of::<EffectSites>,
    [
        [EVERY_KIND, "crate x: 11 effect sites, 0 in ports, 11 outside ports (files 2, environment 1, clock 3, network 1, processes 1, randomness 1, stdio 2)"] => Presence::Present,
        [EVERY_KIND, "crates/x/src/lib.rs:4: files effect outside ports: calls std::fs::read_to_string"] => Presence::Present,
        [EVERY_KIND, "crates/x/src/lib.rs:14: stdio effect outside ports: calls println!"] => Presence::Present,
        [IN_PORTS, "crate x: 2 effect sites, 1 in ports, 1 outside ports (files 1)"] => Presence::Present,
        [IN_PORTS, "lib.rs:3: files effect"] => Presence::Absent,
        [IN_PORTS, "lib.rs:7: files effect outside ports: calls std::fs::write"] => Presence::Present,
        [PRINTED_IN_MODULES, "crate x: 2 effect sites, 1 in ports, 1 outside ports (stdio 1)"] => Presence::Present,
        [PRINTED_IN_MODULES, "lib.rs:3: stdio effect outside ports: calls println!"] => Presence::Present,
        [IN_TESTS, "crate x"] => Presence::Absent,
        [FOREIGN, "crate x"] => Presence::Absent,
    ]
);

#[test]
fn a_ports_module_file_counts_its_effects_without_listing_them() {
    let text = Effects::of(&[Effects::at("crates/x/src/ports/run.rs", "fn f() {\n    std::env::var(\"A\");\n}\n")]);
    assert!(text.contains("crate x: 1 effect sites, 1 in ports, 0 outside ports;"), "{text}");
    assert_eq!(text.matches("environment effect").count(), 0, "{text}");
}

#[test]
fn tests_and_examples_are_not_read() {
    let text = Effects::of(&[
        Effects::at("crates/x/tests/run.rs", "fn f() {\n    std::env::var(\"A\");\n}\n"),
        Effects::at("crates/x/examples/e.rs", "fn f() {\n    println!(\"a\");\n}\n"),
    ]);
    assert_eq!(text.matches("crate x").count(), 0, "{text}");
}

#[test]
fn the_gate_fails_only_the_effects_outside_ports() {
    let text = EffectsInPorts::report(&Sources::parsed(&[Effects::at("crates/x/src/lib.rs", IN_PORTS)])).to_string();
    assert!(text.contains("lib.rs:7: files effect outside ports: calls std::fs::write"), "{text}");
    assert_eq!(text.matches("effect outside ports").count(), 1, "{text}");
    assert_eq!(text.matches("crate x").count(), 0, "{text}");
}
