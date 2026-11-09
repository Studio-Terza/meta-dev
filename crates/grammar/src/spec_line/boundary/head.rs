use logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Head {
    #[token("repair: main")]
    Repair,
    #[token("spec: ")]
    Spec,
}
