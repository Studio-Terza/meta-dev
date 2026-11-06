use lint_fixtures::{Lines, Located, Source, Sources, Text};
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
    Text::new("fn f() {\n    sqlx::query!(\"SELECT id FROM nodes\");\n    query!(\"SELECT id FROM nodes\");\n}\n"),
)];

laws::table!(
    a_store_query_is_flagged_unless_its_sql_is_as_sqlformat_formats_it_at_column_zero_inside_a_raw_string,
    Sources::flagged::<StoreQueries>,
    [
        FORMATTED => Lines::default(),
        WRITTEN => Lines::default(),
        ONE_LINE => ["crates/store/src/lib.rs:2"],
        RAW_LINE => ["crates/store/src/lib.rs:2"],
        ESCAPED => ["crates/store/src/lib.rs:2"],
        INDENTED => ["crates/store/src/lib.rs:4"],
        NESTED => ["crates/store/src/lib.rs:2"],
        OTHER => Lines::default(),
    ]
);
