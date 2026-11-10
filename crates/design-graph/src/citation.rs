mod boundary;

use design_model::Provenance;
use transcript_ports::{Authorship, Uuid};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Citation {
    message: Uuid,
    author: Authorship,
    provenance: Provenance,
}

impl AsRef<Uuid> for Citation {
    fn as_ref(&self) -> &Uuid {
        &self.message
    }
}

impl AsRef<Authorship> for Citation {
    fn as_ref(&self) -> &Authorship {
        &self.author
    }
}

impl AsRef<Provenance> for Citation {
    fn as_ref(&self) -> &Provenance {
        &self.provenance
    }
}
