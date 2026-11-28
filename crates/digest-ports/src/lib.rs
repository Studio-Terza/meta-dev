use hex::FromHex;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Digested([u8; 32]);

#[derive(Debug)]
pub struct Hex;

impl Digested {
    #[must_use]
    pub fn of(bytes: &[u8]) -> Digested {
        Digested(Sha256::digest(bytes).into())
    }

    #[must_use]
    pub fn bytes(self) -> [u8; 32] {
        self.0
    }
}

impl Hex {
    #[must_use]
    pub fn encoded(bytes: impl AsRef<[u8]>) -> String {
        hex::encode(bytes)
    }

    #[must_use]
    pub fn decoded<const COUNT: usize>(text: &str) -> Option<[u8; COUNT]>
    where
        [u8; COUNT]: FromHex,
    {
        <[u8; COUNT]>::from_hex(text).into_iter().next()
    }
}
