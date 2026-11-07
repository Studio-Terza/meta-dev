use lint_fixtures::{Lines, Located, Source, Sources, Text};
use lint_port_dependencies::{CountedCalls, DominatedPackages, EffectsInFillers, HiddenSqlx, OwnedDependencies};

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
const HASHBROWN_USER: Source = Source::new(
    Located::new("crates/design/Cargo.toml"),
    Text::new("[package]\nname = \"design\"\n\n[dependencies]\nhashbrown = \"0.16\"\n"),
);
const SIBLING: Source = Source::new(
    Located::new("crates/design/Cargo.toml"),
    Text::new("[package]\nname = \"design\"\n\n[dependencies]\nlogos-codegen = \"1\"\n"),
);
const FAMILY_OWNER: Source = Source::new(
    Located::new("crates/grammar/Cargo.toml"),
    Text::new(
        "[package]\nname = \"grammar\"\n\n[package.metadata.port]\nowns = [\"logos\", \"logos-codegen\"]\n\n[dependencies]\nlogos = \"1\"\nlogos-codegen = \"1\"\n",
    ),
);
const HASHBROWN_OWNER: Source = Source::new(
    Located::new("crates/grammar/Cargo.toml"),
    Text::new(
        "[package]\nname = \"grammar\"\n\n[package.metadata.port]\nowns = [\"logos\", \"hashbrown\"]\n\n[dependencies]\nlogos = \"1\"\n",
    ),
);

const UNIT_CRATES: &[Source] = &[
    Source::new(Located::new("crates/store-ports/src/lib.rs"), Text::new("pub struct Store;\n")),
    Source::new(Located::new("crates/store-ports-macros/src/lib.rs"), Text::new("pub struct Query;\n")),
    Source::new(Located::new("crates/design-store/src/lib.rs"), Text::new("pub struct Designs;\n")),
];
const PORT: Source = Source::new(
    Located::new("crates/store-ports/Cargo.toml"),
    Text::new(
        "[package]\nname = \"store-ports\"\n\n[package.metadata.port]\nowns = [\"sqlx\", \"sqlx-core\"]\n\n[dependencies]\nsqlx = \"0.9\"\nstore-ports-macros = { path = \"../store-ports-macros\" }\n",
    ),
);
const PORT_ON_MACRO_FAMILY: Source = Source::new(
    Located::new("crates/store-ports/Cargo.toml"),
    Text::new(
        "[package]\nname = \"store-ports\"\n\n[package.metadata.port]\nowns = [\"sqlx\", \"sqlx-core\"]\n\n[dependencies]\nsqlx-macros-core = \"0.9\"\nstore-ports-macros = { path = \"../store-ports-macros\" }\n",
    ),
);
const MACROS: Source = Source::new(
    Located::new("crates/store-ports-macros/Cargo.toml"),
    Text::new(
        "[package]\nname = \"store-ports-macros\"\n\n[package.metadata.port]\nowns = [\"sqlx-macros-core\"]\n\n[dependencies]\nsqlx-macros-core = \"0.9\"\nsqlx-core = \"0.9\"\n",
    ),
);
const STRANGER: Source = Source::new(
    Located::new("crates/design-store/Cargo.toml"),
    Text::new("[package]\nname = \"design-store\"\n\n[dependencies]\nsqlx-core = \"0.9\"\n"),
);

