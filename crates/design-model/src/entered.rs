use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entered {
    #[name("init")]
    Init,
    #[name("apply")]
    Apply,
    #[name("role")]
    Role,
    #[name("inheritance")]
    Inheritance,
    #[name("stage")]
    Stage,
}
