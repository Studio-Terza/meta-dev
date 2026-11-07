use lint_fixtures::{Lines, Located, Source, Sources, Text};
use lint_port_dependencies::{CountedCalls, EffectsInFillers, HiddenSqlx, OwnedDependencies};

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
     {\"id\": \"store-ports\", \"name\": \"store-ports\", \"manifest_path\": \"/w/crates/store-ports/Cargo.toml\", \"dependencies\": [], \"metadata\": {\"port\": {\"owns\": [\"sqlx\"], \"shared\": [\"serde_json\"]}}, \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"store-ports-macros\", \"name\": \"store-ports-macros\", \"manifest_path\": \"/w/crates/store-ports-macros/Cargo.toml\", \"dependencies\": [], \"metadata\": {\"port\": {\"owns\": [\"sqlx-macros-core\"]}}, \"targets\": [{\"kind\": [\"proc-macro\"]}]},\
     {\"id\": \"design-store\", \"name\": \"design-store\", \"manifest_path\": \"/w/crates/design-store/Cargo.toml\", \"dependencies\": [", $declared, "], \"metadata\": null, \"targets\": [{\"kind\": [\"lib\"]}]},\
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
     {\"id\": \"store-ports\", \"name\": \"store-ports\", \"manifest_path\": \"/w/crates/store-ports/Cargo.toml\", \"dependencies\": [], \"metadata\": {\"port\": {\"owns\": [\"sqlx\"]}}, \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"store-ports-macros\", \"name\": \"store-ports-macros\", \"manifest_path\": \"/w/crates/store-ports-macros/Cargo.toml\", \"dependencies\": [{\"name\": \"sqlx-core\"}], \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"design-store\", \"name\": \"design-store\", \"manifest_path\": \"/w/crates/design-store/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]},\
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
     {\"id\": \"alpha\", \"name\": \"alpha\", \"manifest_path\": \"/w/crates/alpha/Cargo.toml\", \"dependencies\": [{\"name\": \"shared-macros\"}, {\"name\": \"ext\"}], \"metadata\": {\"port\": {\"owns\": []}}, \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"beta\", \"name\": \"beta\", \"manifest_path\": \"/w/crates/beta/Cargo.toml\", \"dependencies\": [{\"name\": \"shared-macros\"}], \"metadata\": {\"port\": {\"owns\": []}}, \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"shared-macros\", \"name\": \"shared-macros\", \"manifest_path\": \"/w/crates/shared-macros/Cargo.toml\", \"dependencies\": [{\"name\": \"ext\"}], \"metadata\": {\"port\": {\"owns\": [\"ext\"]}}, \"targets\": [{\"kind\": [\"proc-macro\"]}]},\
     {\"id\": \"ext\", \"name\": \"ext\", \"manifest_path\": \"/w/crates/ext/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"gamma\", \"name\": \"gamma\", \"manifest_path\": \"/w/crates/gamma/Cargo.toml\", \"dependencies\": [{\"name\": \"ext\"}], \"targets\": [{\"kind\": [\"lib\"]}]}\
     ], \"resolve\": {\"nodes\": [\
     {\"id\": \"alpha\", \"dependencies\": [\"shared-macros\", \"ext\"]},\
     {\"id\": \"beta\", \"dependencies\": [\"shared-macros\"]},\
     {\"id\": \"shared-macros\", \"dependencies\": [\"ext\"]},\
     {\"id\": \"gamma\", \"dependencies\": [\"ext\"]}\
     ]}}",
);

const CROSS_UNIT: Text = Text::new(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"alpha\", \"beta\"], \"packages\": [\
     {\"id\": \"alpha\", \"name\": \"alpha\", \"manifest_path\": \"/w/crates/alpha/Cargo.toml\", \"dependencies\": [{\"name\": \"x\"}, {\"name\": \"y\"}], \"metadata\": {\"port\": {\"owns\": [\"x\"]}}, \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"beta\", \"name\": \"beta\", \"manifest_path\": \"/w/crates/beta/Cargo.toml\", \"dependencies\": [{\"name\": \"y\"}], \"metadata\": {\"port\": {\"owns\": [\"y\"]}}, \"targets\": [{\"kind\": [\"lib\"]}]},\
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
     {\"id\": \"alpha\", \"name\": \"alpha\", \"manifest_path\": \"/w/crates/alpha/Cargo.toml\", \"dependencies\": [{\"name\": \"x\"}], \"metadata\": {\"port\": {\"owns\": [\"x\"]}}, \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"beta\", \"name\": \"beta\", \"manifest_path\": \"/w/crates/beta/Cargo.toml\", \"dependencies\": [{\"name\": \"y\"}], \"metadata\": {\"port\": {\"owns\": [\"y\"]}}, \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"gamma\", \"name\": \"gamma\", \"manifest_path\": \"/w/crates/gamma/Cargo.toml\", \"dependencies\": [{\"name\": \"y\"}], \"targets\": [{\"kind\": [\"lib\"]}]},\
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
     {\"id\": \"alpha\", \"name\": \"alpha\", \"manifest_path\": \"/w/crates/alpha/Cargo.toml\", \"dependencies\": [], \"metadata\": {\"port\": {\"owns\": [\"y\"]}}, \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"gamma\", \"name\": \"gamma\", \"manifest_path\": \"/w/crates/gamma/Cargo.toml\", \"dependencies\": [{\"name\": \"y\"}], \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"y\", \"name\": \"y\", \"manifest_path\": \"/r/y/Cargo.toml\", \"dependencies\": []}\
     ], \"resolve\": {\"nodes\": [\
     {\"id\": \"gamma\", \"dependencies\": [\"y\"]}\
     ]}}",
);
const STALE: Text = Text::new(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"alpha\"], \"packages\": [\
     {\"id\": \"alpha\", \"name\": \"alpha\", \"manifest_path\": \"/w/crates/alpha/Cargo.toml\", \"dependencies\": [], \"metadata\": {\"port\": {\"owns\": [\"y\"]}}, \"targets\": [{\"kind\": [\"lib\"]}]},\
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
}

laws::table!(
    a_crate_outside_every_owning_unit_is_flagged_where_it_depends_on_a_package_an_owned_package_reaches,
    Placed::flagged,
    [
        Placed::new(&[PORT, MACROS, ON_CLOSURE], UNIT_CRATES, STORE_ON_LIBSQLITE) => ["crates/design-store/Cargo.toml:5"],
        Placed::new(&[PORT, MACROS, ON_OWNED], UNIT_CRATES, STORE_ON_SQLX) => ["crates/design-store/Cargo.toml:5"],
        Placed::new(&[PORT, MACROS, DEV_ON_OWNED], UNIT_CRATES, STORE_ON_SQLX) => ["crates/design-store/Cargo.toml:5"],
        Placed::new(&[PORT, MACROS, RENAMED], UNIT_CRATES, STORE_ON_DATABASE) => ["crates/design-store/Cargo.toml:5"],
        Placed::new(&[PORT, MACROS, WORKSPACE_RENAMED], UNIT_CRATES, STORE_ON_DATABASE) => ["crates/design-store/Cargo.toml:5"],
        Placed::new(&[PORT, MACROS, ON_MACRO_CLOSURE], UNIT_CRATES, STORE_ON_SQLX_CORE) => ["crates/design-store/Cargo.toml:5"],
        Placed::new(&[PORT, MACROS, ON_OWNED], UNIT_CRATES, STORE_UNDECLARED) => ["crates/design-store/Cargo.toml:4"],
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
        Placed::new(&[PORT, MACROS], UNIT_CRATES, STORE_WITHOUT_MACROS) => ["crates/store-ports-macros/Cargo.toml:9"],
    ]
);

laws::table!(
    a_proc_macro_crate_two_ports_share_owns_for_both_and_flags_only_crates_outside_them,
    Placed::flagged,
    [
        Placed::new(&[ALPHA, BETA, SHARED_MACROS], SHARED_MACRO_CRATES, SHARED_MACRO) => Lines::default(),
        Placed::new(&[ALPHA, BETA, SHARED_MACROS, GAMMA], SHARED_MACRO_CRATES, SHARED_MACRO) => ["crates/gamma/Cargo.toml:5"],
    ]
);

laws::table!(
    a_package_a_unit_names_in_owns_is_depended_on_directly_only_by_that_unit_even_inside_another_units_closure,
    Placed::flagged,
    [
        Placed::new(&[ALPHA_ON_X_AND_Y, BETA_ON_Y], CROSS_UNIT_CRATES, CROSS_UNIT) => ["crates/alpha/Cargo.toml:9"],
        Placed::new(&[ALPHA_ON_X, BETA_ON_Y, GAMMA_ON_Y], CROSS_UNIT_CRATES, OWNER_NAMED) => ["crates/gamma/Cargo.toml:5"],
    ]
);

laws::table!(
    an_owns_entry_its_unit_does_not_reach_is_flagged_and_still_owns_the_package_by_name,
    Placed::flagged,
    [
        Placed::new(&[ALPHA_OWNS_UNREACHED, GAMMA_ON_Y], CROSS_UNIT_CRATES, UNREACHED) => ["crates/gamma/Cargo.toml:5", "crates/alpha/Cargo.toml:5"],
        Placed::new(&[ALPHA_OWNS_UNREACHED], CROSS_UNIT_CRATES, STALE) => ["crates/alpha/Cargo.toml:5"],
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

laws::table!(
    the_sqlx_store_ports_re_exports_is_named_only_in_store_ports_and_its_macros,
    Sources::flagged::<HiddenSqlx>,
    [
        IN_STORE_PORTS => Lines::default(),
        IN_STORE_PORTS_MACROS => Lines::default(),
        SQLX_ELSEWHERE => ["crates/design-store/src/lib.rs:1"],
        SQLX_IN_MACRO => ["crates/design/tests/run.rs:2"],
        SQLX_ITSELF => Lines::default(),
    ]
);
