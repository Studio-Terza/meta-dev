#[path = "generated/laws.rs"]
mod generated_laws;

use design_graph::{AppliedFileRow, AppliedFileRowNew};
use design_messages::StreamSchema;
use design_model::{Entered, Provenance};
use file_ports::Bytes;
use laws::proptest::prelude::{Strategy, any};
use laws::proptest::sample::select;
use newtype::Enumerated;
use sqlite_ports::SqliteOpener;
use store_ports::Opens;

laws::crud!(
    an_applied_file_keeps_the_crud_laws,
    AppliedFileRow,
    (any::<Bytes>(), select(Entered::VARIANTS.to_vec()).prop_map(Provenance::from))
        .prop_map(|(content, provenance)| AppliedFileRowNew { content, provenance }),
    SqliteOpener.in_memory::<StreamSchema>(),
);
