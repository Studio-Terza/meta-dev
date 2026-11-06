use lint_fixtures::{Lines, Located, Source, Sources, Text};
use lint_port_dependencies::{CountedCalls, EffectsInFillers, OwnedDependencies};

const CRATES: &[Source] = &[
    Source::new(Located::new("crates/grammar/src/lib.rs"), Text::new("pub struct Lexer;\n")),
    Source::new(Located::new("crates/design/src/lib.rs"), Text::new("pub struct Slug;\n")),
    Source::new(Located::new("crates/store/src/lib.rs"), Text::new("pub struct Store;\n")),
];
const OWNER: Source = Source::new(
    Located::new("crates/grammar/Cargo.toml"),
    Text::new(
        "[package]\nname = \"grammar\"\n\n[package.metadata.port]\nowns = [\"logos\"]\n\n[dependencies]\nlogos = \"1\"\n",
    ),
);
const UNOWNED: Source = Source::new(
    Located::new("crates/grammar/Cargo.toml"),
    Text::new("[package]\nname = \"grammar\"\n\n[dependencies]\nlogos = \"1\"\n"),
);
const DEPENDS: Source = Source::new(
    Located::new("crates/design/Cargo.toml"),
    Text::new("[package]\nname = \"design\"\n\n[dependencies]\nlogos = \"1\"\n"),
);
const DEV_DEPENDS: Source = Source::new(
    Located::new("crates/design/Cargo.toml"),
    Text::new("[package]\nname = \"design\"\n\n[dev-dependencies]\nlogos.workspace = true\n"),
);
const RENAMED: Source = Source::new(
    Located::new("crates/design/Cargo.toml"),
    Text::new("[package]\nname = \"design\"\n\n[dependencies]\nlexer = { package = \"logos\", version = \"1\" }\n"),
);
const CO_OWNER: Source = Source::new(
    Located::new("crates/design/Cargo.toml"),
    Text::new(
        "[package]\nname = \"design\"\n\n[package.metadata.port]\nowns = [\"logos\"]\n\n[dependencies]\nlogos = \"1\"\n",
    ),
);
const THIRD: Source = Source::new(
    Located::new("crates/store/Cargo.toml"),
    Text::new("[package]\nname = \"store\"\n\n[dependencies]\nlogos = \"1\"\n"),
);
const ELSEWHERE: Source = Source::new(
    Located::new("crates/design/Cargo.toml"),
    Text::new("[package]\nname = \"design\"\n\n[dependencies]\nserde = \"1\"\n"),
);

const FILLED: &[Source] = &[Source::new(
    Located::new("crates/store-ports/src/lib.rs"),
    Text::new(
        "use port_core::port;\n#[port]\npub trait Store {\n    fn read(&self);\n}\npub struct Disk;\nimpl StoreFiller for Disk {\n    fn read(&self) {\n        std::fs::read(\"a\");\n    }\n}\n",
    ),
)];
const INHERENT: &[Source] = &[Source::new(
    Located::new("crates/store-ports/src/lib.rs"),
    Text::new(
        "#[port]\npub trait Store {\n    fn read(&self);\n}\npub struct Disk;\nimpl Disk {\n    fn open() {\n        std::fs::read(\"a\");\n    }\n}\n",
    ),
)];
const FREE: &[Source] = &[
    Source::new(Located::new("crates/store-ports/src/lib.rs"), Text::new("mod store;\nmod load;\n")),
    Source::new(
        Located::new("crates/store-ports/src/store.rs"),
        Text::new("#[port_core::port]\npub trait Store {\n    fn read(&self);\n}\n"),
    ),
    Source::new(
        Located::new("crates/store-ports/src/load.rs"),
        Text::new("fn load() {\n    std::env::var(\"A\");\n}\n"),
    ),
];
const OTHER_TRAIT: &[Source] = &[Source::new(
    Located::new("crates/store-ports/src/lib.rs"),
    Text::new(
        "#[port]\npub trait Store {\n    fn read(&self);\n}\npub struct Disk;\nimpl Default for Disk {\n    fn default() -> Disk {\n        std::fs::read(\"a\");\n        Disk\n    }\n}\n",
    ),
)];
const UNPORTED_FILLER: &[Source] = &[Source::new(
    Located::new("crates/store-ports/src/lib.rs"),
    Text::new(
        "#[port]\npub trait Store {\n    fn read(&self);\n}\n#[deprecated]\npub trait Cache {\n    fn read(&self);\n}\npub struct Disk;\nimpl CacheFiller for Disk {\n    fn read(&self) {\n        std::fs::read(\"a\");\n    }\n}\n",
    ),
)];
const RELEASED: &[Source] = &[Source::new(
    Located::new("crates/store-ports/src/lib.rs"),
    Text::new(
        "#[port]\npub trait Store {\n    fn lock(&self);\n}\npub struct Lock;\nimpl Release for Lock {\n    fn release(&mut self) {\n        std::fs::remove_file(\"a\");\n    }\n}\n",
    ),
)];
const QUALIFIED_RELEASE: &[Source] = &[Source::new(
    Located::new("crates/store-ports/src/lib.rs"),
    Text::new(
        "#[port]\npub trait Store {\n    fn lock(&self);\n}\npub struct Lock;\nimpl port_core::Release for Lock {\n    fn release(&mut self) {\n        std::fs::remove_file(\"a\");\n    }\n}\n",
    ),
)];
const RELEASE_OUTSIDE_PORTS: &[Source] = &[Source::new(
    Located::new("crates/design/src/lib.rs"),
    Text::new(
        "pub trait Store {\n    fn lock(&self);\n}\npub struct Lock;\nimpl Release for Lock {\n    fn release(&mut self) {\n        std::fs::remove_file(\"a\");\n    }\n}\n",
    ),
)];
const RELEASED_ELSEWHERE: &[Source] = &[Source::new(
    Located::new("crates/store-ports/src/lib.rs"),
    Text::new(
        "#[port]\npub trait Store {\n    fn lock(&self);\n}\npub struct Lock;\nimpl Released for Lock {\n    fn release(&mut self) {\n        std::fs::remove_file(\"a\");\n    }\n}\n",
    ),
)];
const IN_TESTS: &[Source] = &[Source::new(
    Located::new("crates/store-ports/src/lib.rs"),
    Text::new(
        "#[port]\npub trait Store {\n    fn read(&self);\n}\n#[cfg(test)]\nmod tests {\n    fn f() {\n        std::fs::read(\"a\");\n    }\n}\n",
    ),
)];
const NO_PORT: &[Source] = &[Source::new(
    Located::new("crates/store-ports/src/lib.rs"),
    Text::new(
        "pub trait Store {\n    fn read(&self);\n}\npub struct Disk;\nimpl Disk {\n    fn open() {\n        std::fs::read(\"a\");\n    }\n}\n",
    ),
)];

