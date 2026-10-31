use serde::{Deserialize, Serialize};

/// Which line of the side file an imported escape came from: the 64-bit FNV-1a hash of the line's
/// bytes. It is the same in every build, so an import cut short by a crash is recognised on retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LineHash(u64);

impl LineHash {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    pub fn of(line: &[u8]) -> Self {
        Self(line.iter().fold(Self::OFFSET_BASIS, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(Self::PRIME)))
    }
}

#[cfg(test)]
mod tests {
    use super::LineHash;

    #[test]
    fn the_hash_is_the_published_fnv_1a_of_the_bytes() {
        assert_eq!((LineHash::of(b"").0, LineHash::of(b"a").0), (0xcbf2_9ce4_8422_2325, 0xaf63_dc4c_8601_ec8c));
    }
}
