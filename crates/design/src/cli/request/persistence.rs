#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Persistence {
    #[default]
    File,
    DryRun,
}
