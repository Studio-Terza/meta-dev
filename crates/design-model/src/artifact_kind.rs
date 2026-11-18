use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactKind {
    Lint,
    Hook,
    Tool,
    Behaviour,
}
