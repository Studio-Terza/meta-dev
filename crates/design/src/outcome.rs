use design_model::{Role, Stage};
use design_store::Earlier;

use crate::{Count, HookEvent};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ingested,
    Shown,
    Searched,
    Initialised,
    Applied,
    AlreadyApplied { earlier: Earlier },
    Moved(Stage),
    Recast(Role),
    Structured { jobs: Count },
    Settings,
    Hooked(HookEvent),
    Blocked,
    Noticed,
    Told,
    Viewed,
    Imported,
}