const EMPTY: Text = Text::new("{\"workspace_root\": \"/w\", \"packages\": [], \"resolve\": null}");
const LOGOS: Text = Text::new(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"grammar\", \"design\"], \"packages\": [\
     {\"id\": \"grammar\", \"name\": \"grammar\", \"manifest_path\": \"/w/crates/grammar/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"design\", \"name\": \"design\", \"manifest_path\": \"/w/crates/design/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"logos\", \"name\": \"logos\", \"manifest_path\": \"/r/logos/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"hashbrown\", \"name\": \"hashbrown\", \"manifest_path\": \"/r/hashbrown/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"serde\", \"name\": \"serde\", \"manifest_path\": \"/r/serde/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]}\
     ], \"resolve\": {\"nodes\": [\
     {\"id\": \"grammar\", \"dependencies\": [\"logos\"]},\
     {\"id\": \"design\", \"dependencies\": [\"serde\"]},\
     {\"id\": \"logos\", \"dependencies\": [\"hashbrown\"]},\
     {\"id\": \"hashbrown\", \"dependencies\": []},\
     {\"id\": \"serde\", \"dependencies\": []}\
     ]}}",
);
const LOGOS_AND_HASHBROWN: Text = Text::new(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"grammar\", \"design\"], \"packages\": [\
     {\"id\": \"grammar\", \"name\": \"grammar\", \"manifest_path\": \"/w/crates/grammar/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"design\", \"name\": \"design\", \"manifest_path\": \"/w/crates/design/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"logos\", \"name\": \"logos\", \"manifest_path\": \"/r/logos/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"hashbrown\", \"name\": \"hashbrown\", \"manifest_path\": \"/r/hashbrown/Cargo.toml\", \"dependencies\": []}\
     ], \"resolve\": {\"nodes\": [\
     {\"id\": \"grammar\", \"dependencies\": [\"logos\"]},\
     {\"id\": \"design\", \"dependencies\": [\"hashbrown\"]},\
     {\"id\": \"logos\", \"dependencies\": [\"hashbrown\"]}\
     ]}}",
);
const STORE: Text = Text::new(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"store-ports\", \"store-ports-macros\", \"design-store\"], \"packages\": [\
     {\"id\": \"store-ports\", \"name\": \"store-ports\", \"manifest_path\": \"/w/crates/store-ports/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"store-ports-macros\", \"name\": \"store-ports-macros\", \"manifest_path\": \"/w/crates/store-ports-macros/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"proc-macro\"]}]},\
     {\"id\": \"design-store\", \"name\": \"design-store\", \"manifest_path\": \"/w/crates/design-store/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"sqlx\", \"name\": \"sqlx\", \"manifest_path\": \"/r/sqlx/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"sqlx-core\", \"name\": \"sqlx-core\", \"manifest_path\": \"/r/sqlx-core/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"sqlx-macros-core\", \"name\": \"sqlx-macros-core\", \"manifest_path\": \"/r/sqlx-macros-core/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"crc\", \"name\": \"crc\", \"manifest_path\": \"/r/crc/Cargo.toml\", \"dependencies\": []}\
     ], \"resolve\": {\"nodes\": [\
     {\"id\": \"store-ports\", \"dependencies\": [\"sqlx\", \"store-ports-macros\"]},\
     {\"id\": \"store-ports-macros\", \"dependencies\": [\"sqlx-macros-core\", \"sqlx-core\"]},\
     {\"id\": \"design-store\", \"dependencies\": [\"store-ports\"]},\
     {\"id\": \"sqlx\", \"dependencies\": [\"sqlx-core\"]},\
     {\"id\": \"sqlx-macros-core\", \"dependencies\": [\"sqlx-core\", \"crc\"]}\
     ]}}",
);
const SQLITE_USER: Source = Source::new(
    Located::new("crates/design-store/Cargo.toml"),
    Text::new("[package]\nname = \"design-store\"\n\n[dependencies]\nlibsqlite3-sys = \"0.30\"\n"),
);
const STORE_BYPASSED: Text = Text::new(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"store-ports\", \"store-ports-macros\", \"design-store\"], \"packages\": [\
     {\"id\": \"store-ports\", \"name\": \"store-ports\", \"manifest_path\": \"/w/crates/store-ports/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"store-ports-macros\", \"name\": \"store-ports-macros\", \"manifest_path\": \"/w/crates/store-ports-macros/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"proc-macro\"]}]},\
     {\"id\": \"design-store\", \"name\": \"design-store\", \"manifest_path\": \"/w/crates/design-store/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"sqlx\", \"name\": \"sqlx\", \"manifest_path\": \"/r/sqlx/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"sqlx-core\", \"name\": \"sqlx-core\", \"manifest_path\": \"/r/sqlx-core/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"sqlx-macros-core\", \"name\": \"sqlx-macros-core\", \"manifest_path\": \"/r/sqlx-macros-core/Cargo.toml\", \"dependencies\": []},\
     {\"id\": \"libsqlite3-sys\", \"name\": \"libsqlite3-sys\", \"manifest_path\": \"/r/libsqlite3-sys/Cargo.toml\", \"dependencies\": []}\
     ], \"resolve\": {\"nodes\": [\
     {\"id\": \"store-ports\", \"dependencies\": [\"sqlx\", \"store-ports-macros\"]},\
     {\"id\": \"store-ports-macros\", \"dependencies\": [\"sqlx-macros-core\", \"sqlx-core\"]},\
     {\"id\": \"design-store\", \"dependencies\": [\"store-ports\", \"libsqlite3-sys\"]},\
     {\"id\": \"sqlx\", \"dependencies\": [\"sqlx-core\", \"libsqlite3-sys\"]}\
     ]}}",
);
const STORE_WITHOUT_MACROS: Text = Text::new(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"store-ports\", \"store-ports-macros\", \"design-store\"], \"packages\": [\
     {\"id\": \"store-ports\", \"name\": \"store-ports\", \"manifest_path\": \"/w/crates/store-ports/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"store-ports-macros\", \"name\": \"store-ports-macros\", \"manifest_path\": \"/w/crates/store-ports-macros/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"design-store\", \"name\": \"design-store\", \"manifest_path\": \"/w/crates/design-store/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]}\
     ], \"resolve\": {\"nodes\": [\
     {\"id\": \"store-ports\", \"dependencies\": [\"store-ports-macros\"]}\
     ]}}",
);
const STORE_UNDEPENDED: Text = Text::new(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"store-ports\", \"store-ports-macros\", \"design-store\"], \"packages\": [\
     {\"id\": \"store-ports\", \"name\": \"store-ports\", \"manifest_path\": \"/w/crates/store-ports/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]},\
     {\"id\": \"store-ports-macros\", \"name\": \"store-ports-macros\", \"manifest_path\": \"/w/crates/store-ports-macros/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"proc-macro\"]}]},\
     {\"id\": \"design-store\", \"name\": \"design-store\", \"manifest_path\": \"/w/crates/design-store/Cargo.toml\", \"dependencies\": [], \"targets\": [{\"kind\": [\"lib\"]}]}\
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

struct Owning;

impl Owning {
    fn flagged(manifests: &[Source]) -> Lines {
        Sources::placed_in::<OwnedDependencies>(manifests, CRATES, EMPTY)
    }
}

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

    fn reported(&self) -> Lines {
        Sources::placed_in::<DominatedPackages>(self.manifests, self.crates, self.workspace)
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
        [OWNER, CO_OWNER] => ["crates/design/Cargo.toml:8", "crates/grammar/Cargo.toml:8"],
        [OWNER, CO_OWNER, THIRD] => ["crates/design/Cargo.toml:8", "crates/grammar/Cargo.toml:8", "crates/store/Cargo.toml:5"],
    ]
);

