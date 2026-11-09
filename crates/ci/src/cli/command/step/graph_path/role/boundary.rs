use grammar::unit_label::{Label, Mode, Target, Unlabeled};
use serde::Deserialize;

use super::Role;

#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
pub(in super::super) struct Labeled(Label);

impl TryFrom<String> for Labeled {
    type Error = Unlabeled;

    fn try_from(text: String) -> Result<Labeled, Unlabeled> {
        text.parse().map(Labeled)
    }
}

impl Role {
    fn targeted(target: Target) -> Role {
        match target.mode() {
            Mode::Check => Role::Target(target),
            Mode::CheckTest => Role::TestTarget(target),
        }
    }
}

impl From<Labeled> for Role {
    fn from(Labeled(label): Labeled) -> Role {
        match label {
            Label::Built => Role::HostBuild,
            Label::Checked => Role::Library,
            Label::CheckedTest => Role::LibraryTest,
            Label::BuildScript => Role::BuildScript,
            Label::BuildRun => Role::BuildRun,
            Label::Target(target) => Role::targeted(target),
        }
    }
}
