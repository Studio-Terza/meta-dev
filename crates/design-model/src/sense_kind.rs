use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SenseKind {
    Decision,
    Opinion,
    Question,
    Ask,
    Instruction,
    Control,
    Remark,
    Smell,
}
