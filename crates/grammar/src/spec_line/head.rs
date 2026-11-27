use grammar_core::logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar_core::logos)]
pub(super) enum Head {
    #[token("repair: main")]
    Repair,
    #[token("spec: ")]
    Spec,
}
