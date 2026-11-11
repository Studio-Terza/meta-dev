use named_derive::Named;
use serde::Serialize;

#[derive(Named, Serialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HookEvent {
    #[name("user-prompt-submit")]
    UserPromptSubmit,
    #[name("stop")]
    Stop,
}
