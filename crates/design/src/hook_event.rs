use named_derive::Named;
use serde::Serialize;

#[derive(Named, Serialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[named(kebab)]
pub enum HookEvent {
    UserPromptSubmit,
    Stop,
}
