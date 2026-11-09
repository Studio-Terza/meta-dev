use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SenseKind {
    #[name("decision")]
    Decision,
    #[name("opinion")]
    Opinion,
    #[name("question")]
    Question,
    #[name("ask")]
    Ask,
    #[name("instruction")]
    Instruction,
    #[name("control")]
    Control,
    #[name("remark")]
    Remark,
    #[name("smell")]
    Smell,
}
