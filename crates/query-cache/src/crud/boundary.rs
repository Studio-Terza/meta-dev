use serde::Deserialize;
use strum::EnumString;

#[derive(EnumString, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case", parse_err_ty = Unplaced, parse_err_fn = Unplaced::from)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Key,
    GeneratedKey,
    Given,
    Generated,
}

impl Origin {
    pub const INSERTED: [Origin; 2] = [Origin::Key, Origin::Given];
    pub const KEYS: [Origin; 2] = [Origin::Key, Origin::GeneratedKey];
    pub const ASSIGNED: [Origin; 2] = [Origin::Given, Origin::Generated];
}

#[derive(Debug, thiserror::Error)]
#[error("an origin is key, generated_key, given or generated, not {0:?}")]
pub struct Unplaced(String);

impl From<&str> for Unplaced {
    fn from(name: &str) -> Unplaced {
        Unplaced(name.to_owned())
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct Field {
    name: String,
    origin: Origin,
}

#[derive(Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Table {
    name: String,
    fields: Vec<Field>,
}

impl Table {
    #[must_use]
    pub fn named(name: &str) -> Table {
        Table { name: name.to_owned(), fields: Vec::new() }
    }

    #[must_use]
    pub fn column(mut self, field: &str, origin: Origin) -> Table {
        self.fields.push(Field { name: field.to_owned(), origin });
        self
    }

    #[must_use]
    pub fn statements(&self) -> [String; 5] {
        let Table { name, fields } = self;
        let named = |wanted: &[Origin]| -> Vec<&str> {
            fields.iter().filter(|field| wanted.contains(&field.origin)).map(|field| field.name.as_str()).collect()
        };
        let given = named(&Origin::INSERTED);
        let unkeyed = named(&Origin::ASSIGNED);
        let key = named(&Origin::KEYS).join("");
        let returned =
            fields.iter().map(|field| format!("{0} AS \"{0}!\"", field.name)).collect::<Vec<String>>().join(", ");
        let placed = |count: usize| (1..=count).map(|at| format!("?{at}")).collect::<Vec<String>>().join(", ");
        let assigned: Vec<String> =
            unkeyed.iter().enumerate().map(|(at, column)| format!("{column} = ?{}", at + 1)).collect();
        [
            given.first().map_or_else(
                || format!("INSERT INTO {name} DEFAULT VALUES RETURNING {returned}"),
                |_| {
                    format!(
                        "INSERT INTO {name} ({}) VALUES ({}) RETURNING {returned}",
                        given.join(", "),
                        placed(given.len())
                    )
                },
            ),
            format!("SELECT {returned} FROM {name} WHERE {key} = ?1"),
            format!(
                "UPDATE {name} SET {} WHERE {key} = ?{}",
                assigned.first().map_or_else(|| format!("{key} = {key}"), |_| assigned.join(", ")),
                unkeyed.len() + 1
            ),
            format!("DELETE FROM {name} WHERE {key} = ?1"),
            format!("SELECT {returned} FROM {name}"),
        ]
    }

    #[must_use]
    pub fn overwrite(&self, column: &str) -> String {
        format!("UPDATE {} SET {column} = ?1", self.name)
    }
}
