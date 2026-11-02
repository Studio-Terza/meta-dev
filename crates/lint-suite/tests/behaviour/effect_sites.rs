use lint_effect_sites::EffectSites;
use lint_fixtures::{Located, Source, Sources, Text};

struct Effects;

impl Effects {
    fn of(sources: &[Source]) -> String {
        Sources::report::<EffectSites>(sources).to_string()
    }

    fn at(path: &'static str, text: &str) -> Source {
        Source::new(Located::new(path), Text::new(Box::leak(text.to_owned().into_boxed_str())))
    }

    fn shows(pair: &[&str]) -> bool {
        match pair {
            [text, expected] => Self::of(&[Self::at("crates/x/src/lib.rs", text)]).contains(expected),
            _ => false,
        }
    }
}

const EVERY_KIND: &str = "use std::{env, fs, io, process, thread};\nuse std::time::{Instant, SystemTime};\nfn f() {\n    fs::read_to_string(\"a\");\n    std::fs::File::open(\"a\");\n    env::var(\"A\");\n    SystemTime::now();\n    Instant::now();\n    thread::sleep(d);\n    std::net::TcpStream::connect(\"a\");\n    process::Command::new(\"ls\");\n    rand::random();\n    io::stdout();\n    println!(\"a\");\n}\n";
const IN_PORTS: &str = "mod ports {\n    pub fn f() {\n        std::fs::write(\"a\", \"b\");\n    }\n}\nfn g() {\n    std::fs::write(\"a\", \"b\");\n}\n";
const IN_TESTS: &str = "fn f() {}\n#[cfg(test)]\nmod tests {\n    fn g() {\n        println!(\"a\");\n    }\n}\n";
const FOREIGN: &str =
    "use clap::Command;\nfn f() {\n    Command::new(\"ls\");\n    cache.read();\n    Self::open();\n}\n";

laws::table!(
    effect_sites_are_counted_per_crate_and_listed_outside_ports,
    Effects::shows,
    [
        [EVERY_KIND, "crate x: 11 effect sites, 0 in ports, 11 outside ports (files 2, environment 1, clock 3, network 1, processes 1, randomness 1, stdio 2)"] => true,
        [EVERY_KIND, "crates/x/src/lib.rs:4: files effect outside ports: calls std::fs::read_to_string"] => true,
        [EVERY_KIND, "crates/x/src/lib.rs:14: stdio effect outside ports: calls println!"] => true,
        [IN_PORTS, "crate x: 2 effect sites, 1 in ports, 1 outside ports (files 1)"] => true,
        [IN_PORTS, "lib.rs:3: files effect"] => false,
        [IN_PORTS, "lib.rs:7: files effect outside ports: calls std::fs::write"] => true,
        [IN_TESTS, "crate x"] => false,
        [FOREIGN, "crate x"] => false,
    ]
);

#[test]
fn a_ports_module_file_counts_its_effects_without_listing_them() {
    let text = Effects::of(&[Effects::at("crates/x/src/ports/run.rs", "fn f() {\n    std::env::var(\"A\");\n}\n")]);
    assert!(text.contains("crate x: 1 effect sites, 1 in ports, 0 outside ports\n"), "{text}");
    assert!(!text.contains("environment effect"), "{text}");
}

#[test]
fn tests_and_examples_are_not_read() {
    let text = Effects::of(&[
        Effects::at("crates/x/tests/run.rs", "fn f() {\n    std::env::var(\"A\");\n}\n"),
        Effects::at("crates/x/examples/e.rs", "fn f() {\n    println!(\"a\");\n}\n"),
    ]);
    assert!(!text.contains("crate x"), "{text}");
}
