use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entered {
    Init,
    Apply,
    Role,
    Inheritance,
    Stage,
}
