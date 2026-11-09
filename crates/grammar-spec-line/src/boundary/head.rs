use grammar::logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
pub(super) enum Head {
    #[token("repair: main")]
    Repair,
    #[token("spec: ")]
    Spec,
}