const IN_PORT_CORE: &[Source] =
    &[Source::new(Located::new("crates/port-core/src/lib.rs"), Text::new("pub fn counted_call() {}\n"))];
const IN_PORT_DERIVE: &[Source] = &[Source::new(
    Located::new("crates/port-derive/src/lib.rs"),
    Text::new("fn f() {\n    quote!(::port_core::counted_call());\n}\n"),
)];
const ACQUIRED_IN_PORT_CORE: &[Source] =
    &[Source::new(Located::new("crates/port-core/src/port.rs"), Text::new("pub fn counted_acquire() {}\n"))];
const ACQUIRED_ELSEWHERE: &[Source] = &[Source::new(
    Located::new("crates/store-ports/src/lib.rs"),
    Text::new("fn f(port: &Port) {\n    port.counted_acquire(|filler, pass| filler.lock(pass));\n}\n"),
)];
const IMPORTED: &[Source] =
    &[Source::new(Located::new("crates/design/src/lib.rs"), Text::new("use port_core::counted_call;\n"))];
const IN_MACRO: &[Source] = &[Source::new(
    Located::new("crates/design/tests/run.rs"),
    Text::new("fn f() {\n    vec![port_core::counted_call(a)];\n}\n"),
)];

struct Owning;

impl Owning {
    fn flagged(manifests: &[Source]) -> Lines {
        Sources::placed_flagged::<OwnedDependencies>(manifests, CRATES)
    }
}

laws::table!(
    an_owned_dependency_is_flagged_in_every_manifest_but_its_owners,
    Owning::flagged,
    [
        [OWNER, DEPENDS] => ["crates/design/Cargo.toml:5"],
        [OWNER, DEV_DEPENDS] => ["crates/design/Cargo.toml:5"],
        [OWNER, RENAMED] => ["crates/design/Cargo.toml:5"],
        [OWNER, ELSEWHERE] => Lines::default(),
        [UNOWNED, DEPENDS] => Lines::default(),
        [OWNER, CO_OWNER] => Lines::default(),
        [OWNER, CO_OWNER, THIRD] => ["crates/store/Cargo.toml:5"],
    ]
);

laws::table!(
    in_a_port_crate_effects_sit_only_in_fillers,
    Sources::flagged::<EffectsInFillers>,
    [
        FILLED => Lines::default(),
        INHERENT => ["crates/store-ports/src/lib.rs:8"],
        FREE => ["crates/store-ports/src/load.rs:2"],
        OTHER_TRAIT => ["crates/store-ports/src/lib.rs:8"],
        UNPORTED_FILLER => ["crates/store-ports/src/lib.rs:12"],
        RELEASED => Lines::default(),
        QUALIFIED_RELEASE => Lines::default(),
        RELEASED_ELSEWHERE => ["crates/store-ports/src/lib.rs:8"],
        RELEASE_OUTSIDE_PORTS => Lines::default(),
        IN_TESTS => Lines::default(),
        NO_PORT => Lines::default(),
    ]
);

laws::table!(
    counted_call_and_counted_acquire_are_named_only_in_port_core_and_port_derive,
    Sources::flagged::<CountedCalls>,
    [
        IN_PORT_CORE => Lines::default(),
        IN_PORT_DERIVE => Lines::default(),
        ACQUIRED_IN_PORT_CORE => Lines::default(),
        ACQUIRED_ELSEWHERE => ["crates/store-ports/src/lib.rs:2"],
        IMPORTED => ["crates/design/src/lib.rs:1"],
        IN_MACRO => ["crates/design/tests/run.rs:2"],
    ]
);
