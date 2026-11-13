use lint_layout::Name;
use workspace_metadata::CrateName;

#[derive(Debug)]
pub(super) struct Underscored;

impl Underscored {
    const HYPHEN: char = '-';
    const UNDERSCORE: char = '_';

    pub(super) fn crate_named(name: &CrateName) -> Name {
        let spelled: String = name
            .as_ref()
            .chars()
            .map(|letter| Some(letter).filter(|found| *found != Self::HYPHEN).unwrap_or(Self::UNDERSCORE))
            .collect();
        Name::from(spelled.as_str())
    }
}
