#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Breadth {
    #[default]
    Crate,
    Workspace,
}
