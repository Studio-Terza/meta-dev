use sha2::{Digest, Sha256};
use store_ports::Schema;

#[derive(Clone, Copy, Debug)]
pub(in super::super) struct Stamped {
    pub(super) sql: &'static str,
    pub(super) stamp: i32,
}

impl Stamped {
    pub(in super::super) fn of<S: Schema>() -> Stamped {
        let digest = Sha256::digest(S::SQL.as_bytes());
        let [first, second, third, fourth, ..]: [u8; 32] = digest.into();
        Stamped { sql: S::SQL, stamp: i32::from_be_bytes([first, second, third, fourth]) }
    }
}
