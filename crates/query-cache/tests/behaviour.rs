use query_cache::{Entry, Origin, Table};

const TWO_PARAMETERS: &str = r#"{"describe": {"columns": [], "parameters": {"Right": 2}, "nullable": []}}"#;

struct Binding(usize);

impl Binding {
    fn refused(&self) -> Option<String> {
        serde_json::from_str::<Entry<String>>(TWO_PARAMETERS)
            .ok()
            .and_then(|entry| entry.bound(self.0).err())
            .map(|refusal| refusal.to_string())
    }
}

laws::table!(
    a_cached_query_binds_only_as_many_values_as_it_takes,
    Binding::refused,
    [
        Binding(2) => None,
        Binding(3) => Some(String::from("the query takes 2 parameters but 3 are bound")),
        Binding(1) => Some(String::from("the query takes 2 parameters but 1 are bound")),
    ]
);

laws::table!(
    a_record_table_spells_its_four_statements,
    Table::statements,
    [
        Table::named("r").column("id", Origin::GeneratedKey).column("body", Origin::Given).column("at", Origin::Generated) => [
            String::from(r#"INSERT INTO r (body) VALUES (?1) RETURNING id AS "id!", body AS "body!", at AS "at!""#),
            String::from(r#"SELECT id AS "id!", body AS "body!", at AS "at!" FROM r WHERE id = ?1"#),
            String::from("UPDATE r SET body = ?1, at = ?2 WHERE id = ?3"),
            String::from("DELETE FROM r WHERE id = ?1"),
        ],
        Table::named("s").column("id", Origin::GeneratedKey) => [
            String::from(r#"INSERT INTO s DEFAULT VALUES RETURNING id AS "id!""#),
            String::from(r#"SELECT id AS "id!" FROM s WHERE id = ?1"#),
            String::from("UPDATE s SET id = id WHERE id = ?1"),
            String::from("DELETE FROM s WHERE id = ?1"),
        ],
    ]
);
