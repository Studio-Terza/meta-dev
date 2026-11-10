use query_cache::Entry;

const TWO_PARAMETERS: &str = r#"{"describe": {"columns": [], "parameters": {"Right": 2}, "nullable": []}}"#;

struct Binding(usize);

impl Binding {
    fn refused(&self) -> Option<String> {
        serde_json::from_str::<Entry>(TWO_PARAMETERS)
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