laws::table!(
    only_declared_packages_are_owned_and_a_declared_member_is_flagged_outside_its_owner,
    Placed::flagged,
    [
        Placed::new(&[FAMILY_OWNER, SIBLING], CRATES, LOGOS) => ["crates/design/Cargo.toml:5"],
        Placed::new(&[FAMILY_OWNER], CRATES, LOGOS) => Lines::default(),
        Placed::new(&[OWNER, SIBLING], CRATES, LOGOS) => Lines::default(),
        Placed::new(&[OWNER, HASHBROWN_USER], CRATES, LOGOS_AND_HASHBROWN) => Lines::default(),
        Placed::new(&[HASHBROWN_OWNER, HASHBROWN_USER], CRATES, LOGOS) => ["crates/design/Cargo.toml:5"],
    ]
);

laws::table!(
    a_port_and_the_proc_macro_crate_it_depends_on_own_jointly,
    Placed::flagged,
    [
        Placed::new(&[PORT, MACROS], UNIT_CRATES, STORE) => Lines::default(),
        Placed::new(&[PORT_ON_MACRO_FAMILY, MACROS], UNIT_CRATES, STORE) => Lines::default(),
        Placed::new(&[PORT, MACROS, STRANGER], UNIT_CRATES, STORE) => ["crates/design-store/Cargo.toml:5"],
        Placed::new(&[PORT, MACROS], UNIT_CRATES, STORE_WITHOUT_MACROS) => ["crates/store-ports-macros/Cargo.toml:9"],
        Placed::new(&[PORT_ON_MACRO_FAMILY, MACROS], UNIT_CRATES, STORE_WITHOUT_MACROS) => ["crates/store-ports/Cargo.toml:8", "crates/store-ports-macros/Cargo.toml:9"],
        Placed::new(&[PORT, MACROS], UNIT_CRATES, STORE_UNDEPENDED) => ["crates/store-ports-macros/Cargo.toml:9"],
    ]
);

laws::table!(
    a_crate_outside_a_unit_is_reported_where_it_depends_on_an_undeclared_package_only_that_units_packages_reach,
    Placed::reported,
    [
        Placed::new(&[OWNER], CRATES, LOGOS) => Lines::default(),
        Placed::new(&[OWNER, HASHBROWN_USER], CRATES, LOGOS_AND_HASHBROWN) => ["crates/design/Cargo.toml:5"],
        Placed::new(&[HASHBROWN_OWNER, HASHBROWN_USER], CRATES, LOGOS_AND_HASHBROWN) => Lines::default(),
        Placed::new(&[UNOWNED, HASHBROWN_USER], CRATES, LOGOS_AND_HASHBROWN) => Lines::default(),
        Placed::new(&[OWNER, ELSEWHERE], CRATES, LOGOS) => Lines::default(),
        Placed::new(&[OWNER], CRATES, EMPTY) => Lines::default(),
    ]
);

laws::table!(
    a_crate_depending_on_what_a_port_unit_alone_reaches_is_reported_at_that_dependency,
    Placed::reported,
    [
        Placed::new(&[PORT, MACROS, SQLITE_USER], UNIT_CRATES, STORE_BYPASSED) => ["crates/design-store/Cargo.toml:5"],
        Placed::new(&[PORT, MACROS], UNIT_CRATES, STORE_BYPASSED) => Lines::default(),
        Placed::new(&[PORT, MACROS], UNIT_CRATES, STORE) => Lines::default(),
        Placed::new(&[PORT, MACROS], UNIT_CRATES, STORE_UNDEPENDED) => Lines::default(),
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
