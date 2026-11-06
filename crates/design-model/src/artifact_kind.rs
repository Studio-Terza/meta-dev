use named_derive::Named;
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
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
