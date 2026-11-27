use crate::HookEvent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ingested,
    Settings,
    Hooked(HookEvent),
    Blocked,
    Trailed,
    Walked,
    Paired,
    Verified,
    Backtested,
}
