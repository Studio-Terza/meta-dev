use lint_fixtures::{Lines, Located, Queried, Source, Sources, Text};
use lint_store_queries::StoreQueries;

const FORMATTED: &[Source] = &[Source::new(
    Located::new("crates/store/src/lib.rs"),
    Text::new(
        "fn f() {\n    s.rows(store_ports::query!(\n        Row,\n        r#\"\nSELECT\n  id\nFROM\n  nodes\n\"#,\n        id: NodeId\n    ));\n}\n",
    ),
)];
const WRITTEN: &[Source] = &[Source::new(
    Located::new("crates/store/src/lib.rs"),
    Text::new(
        "fn f() {\n    s.written(store_ports::query!(\n        r#\"\nSELECT\n  id\nFROM\n  nodes\n\"#\n    ));\n}\n",
    ),
)];
const ONE_LINE: &[Source] = &[Source::new(
    Located::new("crates/store/src/lib.rs"),
    Text::new("fn f() {\n    s.rows(store_ports::query!(Row, \"SELECT id FROM nodes\", id: NodeId));\n}\n"),
)];
const RAW_LINE: &[Source] = &[Source::new(
    Located::new("crates/store/src/lib.rs"),
    Text::new("fn f() {\n    s.rows(store_ports::query!(Row, r#\"SELECT id FROM nodes\"#));\n}\n"),
)];
const ESCAPED: &[Source] = &[Source::new(
    Located::new("crates/store/src/lib.rs"),
    Text::new("fn f() {\n    s.rows(store_ports::query!(Row, \"\\nSELECT\\n  id\\nFROM\\n  nodes\\n\"));\n}\n"),
)];
const INDENTED: &[Source] = &[Source::new(
    Located::new("crates/store/src/lib.rs"),
    Text::new(
        "fn f() {\n    s.rows(store_ports::query!(\n        Row,\n        r#\"\n        SELECT\n          id\n        FROM\n          nodes\n        \"#\n    ));\n}\n",
    ),
)];
const NESTED: &[Source] = &[Source::new(
    Located::new("crates/store/src/lib.rs"),
    Text::new("fn f() {\n    vec![store_ports::query!(Row, \"SELECT id FROM nodes\")];\n}\n"),
)];
const OTHER: &[Source] = &[Source::new(
    Located::new("crates/store/src/lib.rs"),
    Text::new(
        "use store_ports::query;\nfn f() {\n    sqlx::query!(\"SELECT id FROM nodes\");\n    query!(\"SELECT id FROM nodes\");\n}\n",
    ),
)];
const RENAMED: &[Source] = &[Source::new(
    Located::new("crates/store/src/lib.rs"),
    Text::new("use store_ports::query as q;\nfn f() {\n    s.rows(q!(Row, \"select id from nodes\"));\n}\n"),
)];
const NESTED_RENAMED: &[Source] = &[Source::new(
    Located::new("crates/store/src/lib.rs"),
    Text::new("use store_ports::query as q;\nfn f() {\n    vec![q!(\"select id from nodes\")];\n}\n"),
)];
const GLOBBED: &[Source] = &[
    Source::new(
        Located::new("crates/store/src/lib.rs"),
        Text::new("use store_ports::*;\nfn f() {\n    s.rows(query!(Row, \"select id from nodes\"));\n}\n"),
    ),
    Source::new(Located::new("crates/store-ports/src/lib.rs"), Text::new("pub use store_ports_macros::query;\n")),
    Source::new(
        Located::new("crates/store-ports-macros/src/lib.rs"),
        Text::new("#[proc_macro]\npub fn query(input: TokenStream) -> TokenStream {\n    input\n}\n"),
    ),
];
const LOOKALIKE: &[Source] = &[
    Source::new(
        Located::new("crates/store/src/lib.rs"),
        Text::new("use rows::*;\nfn f() {\n    s.rows(query!(Row, \"select id from nodes\"));\n}\n"),
    ),
    Source::new(
        Located::new("crates/rows/src/lib.rs"),
        Text::new("#[proc_macro]\npub fn query(input: TokenStream) -> TokenStream {\n    input\n}\n"),
    ),
];
const FROM_THE_MACROS_CRATE: &[Source] = &[Source::new(
    Located::new("crates/store/src/lib.rs"),
    Text::new("fn f() {\n    s.rows(store_ports_macros::query!(Row, \"SELECT id FROM nodes\"));\n}\n"),
)];
const UNIMPORTED: &[Source] = &[Source::new(
    Located::new("crates/store/src/lib.rs"),
    Text::new("fn f() {\n    query!(\"SELECT id FROM nodes\");\n}\n"),
)];
const IMPORTED_IN_MODULE: &[Source] = &[Source::new(
    Located::new("crates/store/src/lib.rs"),
    Text::new(
        "mod rows {\n    use store_ports::query;\n    fn f() {\n        query!(\"SELECT id FROM nodes\");\n    }\n}\nfn g() {\n    query!(\"SELECT id FROM nodes\");\n}\n",
    ),
)];

laws::table!(
    a_store_query_is_flagged_unless_its_sql_is_as_sqlformat_formats_it_at_column_zero_inside_a_raw_string,
    Sources::flagged::<Queried<StoreQueries>>,
    [
        FORMATTED => Lines::default(),
        WRITTEN => Lines::default(),
        ONE_LINE => Lines::from(["crates/store/src/lib.rs:2"]),
        RAW_LINE => Lines::from(["crates/store/src/lib.rs:2"]),
        ESCAPED => Lines::from(["crates/store/src/lib.rs:2"]),
        INDENTED => Lines::from(["crates/store/src/lib.rs:4"]),
        NESTED => Lines::from(["crates/store/src/lib.rs:2"]),
        OTHER => Lines::from(["crates/store/src/lib.rs:4"]),
        RENAMED => Lines::from(["crates/store/src/lib.rs:3"]),
        NESTED_RENAMED => Lines::from(["crates/store/src/lib.rs:3"]),
        GLOBBED => Lines::from(["crates/store/src/lib.rs:3"]),
        LOOKALIKE => Lines::default(),
        FROM_THE_MACROS_CRATE => Lines::from(["crates/store/src/lib.rs:2"]),
        UNIMPORTED => Lines::default(),
        IMPORTED_IN_MODULE => Lines::from(["crates/store/src/lib.rs:4"]),
    ]
);
