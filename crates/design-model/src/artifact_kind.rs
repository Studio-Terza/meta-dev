use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactKind {
    #[name("lint")]
    Lint,
    #[name("hook")]
    Hook,
    #[name("tool")]
    Tool,
    #[name("behaviour")]
    Behaviour,
}
