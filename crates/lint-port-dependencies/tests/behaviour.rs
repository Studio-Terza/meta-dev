use lint_fixtures::{Lines, Located, Shown, Source, Sources, Text};
use lint_port_dependencies::{CountedCalls, EffectsInFillers, HiddenSqlx, KeyDependencyRoles, OwnedDependencies};

const UNIT_CRATES: &[Source] = &[
    Source::new(Located::new("crates/store-ports/src/lib.rs"), Text::new("pub struct Store;\n")),
    Source::new(Located::new("crates/store-ports-macros/src/lib.rs"), Text::new("pub struct Query;\n")),
    Source::new(Located::new("crates/design-store/src/lib.rs"), Text::new("pub struct Designs;\n")),
];
const PORT: Source = Source::new(
    Located::new("crates/store-ports/Cargo.toml"),
    Text::new(
        "[package]\nname = \"store-ports\"\n\n[package.metadata.port]\nowns = [\"sqlx\"]\nshared = [\"serde_json\"]\n\n[dependencies]\nsqlx = \"0.9\"\nstore-ports-macros = { path = \"../store-ports-macros\" }\n",
    ),
);
const PORT_ON_CLOSURE: Source = Source::new(
    Located::new("crates/store-ports/Cargo.toml"),
    Text::new(
        "[package]\nname = \"store-ports\"\n\n[package.metadata.port]\nowns = [\"sqlx\"]\nshared = [\"serde_json\"]\n\n[dependencies]\nlibsqlite3-sys = \"0.37\"\nstore-ports-macros = { path = \"../store-ports-macros\" }\n",
    ),
);
const MACROS: Source = Source::new(
    Located::new("crates/store-ports-macros/Cargo.toml"),
    Text::new(
        "[package]\nname = \"store-ports-macros\"\n\n[package.metadata.port]\nowns = [\"sqlx-macros-core\"]\n\n[dependencies]\nsqlx-macros-core = \"0.9\"\nsqlx-core = \"0.9\"\n",
    ),
);
const ON_CLOSURE: Source = Source::new(
    Located::new("crates/design-store/Cargo.toml"),
    Text::new("[package]\nname = \"design-store\"\n\n[dependencies]\nlibsqlite3-sys = \"0.37\"\n"),
);
const ON_OWNED: Source = Source::new(
    Located::new("crates/design-store/Cargo.toml"),
    Text::new("[package]\nname = \"design-store\"\n\n[dependencies]\nsqlx = \"0.9\"\n"),
);
const DEV_ON_OWNED: Source = Source::new(
    Located::new("crates/design-store/Cargo.toml"),
    Text::new("[package]\nname = \"design-store\"\n\n[dev-dependencies]\nsqlx.workspace = true\n"),
);
const RENAMED: Source = Source::new(
    Located::new("crates/design-store/Cargo.toml"),
    Text::new(
        "[package]\nname = \"design-store\"\n\n[dependencies]\ndatabase = { package = \"sqlx\", version = \"0.9\" }\n",
    ),
);
const WORKSPACE_RENAMED: Source = Source::new(
    Located::new("crates/design-store/Cargo.toml"),
    Text::new("[package]\nname = \"design-store\"\n\n[dependencies]\ndatabase.workspace = true\n"),
);
const ON_MACRO_CLOSURE: Source = Source::new(
    Located::new("crates/design-store/Cargo.toml"),
    Text::new("[package]\nname = \"design-store\"\n\n[dependencies]\nsqlx-core = \"0.9\"\n"),
);
const ON_SHARED: Source = Source::new(
    Located::new("crates/design-store/Cargo.toml"),
    Text::new("[package]\nname = \"design-store\"\n\n[dependencies]\nserde_json = \"1\"\n"),
);
const ON_UNOWNED: Source = Source::new(
    Located::new("crates/design-store/Cargo.toml"),
    Text::new("[package]\nname = \"design-store\"\n\n[dependencies]\nlogos = \"0.16\"\n"),
);

const SHARED_MACRO_CRATES: &[Source] = &[
    Source::new(Located::new("crates/alpha/src/lib.rs"), Text::new("pub struct Alpha;\n")),
    Source::new(Located::new("crates/beta/src/lib.rs"), Text::new("pub struct Beta;\n")),
    Source::new(Located::new("crates/shared-macros/src/lib.rs"), Text::new("pub struct Derived;\n")),
    Source::new(Located::new("crates/gamma/src/lib.rs"), Text::new("pub struct Gamma;\n")),
];
const ALPHA: Source = Source::new(
    Located::new("crates/alpha/Cargo.toml"),
    Text::new(
        "[package]\nname = \"alpha\"\n\n[package.metadata.port]\nowns = []\n\n[dependencies]\nshared-macros = { path = \"../shared-macros\" }\next = { path = \"../ext\" }\n",
    ),
);
const BETA: Source = Source::new(
    Located::new("crates/beta/Cargo.toml"),
    Text::new(
        "[package]\nname = \"beta\"\n\n[package.metadata.port]\nowns = []\n\n[dependencies]\nshared-macros = { path = \"../shared-macros\" }\n",
    ),
);
const SHARED_MACROS: Source = Source::new(
    Located::new("crates/shared-macros/Cargo.toml"),
    Text::new(
        "[package]\nname = \"shared-macros\"\n\n[package.metadata.port]\nowns = [\"ext\"]\n\n[dependencies]\next = { path = \"../ext\" }\n",
    ),
);
const GAMMA: Source = Source::new(
    Located::new("crates/gamma/Cargo.toml"),
    Text::new("[package]\nname = \"gamma\"\n\n[dependencies]\next = { path = \"../ext\" }\n"),
);

