use lint_fixtures::{Source, Sources};
use lint_raw_resources::RawResources;

const NONE: [&str; 0] = [];
const QUALIFIED: &str = "fn f() {\n    std::fs::remove_dir_all(\"x\");\n}\n";
const MODULE_IMPORT: &str = "use std::mem;\nfn f(x: u8) {\n    mem::forget(x);\n}\n";
const FUNCTION_IMPORT: &str = "use std::env::temp_dir;\nfn f() {\n    temp_dir();\n}\n";
const RENAMED: &str = "use std::mem::forget as leak;\nfn f(x: u8) {\n    leak(x);\n}\n";
const MODULE_RENAMED: &str = "use std::fs as disk;\nfn f() {\n    disk::remove_dir_all(\"x\");\n}\n";
const GROUPED: &str = "use std::{env, fs::{self, remove_dir_all}};\nfn f() {\n    env::temp_dir();\n    fs::remove_dir_all(\"x\");\n    remove_dir_all(\"x\");\n}\n";
const SELF_RENAMED: &str = "use std::fs::{self as disk};\nfn f() {\n    disk::remove_dir_all(\"x\");\n}\n";
const GLOB: &str = "use std::mem::*;\nfn f(x: u8) {\n    forget(x);\n}\n";
const AS_VALUE: &str = "fn f(x: Option<u8>) {\n    x.map(std::mem::forget);\n}\n";
const IN_MACRO: &str = "fn f() {\n    println!(\"{:?}\", std::env::temp_dir());\n}\n";
const IN_TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f(x: u8) {\n        std::mem::forget(x);\n    }\n}\n";
const OWN_FORGET: &str =
    "fn forget(x: u8) {}\nfn f(cache: Cache) {\n    forget(1);\n    cache.forget();\n    Cache::forget(1);\n}\n";
const OTHER_FS: &str = "use std::fs;\nfn f() {\n    fs::remove_dir(\"x\");\n    fs::read(\"x\");\n}\n";
const INLINE_PORTS: &str = "mod ports {\n    fn f(x: u8) {\n        std::mem::forget(x);\n    }\n}\n";
const INLINE_OTHER: &str = "mod adapters {\n    fn f(x: u8) {\n        std::mem::forget(x);\n    }\n}\n";

laws::table!(raw_calls_are_flagged, Sources::flagged::<RawResources>, [
    [Source::new("a.rs", QUALIFIED)] => ["a.rs:2"],
    [Source::new("a.rs", MODULE_IMPORT)] => ["a.rs:3"],
    [Source::new("a.rs", FUNCTION_IMPORT)] => ["a.rs:3"],
    [Source::new("a.rs", RENAMED)] => ["a.rs:3"],
    [Source::new("a.rs", MODULE_RENAMED)] => ["a.rs:3"],
    [Source::new("a.rs", GROUPED)] => ["a.rs:3", "a.rs:4", "a.rs:5"],
    [Source::new("a.rs", SELF_RENAMED)] => ["a.rs:3"],
    [Source::new("a.rs", GLOB)] => ["a.rs:3"],
    [Source::new("a.rs", AS_VALUE)] => ["a.rs:2"],
    [Source::new("a.rs", IN_MACRO)] => ["a.rs:2"],
    [Source::new("a.rs", IN_TEST_MODULE)] => ["a.rs:4"],
    [Source::new("x/tests/run.rs", QUALIFIED)] => ["x/tests/run.rs:2"],
    [Source::new("a.rs", INLINE_OTHER)] => ["a.rs:3"],
]);

laws::table!(other_calls_and_ports_are_not, Sources::flagged::<RawResources>, [
    [Source::new("a.rs", OWN_FORGET)] => NONE,
    [Source::new("a.rs", OTHER_FS)] => NONE,
    [Source::new("a.rs", INLINE_PORTS)] => NONE,
    [Source::new("crates/x/src/ports/run.rs", QUALIFIED)] => NONE,
    [Source::new("crates/x/src/ports.rs", QUALIFIED)] => NONE,
]);

#[test]
fn the_message_names_the_call_and_the_guard() {
    let shown = Sources::report::<RawResources>(&[Source::new("a.rs", QUALIFIED)]).to_string();
    assert!(["std::fs::remove_dir_all", "guard", "port"].iter().all(|word| shown.contains(word)));
}
