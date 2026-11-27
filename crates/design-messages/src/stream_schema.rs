use store_ports::Schema;

#[derive(Clone, Copy, Debug)]
pub struct StreamSchema;

impl Schema for StreamSchema {
    const SQL: &'static str = include_str!("schema.sql");
}
