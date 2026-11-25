mod boundary;

use port_core::Op;
use store_ports::Version;

#[derive(Debug, Op)]
pub struct Anchored;

impl Anchored {
    pub(super) const PENDING: Version = Version::new(20_261_109_000_001);
}
