use serde_json::{Map, Value};
use store::Value as Stored;

pub struct Loose {
    pub held: Value,
    pub rest: Map<String, Value>,
    pub table: toml::de::DeTable<'static>,
    pub stored: Stored,
}

impl Loose {
    pub fn absent() -> Value {
        Value::Null
    }
}

mod boundary {
    pub struct Kept(pub serde_json::Value);
}