const CROSS_UNIT_CRATES: &[Source] = &[
    Source::new(Located::new("crates/alpha/src/lib.rs"), Text::new("pub struct Alpha;\n")),
    Source::new(Located::new("crates/beta/src/lib.rs"), Text::new("pub struct Beta;\n")),
    Source::new(Located::new("crates/gamma/src/lib.rs"), Text::new("pub struct Gamma;\n")),
];
const ALPHA_ON_X: Source = Source::new(
    Located::new("crates/alpha/Cargo.toml"),
    Text::new("[package]\nname = \"alpha\"\n\n[package.metadata.port]\nowns = [\"x\"]\n\n[dependencies]\nx = \"1\"\n"),
);
const ALPHA_ON_X_AND_Y: Source = Source::new(
    Located::new("crates/alpha/Cargo.toml"),
    Text::new(
        "[package]\nname = \"alpha\"\n\n[package.metadata.port]\nowns = [\"x\"]\n\n[dependencies]\nx = \"1\"\ny = \"1\"\n",
    ),
);
const BETA_ON_Y: Source = Source::new(
    Located::new("crates/beta/Cargo.toml"),
    Text::new("[package]\nname = \"beta\"\n\n[package.metadata.port]\nowns = [\"y\"]\n\n[dependencies]\ny = \"1\"\n"),
);
const GAMMA_ON_Y: Source = Source::new(
    Located::new("crates/gamma/Cargo.toml"),
    Text::new("[package]\nname = \"gamma\"\n\n[dependencies]\ny = \"1\"\n"),
);
const ALPHA_OWNS_UNREACHED: Source = Source::new(
    Located::new("crates/alpha/Cargo.toml"),
    Text::new("[package]\nname = \"alpha\"\n\n[package.metadata.port]\nowns = [\"y\"]\n"),
);

macro_rules! store {
    ($declared:literal, $required:literal) => {
        Text::new(concat!(
            "{\"workspace_root\": \"/w\", \"workspace_members\": [\"store-ports\", \"store-ports-macros\", \"design-store\"], \"packages\": [\
     {\"id\": \"store-ports\", \"name\": \"store-ports\", \"manifest_path\": \"/w/crates/store-ports/Cargo.toml\", \"dependencies\": [], \"metadata\": {\"port\": {\"owns\": [\"sqlx\"], \"shared\": [\"serde_json\"]}}, \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/store-ports/src/lib.rs\"}]},\
     {\"id\": \"store-ports-macros\", \"name\": \"store-ports-macros\", \"manifest_path\": \"/w/crates/store-ports-macros/Cargo.toml\", \"dependencies\": [], \"metadata\": {\"port\": {\"owns\": [\"sqlx-macros-core\"]}}, \"targets\": [{\"kind\": [\"proc-macro\"], \"src_path\": \"/w/crates/store-ports-macros/src/lib.rs\"}]},\
     {\"id\": \"design-store\", \"name\": \"design-store\", \"manifest_path\": \"/w/crates/design-store/Cargo.toml\", \"dependencies\": [", $declared, "], \"metadata\": null, \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/design-store/src/lib.rs\"}]},\
     {\"id\": \"sqlx\", \"name\": \"sqlx\", \"manifest_path\": \"/r/sqlx/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"sqlx-core\", \"name\": \"sqlx-core\", \"manifest_path\": \"/r/sqlx-core/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"sqlx-macros-core\", \"name\": \"sqlx-macros-core\", \"manifest_path\": \"/r/sqlx-macros-core/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"libsqlite3-sys\", \"name\": \"libsqlite3-sys\", \"manifest_path\": \"/r/libsqlite3-sys/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"serde_json\", \"name\": \"serde_json\", \"manifest_path\": \"/r/serde_json/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"logos\", \"name\": \"logos\", \"manifest_path\": \"/r/logos/Cargo.toml\", \"dependencies\": []}\
     ], \"resolve\": {\"nodes\": [\
     {\"id\": \"store-ports\", \"dependencies\": [\"sqlx\", \"libsqlite3-sys\", \"store-ports-macros\"]},\
     {\"id\": \"store-ports-macros\", \"dependencies\": [\"sqlx-macros-core\", \"sqlx-core\"]},\
     {\"id\": \"design-store\", \"dependencies\": [", $required, "]},\
     {\"id\": \"sqlx\", \"dependencies\": [\"sqlx-core\", \"libsqlite3-sys\"]},\
     {\"id\": \"sqlx-core\", \"dependencies\": [\"serde_json\"]},\
     {\"id\": \"sqlx-macros-core\", \"dependencies\": [\"sqlx-core\"]}\
     ]}}"
        ))
    };
}

const STORE: Text = store!("", "");
const STORE_ON_SQLX: Text = store!("{\"name\": \"sqlx\"}", "\"sqlx\"");
const STORE_ON_DATABASE: Text = store!("{\"name\": \"sqlx\", \"rename\": \"database\"}", "\"sqlx\"");
const STORE_UNDECLARED: Text = store!("", "\"sqlx\"");
const STORE_ON_LIBSQLITE: Text = store!("{\"name\": \"libsqlite3-sys\"}", "\"libsqlite3-sys\"");
const STORE_ON_SQLX_CORE: Text = store!("{\"name\": \"sqlx-core\"}", "\"sqlx-core\"");
const STORE_ON_SERDE_JSON: Text = store!("{\"name\": \"serde_json\"}", "\"serde_json\"");
const STORE_ON_LOGOS: Text = store!("{\"name\": \"logos\"}", "\"logos\"");
const STORE_WITHOUT_MACROS: Text = Text::new(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"store-ports\", \"store-ports-macros\", \"design-store\"], \"packages\": [\
     {\"id\": \"store-ports\", \"name\": \"store-ports\", \"manifest_path\": \"/w/crates/store-ports/Cargo.toml\", \"dependencies\": [], \"metadata\": {\"port\": {\"owns\": [\"sqlx\"]}}, \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/store-ports/src/lib.rs\"}]},\
     {\"id\": \"store-ports-macros\", \"name\": \"store-ports-macros\", \"manifest_path\": \"/w/crates/store-ports-macros/Cargo.toml\", \"dependencies\": [{\"name\": \"sqlx-core\"}], \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/store-ports-macros/src/lib.rs\"}]},\
     {\"id\": \"design-store\", \"name\": \"design-store\", \"manifest_path\": \"/w/crates/design-store/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/design-store/src/lib.rs\"}]},\
     {\"id\": \"sqlx\", \"name\": \"sqlx\", \"manifest_path\": \"/r/sqlx/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"sqlx-core\", \"name\": \"sqlx-core\", \"manifest_path\": \"/r/sqlx-core/Cargo.toml\", \"dependencies\": []}\
     ], \"resolve\": {\"nodes\": [\
     {\"id\": \"store-ports\", \"dependencies\": [\"sqlx\", \"store-ports-macros\"]},\
     {\"id\": \"store-ports-macros\", \"dependencies\": [\"sqlx-core\"]},\
     {\"id\": \"sqlx\", \"dependencies\": [\"sqlx-core\"]}\
     ]}}",
);
const SHARED_MACRO: Text = Text::new(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"alpha\", \"beta\", \"shared-macros\", \"ext\", \"gamma\"], \"packages\": [\
     {\"id\": \"alpha\", \"name\": \"alpha\", \"manifest_path\": \"/w/crates/alpha/Cargo.toml\", \"dependencies\": [{\"name\": \"shared-macros\"}, {\"name\": \"ext\"}], \"metadata\": {\"port\": {\"owns\": []}}, \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/alpha/src/lib.rs\"}]},\
     {\"id\": \"beta\", \"name\": \"beta\", \"manifest_path\": \"/w/crates/beta/Cargo.toml\", \"dependencies\": [{\"name\": \"shared-macros\"}], \"metadata\": {\"port\": {\"owns\": []}}, \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/beta/src/lib.rs\"}]},\
     {\"id\": \"shared-macros\", \"name\": \"shared-macros\", \"manifest_path\": \"/w/crates/shared-macros/Cargo.toml\", \"dependencies\": [{\"name\": \"ext\"}], \"metadata\": {\"port\": {\"owns\": [\"ext\"]}}, \"targets\": [{\"kind\": [\"proc-macro\"], \"src_path\": \"/w/crates/shared-macros/src/lib.rs\"}]},\
     {\"id\": \"ext\", \"name\": \"ext\", \"manifest_path\": \"/w/crates/ext/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/ext/src/lib.rs\"}]},\
     {\"id\": \"gamma\", \"name\": \"gamma\", \"manifest_path\": \"/w/crates/gamma/Cargo.toml\", \"dependencies\": [{\"name\": \"ext\"}], \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/gamma/src/lib.rs\"}]}\
     ], \"resolve\": {\"nodes\": [\
     {\"id\": \"alpha\", \"dependencies\": [\"shared-macros\", \"ext\"]},\
     {\"id\": \"beta\", \"dependencies\": [\"shared-macros\"]},\
     {\"id\": \"shared-macros\", \"dependencies\": [\"ext\"]},\
     {\"id\": \"gamma\", \"dependencies\": [\"ext\"]}\
     ]}}",
);

