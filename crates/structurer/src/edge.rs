#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge<Source, Target = Source> {
    source: Source,
    target: Target,
}

#[derive(Debug, thiserror::Error)]
#[error("an edge cannot join an end to itself")]
pub struct Looped;

impl<Source: Clone, Target: From<Source> + PartialEq> Edge<Source, Target> {
    pub fn new(source: Source, target: Target) -> Result<Edge<Source, Target>, Looped> {
        let itself = Target::from(source.clone());
        Some(target).filter(|target| *target != itself).map(|target| Edge { source, target }).ok_or(Looped)
    }
}

impl<Source, Target> Edge<Source, Target> {
    pub fn ends<Output>(&self, read: impl FnOnce(&Source, &Target) -> Output) -> Output {
        read(&self.source, &self.target)
    }
}
