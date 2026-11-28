use sha2::{Digest, Sha256};
use store_ports::Schema;

#[derive(Clone, Debug)]
pub(in super::super) struct Stamped {
    schema: Schema,
    stamp: i32,
}

impl From<&Schema> for Stamped {
    fn from(schema: &Schema) -> Stamped {
        let digest = schema
            .statements()
            .fold(Sha256::new(), |hashed, statement| hashed.chain_update(statement.as_ref()).chain_update([u8::MIN]))
            .finalize();
        let [first, second, third, fourth, ..]: [u8; 32] = digest.into();
        Stamped { schema: schema.clone(), stamp: i32::from_be_bytes([first, second, third, fourth]) }
    }
}

impl Stamped {
    pub(super) fn schema(&self) -> &Schema {
        &self.schema
    }

    pub(super) fn stamp(&self) -> i32 {
        self.stamp
    }
}