const CROSS_UNIT: Text = Text::new(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"alpha\", \"beta\"], \"packages\": [\
     {\"id\": \"alpha\", \"name\": \"alpha\", \"manifest_path\": \"/w/crates/alpha/Cargo.toml\", \"dependencies\": [{\"name\": \"x\"}, {\"name\": \"y\"}], \"metadata\": {\"port\": {\"owns\": [\"x\"]}}, \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/alpha/src/lib.rs\"}]},\
     {\"id\": \"beta\", \"name\": \"beta\", \"manifest_path\": \"/w/crates/beta/Cargo.toml\", \"dependencies\": [{\"name\": \"y\"}], \"metadata\": {\"port\": {\"owns\": [\"y\"]}}, \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/beta/src/lib.rs\"}]},\
     {\"id\": \"x\", \"name\": \"x\", \"manifest_path\": \"/r/x/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"y\", \"name\": \"y\", \"manifest_path\": \"/r/y/Cargo.toml\", \"dependencies\": []}\
     ], \"resolve\": {\"nodes\": [\
     {\"id\": \"alpha\", \"dependencies\": [\"x\", \"y\"]},\
     {\"id\": \"beta\", \"dependencies\": [\"y\"]},\
     {\"id\": \"x\", \"dependencies\": [\"y\"]}\
     ]}}",
);
const OWNER_NAMED: Text = Text::new(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"alpha\", \"beta\", \"gamma\"], \"packages\": [\
     {\"id\": \"alpha\", \"name\": \"alpha\", \"manifest_path\": \"/w/crates/alpha/Cargo.toml\", \"dependencies\": [{\"name\": \"x\"}], \"metadata\": {\"port\": {\"owns\": [\"x\"]}}, \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/alpha/src/lib.rs\"}]},\
     {\"id\": \"beta\", \"name\": \"beta\", \"manifest_path\": \"/w/crates/beta/Cargo.toml\", \"dependencies\": [{\"name\": \"y\"}], \"metadata\": {\"port\": {\"owns\": [\"y\"]}}, \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/beta/src/lib.rs\"}]},\
     {\"id\": \"gamma\", \"name\": \"gamma\", \"manifest_path\": \"/w/crates/gamma/Cargo.toml\", \"dependencies\": [{\"name\": \"y\"}], \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/gamma/src/lib.rs\"}]},\
     {\"id\": \"x\", \"name\": \"x\", \"manifest_path\": \"/r/x/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"y\", \"name\": \"y\", \"manifest_path\": \"/r/y/Cargo.toml\", \"dependencies\": []}\
     ], \"resolve\": {\"nodes\": [\
     {\"id\": \"alpha\", \"dependencies\": [\"x\"]},\
     {\"id\": \"beta\", \"dependencies\": [\"y\"]},\
     {\"id\": \"gamma\", \"dependencies\": [\"y\"]},\
     {\"id\": \"x\", \"dependencies\": [\"y\"]}\
     ]}}",
);
const UNREACHED: Text = Text::new(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"alpha\", \"gamma\"], \"packages\": [\
     {\"id\": \"alpha\", \"name\": \"alpha\", \"manifest_path\": \"/w/crates/alpha/Cargo.toml\", \"dependencies\": [], \"metadata\": {\"port\": {\"owns\": [\"y\"]}}, \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/alpha/src/lib.rs\"}]},\
     {\"id\": \"gamma\", \"name\": \"gamma\", \"manifest_path\": \"/w/crates/gamma/Cargo.toml\", \"dependencies\": [{\"name\": \"y\"}], \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/gamma/src/lib.rs\"}]},\
     {\"id\": \"y\", \"name\": \"y\", \"manifest_path\": \"/r/y/Cargo.toml\", \"dependencies\": []}\
     ], \"resolve\": {\"nodes\": [\
     {\"id\": \"gamma\", \"dependencies\": [\"y\"]}\
     ]}}",
);
const STALE: Text = Text::new(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"alpha\"], \"packages\": [\
     {\"id\": \"alpha\", \"name\": \"alpha\", \"manifest_path\": \"/w/crates/alpha/Cargo.toml\", \"dependencies\": [], \"metadata\": {\"port\": {\"owns\": [\"y\"]}}, \"targets\": [{\"kind\": [\"lib\"], \"src_path\": \"/w/crates/alpha/src/lib.rs\"}]},\
     {\"id\": \"y\", \"name\": \"y\", \"manifest_path\": \"/r/y/Cargo.toml\", \"dependencies\": []}\
     ], \"resolve\": {\"nodes\": []}}",
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

const IN_STORE_PORTS: &[Source] =
    &[Source::new(Located::new("crates/store-ports/src/lib.rs"), Text::new("pub use sqlx as __sqlx;\n"))];
const IN_STORE_PORTS_MACROS: &[Source] = &[Source::new(
    Located::new("crates/store-ports-macros/src/lib.rs"),
    Text::new("fn f() {\n    quote!(store_ports::__sqlx);\n}\n"),
)];
const SQLX_ELSEWHERE: &[Source] = &[Source::new(
    Located::new("crates/design-store/src/lib.rs"),
    Text::new("use store_ports::__sqlx::SqliteConnection;\n"),
)];
const SQLX_IN_MACRO: &[Source] = &[Source::new(
    Located::new("crates/design/tests/run.rs"),
    Text::new("fn f() {\n    vec![store_ports::__sqlx::query(a)];\n}\n"),
)];
const SQLX_ITSELF: &[Source] =
    &[Source::new(Located::new("crates/design/src/lib.rs"), Text::new("use store_ports::Store;\nfn sqlx() {}\n"))];

const ROLE_CRATES: &[Source] = &[
    Source::new(
        Located::new("crates/grammar/src/lib.rs"),
        Text::new("#![deny(clippy::wildcard_enum_match_arm)]\n\npub use logos;\n"),
    ),
    Source::new(
        Located::new("crates/grammar-notes/src/lib.rs"),
        Text::new("#![deny(clippy::wildcard_enum_match_arm)]\n\npub struct Notes;\n"),
    ),
    Source::new(
        Located::new("crates/grammar-ident/src/lib.rs"),
        Text::new("#![deny(clippy::wildcard_enum_match_arm)]\n\npub struct Words;\n"),
    ),
    Source::new(Located::new("crates/design-model/src/lib.rs"), Text::new("pub struct Title;\n")),
    Source::new(Located::new("crates/port-derive/src/lib.rs"), Text::new("pub struct Declared;\n")),
    Source::new(Located::new("crates/store-ports/src/lib.rs"), Text::new("pub struct Store;\n")),
    Source::new(Located::new("crates/store-ports-macros/src/lib.rs"), Text::new("pub struct Query;\n")),
    Source::new(Located::new("crates/design-store/src/lib.rs"), Text::new("pub struct Designs;\n")),
    Source::new(Located::new("crates/design/src/lib.rs"), Text::new("pub struct Design;\n")),
];
const LOGOS_PORT: Source = Source::new(
    Located::new("crates/grammar/Cargo.toml"),
    Text::new(
        "[package]\nname = \"grammar\"\n\n[package.metadata.port]\nowns = [\"logos\"]\ndependents = \"lexer\"\n\n[dependencies]\nlogos = \"0.16\"\n",
    ),
);
const NOTES_LEXER: Source = Source::new(
    Located::new("crates/grammar-notes/Cargo.toml"),
    Text::new(
        "[package]\nname = \"grammar-notes\"\n\n[package.metadata.lexer]\nformat = \"notes\"\n\n[dependencies]\ngrammar = { path = \"../grammar\" }\nstore-ports = { path = \"../store-ports\" }\n",
    ),
);
const IDENT_LEXER: Source = Source::new(
    Located::new("crates/grammar-ident/Cargo.toml"),
    Text::new(
        "[package]\nname = \"grammar-ident\"\n\n[package.metadata.lexer]\nformat = \"ident\"\n\n[dependencies]\ngrammar = { path = \"../grammar\" }\n",
    ),
);
const MODEL: Source = Source::new(
    Located::new("crates/design-model/Cargo.toml"),
    Text::new(
        "[package]\nname = \"design-model\"\n\n[dependencies]\ngrammar = { path = \"../grammar\" }\ngrammar-notes = { path = \"../grammar-notes\" }\n",
    ),
);
const DERIVE: Source = Source::new(
    Located::new("crates/port-derive/Cargo.toml"),
    Text::new(
        "[package]\nname = \"port-derive\"\n\n[dependencies]\ndesign-model = { path = \"../design-model\" }\ngrammar-ident = { path = \"../grammar-ident\" }\n",
    ),
);

macro_rules! edge {
    ($dependency:literal) => {
        concat!("{\"name\": \"", $dependency, "\", \"pkg\": \"", $dependency, "\", \"dep_kinds\": [{\"kind\": null}]}")
    };
}

macro_rules! member {
    ($name:literal, $kind:literal, $metadata:literal) => {
        concat!(
            "{\"id\": \"", $name, "\", \"name\": \"", $name, "\", \"manifest_path\": \"/w/crates/", $name,
            "/Cargo.toml\", \"dependencies\": [], \"metadata\": ", $metadata, ", \"targets\": [{\"kind\": [\"", $kind, "\"], \"src_path\": \"/w/crates/", $name, "/src/lib.rs\"}]}"
        )
    };
    ($name:literal, $kind:literal, $metadata:literal, $first:literal $(, $rest:literal)*) => {
        concat!(
            "{\"id\": \"", $name, "\", \"name\": \"", $name, "\", \"manifest_path\": \"/w/crates/", $name,
            "/Cargo.toml\", \"dependencies\": [{\"name\": \"", $first, "\"}", $(", {\"name\": \"", $rest, "\"}",)*
            "], \"metadata\": ", $metadata, ", \"targets\": [{\"kind\": [\"", $kind, "\"], \"src_path\": \"/w/crates/", $name, "/src/lib.rs\"}]}"
        )
    };
}

macro_rules! node {
    ($name:literal) => {
        concat!("{\"id\": \"", $name, "\", \"dependencies\": [], \"deps\": []}")
    };
    ($name:literal, $first:literal $(, $rest:literal)*) => {
        concat!(
            "{\"id\": \"", $name, "\", \"dependencies\": [\"", $first, "\"", $(", \"", $rest, "\"",)* "], \"deps\": [",
            edge!($first), $(", ", edge!($rest),)* "]}"
        )
    };
}

macro_rules! roles {
    ($notes:expr, $ident:literal, $model:expr, $derive:expr) => {
        Text::new(concat!(
            "{\"workspace_root\": \"/w\", \"workspace_members\": [\"grammar\", \"grammar-notes\", \"grammar-ident\", \"design-model\", \"port-derive\", \"store-ports\", \"store-ports-macros\", \"design-store\", \"design\"], \"packages\": [",
            member!("grammar", "lib", "{\"port\": {\"owns\": [\"logos\"], \"dependents\": \"lexer\"}}", "logos"), ", ",
            member!("grammar-notes", "lib", "{\"lexer\": {\"format\": \"notes\"}}", "grammar", "store-ports"), ", ",
            member!("grammar-ident", "lib", $ident, "grammar"), ", ",
            member!("design-model", "lib", "null", "grammar", "grammar-notes"), ", ",
            member!("port-derive", "proc-macro", "null", "design-model", "grammar-ident"), ", ",
            member!("store-ports", "lib", "{\"port\": {\"owns\": [\"sqlx\"]}}", "sqlx", "store-ports-macros"), ", ",
            member!("store-ports-macros", "proc-macro", "{\"port\": {\"owns\": [\"sqlx-macros-core\"], \"expanders\": {\"role\": \"store\", \"macros\": [\"query\", \"migrate\", \"Columns\"]}}}", "sqlx-macros-core"), ", ",
            member!("design-store", "lib", "{\"store\": {}}", "store-ports"), ", ",
            member!("design", "lib", "null", "store-ports", "design-store"), ", ",
            "{\"id\": \"logos\", \"name\": \"logos\", \"manifest_path\": \"/r/logos/Cargo.toml\", \"dependencies\": []}, ",
            "{\"id\": \"sqlx\", \"name\": \"sqlx\", \"manifest_path\": \"/r/sqlx/Cargo.toml\", \"dependencies\": []}, ",
            "{\"id\": \"sqlx-macros-core\", \"name\": \"sqlx-macros-core\", \"manifest_path\": \"/r/sqlx-macros-core/Cargo.toml\", \"dependencies\": []}",
            "], \"resolve\": {\"nodes\": [",
            node!("grammar", "logos"), ", ",
            $notes, ", ",
            node!("grammar-ident", "grammar"), ", ",
            $model, ", ",
            $derive, ", ",
            node!("store-ports", "sqlx", "store-ports-macros"), ", ",
            node!("store-ports-macros", "sqlx-macros-core"), ", ",
            node!("design-store", "store-ports"), ", ",
            node!("design", "store-ports", "design-store"), ", ",
            node!("logos"), ", ",
            node!("sqlx"), ", ",
            node!("sqlx-macros-core"),
            "]}}"
        ))
    };
}

const LEXERS_IN_ROLE: Text = roles!(
    node!("grammar-notes", "grammar"),
    "{\"lexer\": {\"format\": \"ident\"}}",
    node!("design-model", "grammar-notes"),
    node!("port-derive", "grammar-ident")
);
const MODEL_ON_PORT: Text = roles!(
    node!("grammar-notes", "grammar"),
    "{\"lexer\": {\"format\": \"ident\"}}",
    node!("design-model", "grammar", "grammar-notes"),
    node!("port-derive", "grammar-ident")
);
const LEXER_ON_STORE: Text = roles!(
    node!("grammar-notes", "grammar", "store-ports"),
    "{\"lexer\": {\"format\": \"ident\"}}",
    node!("design-model", "grammar-notes"),
    node!("port-derive", "grammar-ident")
);
const SHARED_FORMAT: Text = roles!(
    node!("grammar-notes", "grammar"),
    "{\"lexer\": {\"format\": \"notes\"}}",
    node!("design-model", "grammar-notes"),
    node!("port-derive", "grammar-ident")
);
const MACRO_THROUGH_MODEL: Text = roles!(
    node!("grammar-notes", "grammar"),
    "{\"lexer\": {\"format\": \"ident\"}}",
    node!("design-model", "grammar", "grammar-notes"),
    node!("port-derive", "design-model", "grammar-ident")
);
const MACRO_THROUGH_PORT: Text = roles!(
    node!("grammar-notes", "grammar"),
    "null",
    node!("design-model", "grammar-notes"),
    node!("port-derive", "grammar-ident")
);

const DENIED: &str = "#![deny(clippy::wildcard_enum_match_arm)]\n\npub struct Lexed;\n";
const UNDENIED: &str = "pub struct Lexed;\n";

macro_rules! serving {
    ($grammar:expr, $ident:expr, $unspelled:expr) => {
        &[
            Source::new(Located::new("crates/grammar/src/lib.rs"), Text::new($grammar)),
            Source::new(Located::new("crates/grammar-ident/src/lib.rs"), Text::new($ident)),
            Source::new(Located::new("crates/grammar-unspelled/src/lib.rs"), Text::new($unspelled)),
            Source::new(Located::new("crates/design-model/src/lib.rs"), Text::new(UNDENIED)),
        ]
    };
}

const ALL_DENIED: &[Source] = serving!(DENIED, DENIED, DENIED);
const LEXER_UNDENIED: &[Source] = serving!(DENIED, UNDENIED, DENIED);
const PORT_UNDENIED: &[Source] = serving!(UNDENIED, DENIED, DENIED);
const HELPER_UNDENIED: &[Source] = serving!(DENIED, DENIED, UNDENIED);
const LEXER_FORBIDDEN: &[Source] =
    serving!(DENIED, "#![forbid(clippy::wildcard_enum_match_arm)]\n\npub struct Lexed;\n", DENIED);
const LEXER_DENIED_IN_TESTS: &[Source] =
    serving!(DENIED, "#![cfg_attr(test, deny(clippy::wildcard_enum_match_arm))]\n\npub struct Lexed;\n", DENIED);
const LEXER_DENIED_OTHER: &[Source] =
    serving!(DENIED, "#![deny(clippy::wildcard_imports)]\n\npub struct Lexed;\n", DENIED);
const LEXER_WITHOUT_ROOT: &[Source] = &[
    Source::new(Located::new("crates/grammar/src/lib.rs"), Text::new(DENIED)),
    Source::new(Located::new("crates/grammar-ident/src/words.rs"), Text::new(UNDENIED)),
    Source::new(Located::new("crates/grammar-unspelled/src/lib.rs"), Text::new(DENIED)),
];
const UNSPELLED: Source = Source::new(
    Located::new("crates/grammar-unspelled/Cargo.toml"),
    Text::new("[package]\nname = \"grammar-unspelled\"\n"),
);
const MODEL_ON_LEXER: Source = Source::new(
    Located::new("crates/design-model/Cargo.toml"),
    Text::new(
        "[package]\nname = \"design-model\"\n\n[dependencies]\ngrammar-ident = { path = \"../grammar-ident\" }\n",
    ),
);
const SERVING_MANIFESTS: [Source; 4] = [LOGOS_PORT, IDENT_LEXER, UNSPELLED, MODEL_ON_LEXER];

macro_rules! served {
    ($model:expr) => {
        Text::new(concat!(
            "{\"workspace_root\": \"/w\", \"workspace_members\": [\"grammar\", \"grammar-ident\", \"grammar-unspelled\", \"design-model\"], \"packages\": [",
            member!("grammar", "lib", "{\"port\": {\"owns\": [\"logos\"], \"dependents\": \"lexer\"}}", "logos"), ", ",
            member!("grammar-ident", "lib", "{\"lexer\": {\"format\": \"ident\"}}", "grammar", "grammar-unspelled"), ", ",
            member!("grammar-unspelled", "lib", "null"), ", ",
            member!("design-model", "lib", "null", "grammar-ident", "grammar-unspelled"), ", ",
            "{\"id\": \"logos\", \"name\": \"logos\", \"manifest_path\": \"/r/logos/Cargo.toml\", \"dependencies\": []}",
            "], \"resolve\": {\"nodes\": [",
            node!("grammar", "logos"), ", ",
            node!("grammar-ident", "grammar", "grammar-unspelled"), ", ",
            node!("grammar-unspelled"), ", ",
            $model, ", ",
            node!("logos"),
            "]}}"
        ))
    };
}

const ONLY_LEXERS_ON_HELPER: Text = served!(node!("design-model", "grammar-ident"));
const MODEL_ON_HELPER: Text = served!(node!("design-model", "grammar-ident", "grammar-unspelled"));

macro_rules! expanding {
    ($path:literal, $text:literal) => {
        &[
            Source::new(Located::new("crates/store-ports/src/lib.rs"), Text::new("pub struct Store;\n")),
            Source::new(Located::new("crates/store-ports-macros/src/lib.rs"), Text::new("pub struct Query;\n")),
            Source::new(Located::new($path), Text::new($text)),
        ]
    };
}

const QUERIED: &[Source] =
    expanding!("crates/design/src/lib.rs", "fn f() {\n    store_ports::query!(r#\"\nSELECT\n  1\n\"#);\n}\n");
const IMPORTED_QUERY: &[Source] = expanding!(
    "crates/design/src/lib.rs",
    "use store_ports::query;\nfn f() {\n    vec![query!(r#\"\nSELECT\n  1\n\"#)];\n}\n"
);
const DERIVED_COLUMNS: &[Source] =
    expanding!("crates/design/src/lib.rs", "use store_ports::Columns;\n#[derive(Debug, Columns)]\nstruct Row;\n");
const MIGRATED: &[Source] =
    expanding!("crates/design/src/lib.rs", "static MIGRATOR: Migrator = store_ports::migrate!();\n");
const QUERIED_THROUGH_MACRO_CRATE: &[Source] =
    expanding!("crates/design/src/lib.rs", "fn f() {\n    store_ports_macros::query!(r#\"\nSELECT\n  1\n\"#);\n}\n");
const CONDITIONALLY_DERIVED: &[Source] = expanding!(
    "crates/design/src/lib.rs",
    "use store_ports::Columns;\n#[cfg_attr(all(), derive(Columns))]\nstruct Row;\n"
);
const NESTED_CFG_DERIVED: &[Source] = expanding!(
    "crates/design/src/lib.rs",
    "use store_ports::Columns;\n#[cfg_attr(all(), cfg_attr(any(), derive(Debug, Columns)))]\nstruct Row;\n"
);
const CFG_DERIVED_OTHER: &[Source] = expanding!(
    "crates/design/src/lib.rs",
    "use store_ports::Columns;\n#[cfg_attr(all(), derive(Debug))]\nstruct Row;\n"
);
const QUERIED_IN_TESTS: &[Source] =
    expanding!("crates/design/tests/run.rs", "fn f() {\n    store_ports::query!(r#\"\nSELECT\n  1\n\"#);\n}\n");
const QUERIED_IN_TEST_MODULE: &[Source] = expanding!(
    "crates/design/src/lib.rs",
    "#[cfg(test)]\nmod tests {\n    fn f() {\n        store_ports::query!(r#\"\nSELECT\n  1\n\"#);\n    }\n}\n"
);
const QUERIED_IN_STORE: &[Source] =
    expanding!("crates/design-store/src/lib.rs", "fn f() {\n    store_ports::query!(r#\"\nSELECT\n  1\n\"#);\n}\n");
const QUERIED_IN_PORT: &[Source] = &[Source::new(
    Located::new("crates/store-ports/src/lib.rs"),
    Text::new(
        "fn f() {\n    crate::query!(r#\"\nSELECT\n  1\n\"#);\n    store_ports::query!(r#\"\nSELECT\n  1\n\"#);\n}\n",
    ),
)];
const OTHER_QUERY: &[Source] = expanding!("crates/design/src/lib.rs", "fn f() {\n    sqlite::query!(a);\n}\n");

struct Placed {
    manifests: &'static [Source],
    crates: &'static [Source],
    workspace: Text<'static>,
}

impl Placed {
    const fn new(manifests: &'static [Source], crates: &'static [Source], workspace: Text<'static>) -> Placed {
        Placed { manifests, crates, workspace }
    }

    fn flagged(&self) -> Lines {
        Sources::placed_in::<OwnedDependencies>(self.manifests, self.crates, self.workspace)
    }

    fn roles(&self) -> Lines {
        Sources::placed_in::<KeyDependencyRoles>(self.manifests, self.crates, self.workspace)
    }
}

struct Expanding(&'static [Source]);

impl Expanding {
    fn flagged(&self) -> Lines {
        Sources::placed_in::<KeyDependencyRoles>(&EXPANDING_MANIFESTS, self.0, LEXERS_IN_ROLE)
    }
}

const EXPANDING_MANIFESTS: [Source; 4] = [
    Source::new(
        Located::new("crates/store-ports/Cargo.toml"),
        Text::new("[package]\nname = \"store-ports\"\n\n[package.metadata.port]\nowns = [\"sqlx\"]\n"),
    ),
    Source::new(
        Located::new("crates/store-ports-macros/Cargo.toml"),
        Text::new(
            "[package]\nname = \"store-ports-macros\"\n\n[package.metadata.port]\nowns = [\"sqlx-macros-core\"]\nexpanders = { role = \"store\", macros = [\"query\", \"migrate\", \"Columns\"] }\n",
        ),
    ),
    Source::new(
        Located::new("crates/design-store/Cargo.toml"),
        Text::new("[package]\nname = \"design-store\"\n\n[package.metadata.store]\n"),
    ),
    Source::new(Located::new("crates/design/Cargo.toml"), Text::new("[package]\nname = \"design\"\n")),
];

const ROLE_MANIFESTS: [Source; 5] = [LOGOS_PORT, NOTES_LEXER, IDENT_LEXER, MODEL, DERIVE];

laws::table!(
    a_crate_outside_every_owning_unit_is_flagged_where_it_depends_on_a_package_an_owned_package_reaches,
    Placed::flagged,
    [
        Placed::new(&[PORT, MACROS, ON_CLOSURE], UNIT_CRATES, STORE_ON_LIBSQLITE) => Lines::from(["crates/design-store/Cargo.toml:5"]),
        Placed::new(&[PORT, MACROS, ON_OWNED], UNIT_CRATES, STORE_ON_SQLX) => Lines::from(["crates/design-store/Cargo.toml:5"]),
        Placed::new(&[PORT, MACROS, DEV_ON_OWNED], UNIT_CRATES, STORE_ON_SQLX) => Lines::from(["crates/design-store/Cargo.toml:5"]),
        Placed::new(&[PORT, MACROS, RENAMED], UNIT_CRATES, STORE_ON_DATABASE) => Lines::from(["crates/design-store/Cargo.toml:5"]),
        Placed::new(&[PORT, MACROS, WORKSPACE_RENAMED], UNIT_CRATES, STORE_ON_DATABASE) => Lines::from(["crates/design-store/Cargo.toml:5"]),
        Placed::new(&[PORT, MACROS, ON_MACRO_CLOSURE], UNIT_CRATES, STORE_ON_SQLX_CORE) => Lines::from(["crates/design-store/Cargo.toml:5"]),
        Placed::new(&[PORT, MACROS, ON_OWNED], UNIT_CRATES, STORE_UNDECLARED) => Lines::from(["crates/design-store/Cargo.toml:4"]),
    ]
);

laws::table!(
    a_shared_package_and_a_package_outside_every_owned_closure_pass,
    Placed::flagged,
    [
        Placed::new(&[PORT, MACROS, ON_SHARED], UNIT_CRATES, STORE_ON_SERDE_JSON) => Lines::default(),
        Placed::new(&[PORT, MACROS, ON_UNOWNED], UNIT_CRATES, STORE_ON_LOGOS) => Lines::default(),
    ]
);

laws::table!(
    a_port_and_the_proc_macro_crates_it_depends_on_own_jointly,
    Placed::flagged,
    [
        Placed::new(&[PORT_ON_CLOSURE, MACROS], UNIT_CRATES, STORE) => Lines::default(),
        Placed::new(&[PORT, MACROS], UNIT_CRATES, STORE_WITHOUT_MACROS) => Lines::from(["crates/store-ports-macros/Cargo.toml:9"]),
    ]
);

laws::table!(
    a_proc_macro_crate_two_ports_share_owns_for_both_and_flags_only_crates_outside_them,
    Placed::flagged,
    [
        Placed::new(&[ALPHA, BETA, SHARED_MACROS], SHARED_MACRO_CRATES, SHARED_MACRO) => Lines::default(),
        Placed::new(&[ALPHA, BETA, SHARED_MACROS, GAMMA], SHARED_MACRO_CRATES, SHARED_MACRO) => Lines::from(["crates/gamma/Cargo.toml:5"]),
    ]
);

laws::table!(
    a_package_a_unit_names_in_owns_is_depended_on_directly_only_by_that_unit_even_inside_another_units_closure,
    Placed::flagged,
    [
        Placed::new(&[ALPHA_ON_X_AND_Y, BETA_ON_Y], CROSS_UNIT_CRATES, CROSS_UNIT) => Lines::from(["crates/alpha/Cargo.toml:9"]),
        Placed::new(&[ALPHA_ON_X, BETA_ON_Y, GAMMA_ON_Y], CROSS_UNIT_CRATES, OWNER_NAMED) => Lines::from(["crates/gamma/Cargo.toml:5"]),
    ]
);

laws::table!(
    an_owns_entry_its_unit_does_not_reach_is_flagged_and_still_owns_the_package_by_name,
    Placed::flagged,
    [
        Placed::new(&[ALPHA_OWNS_UNREACHED, GAMMA_ON_Y], CROSS_UNIT_CRATES, UNREACHED) => Lines::from(["crates/gamma/Cargo.toml:5", "crates/alpha/Cargo.toml:5"]),
        Placed::new(&[ALPHA_OWNS_UNREACHED], CROSS_UNIT_CRATES, STALE) => Lines::from(["crates/alpha/Cargo.toml:5"]),
    ]
);

laws::table!(
    a_crate_that_depends_directly_on_a_port_naming_dependents_declares_that_role,
    Placed::roles,
    [
        Placed::new(&ROLE_MANIFESTS, ROLE_CRATES, LEXERS_IN_ROLE) => Lines::default(),
        Placed::new(&ROLE_MANIFESTS, ROLE_CRATES, MODEL_ON_PORT) => Lines::from(["crates/design-model/Cargo.toml:5"]),
    ]
);

laws::table!(
    a_lexer_is_a_leaf_of_its_own_format,
    Placed::roles,
    [
        Placed::new(&ROLE_MANIFESTS, ROLE_CRATES, LEXER_ON_STORE) => Lines::from(["crates/grammar-notes/Cargo.toml:9"]),
        Placed::new(&ROLE_MANIFESTS, ROLE_CRATES, SHARED_FORMAT) => Lines::from(["crates/grammar-ident/Cargo.toml:5", "crates/grammar-notes/Cargo.toml:5"]),
    ]
);

laws::table!(
    a_workspace_proc_macro_reaches_an_owned_package_only_through_a_crate_of_the_dependents_role,
    Placed::roles,
    [
        Placed::new(&ROLE_MANIFESTS, ROLE_CRATES, MACRO_THROUGH_MODEL) => Lines::from(["crates/design-model/Cargo.toml:5", "crates/port-derive/Cargo.toml:5"]),
        Placed::new(&ROLE_MANIFESTS, ROLE_CRATES, MACRO_THROUGH_PORT) => Lines::from(["crates/grammar-ident/Cargo.toml:8", "crates/port-derive/Cargo.toml:6"]),
    ]
);

laws::table!(
    a_lexer_its_port_and_each_crate_only_they_depend_on_deny_the_lints_grammar_denied,
    Placed::roles,
    [
        Placed::new(&SERVING_MANIFESTS, ALL_DENIED, ONLY_LEXERS_ON_HELPER) => Lines::default(),
        Placed::new(&SERVING_MANIFESTS, LEXER_UNDENIED, ONLY_LEXERS_ON_HELPER) => Lines::from(["crates/grammar-ident/src/lib.rs:1"]),
        Placed::new(&SERVING_MANIFESTS, PORT_UNDENIED, ONLY_LEXERS_ON_HELPER) => Lines::from(["crates/grammar/src/lib.rs:1"]),
        Placed::new(&SERVING_MANIFESTS, HELPER_UNDENIED, ONLY_LEXERS_ON_HELPER) => Lines::from(["crates/grammar-unspelled/src/lib.rs:1"]),
        Placed::new(&SERVING_MANIFESTS, HELPER_UNDENIED, MODEL_ON_HELPER) => Lines::default(),
        Placed::new(&SERVING_MANIFESTS, LEXER_FORBIDDEN, ONLY_LEXERS_ON_HELPER) => Lines::default(),
        Placed::new(&SERVING_MANIFESTS, LEXER_DENIED_IN_TESTS, ONLY_LEXERS_ON_HELPER) => Lines::from(["crates/grammar-ident/src/lib.rs:1"]),
        Placed::new(&SERVING_MANIFESTS, LEXER_DENIED_OTHER, ONLY_LEXERS_ON_HELPER) => Lines::from(["crates/grammar-ident/src/lib.rs:1"]),
        Placed::new(&SERVING_MANIFESTS, LEXER_WITHOUT_ROOT, ONLY_LEXERS_ON_HELPER) => Lines::from(["crates/grammar-ident/Cargo.toml:1"]),
    ]
);

laws::table!(
    shipped_code_outside_a_store_expands_no_store_ports_macro,
    Expanding::flagged,
    [
        Expanding(QUERIED) => Lines::from(["crates/design/src/lib.rs:2"]),
        Expanding(IMPORTED_QUERY) => Lines::from(["crates/design/src/lib.rs:3"]),
        Expanding(DERIVED_COLUMNS) => Lines::from(["crates/design/src/lib.rs:2"]),
        Expanding(MIGRATED) => Lines::from(["crates/design/src/lib.rs:1"]),
        Expanding(QUERIED_THROUGH_MACRO_CRATE) => Lines::from(["crates/design/src/lib.rs:2"]),
        Expanding(CONDITIONALLY_DERIVED) => Lines::from(["crates/design/src/lib.rs:2"]),
        Expanding(NESTED_CFG_DERIVED) => Lines::from(["crates/design/src/lib.rs:2"]),
        Expanding(CFG_DERIVED_OTHER) => Lines::default(),
        Expanding(QUERIED_IN_TESTS) => Lines::default(),
        Expanding(QUERIED_IN_TEST_MODULE) => Lines::default(),
        Expanding(QUERIED_IN_STORE) => Lines::default(),
        Expanding(QUERIED_IN_PORT) => Lines::default(),
        Expanding(OTHER_QUERY) => Lines::default(),
    ]
);

laws::table!(
    in_a_port_crate_effects_sit_only_in_fillers,
    Sources::flagged::<EffectsInFillers>,
    [
        FILLED => Lines::default(),
        INHERENT => Lines::from(["crates/store-ports/src/lib.rs:8"]),
        FREE => Lines::from(["crates/store-ports/src/load.rs:2"]),
        OTHER_TRAIT => Lines::from(["crates/store-ports/src/lib.rs:8"]),
        UNPORTED_FILLER => Lines::from(["crates/store-ports/src/lib.rs:12"]),
        RELEASED => Lines::default(),
        QUALIFIED_RELEASE => Lines::default(),
        RELEASED_ELSEWHERE => Lines::from(["crates/store-ports/src/lib.rs:8"]),
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
        ACQUIRED_ELSEWHERE => Lines::from(["crates/store-ports/src/lib.rs:2"]),
        IMPORTED => Lines::from(["crates/design/src/lib.rs:1"]),
        IN_MACRO => Lines::from(["crates/design/tests/run.rs:2"]),
    ]
);

laws::table!(
    the_sqlx_store_ports_re_exports_is_named_only_in_store_ports_and_its_macros,
    Sources::flagged::<HiddenSqlx>,
    [
        IN_STORE_PORTS => Lines::default(),
        IN_STORE_PORTS_MACROS => Lines::default(),
        SQLX_ELSEWHERE => Lines::from(["crates/design-store/src/lib.rs:1"]),
        SQLX_IN_MACRO => Lines::from(["crates/design/tests/run.rs:2"]),
        SQLX_ITSELF => Lines::default(),
    ]
);

#[test]
fn a_lexer_sharing_its_format_is_told_which_other_lexer_shares_it() {
    let told = Shown::placed_in::<KeyDependencyRoles>(&ROLE_MANIFESTS, ROLE_CRATES, SHARED_FORMAT);
    let ident = told.iter().filter(|line| line.contains("grammar-ident/Cargo.toml"));
    assert_eq!(ident.filter(|line| line.contains("which grammar-notes also lexes")).count(), 1, "{told:?}");
}

#[test]
fn a_stale_owns_entry_and_a_lexer_off_its_leaf_say_what_they_found() {
    let stale = Shown::placed_in::<OwnedDependencies>(&[ALPHA_OWNS_UNREACHED], CROSS_UNIT_CRATES, STALE);
    let unleafed = Shown::placed_in::<KeyDependencyRoles>(&ROLE_MANIFESTS, ROLE_CRATES, LEXER_ON_STORE);
    assert_eq!(
        stale.iter().filter(|line| line.contains("which nothing in its unit depends on")).count(),
        1,
        "{stale:?}"
    );
    assert_eq!(
        unleafed.iter().filter(|line| line.contains("is a lexer and depends on store-ports")).count(),
        1,
        "{unleafed:?}"
    );
}

#[test]
fn an_expansion_outside_its_role_names_the_role_to_declare() {
    let told = Shown::placed_in::<KeyDependencyRoles>(&EXPANDING_MANIFESTS, QUERIED, LEXERS_IN_ROLE);
    let named = told.iter().filter(|line| line.contains("in a crate that does not declare [package.metadata.store]"));
    assert_eq!(named.count(), 1, "{told:?}");
}
