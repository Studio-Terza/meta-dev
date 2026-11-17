use crate::Watching;
use crate::drawn::Drawn;
use crate::hooks::{Fold, Folding, Kind};
use crate::scoped::{Attaches, Attachment};
use crate::sweep::Gathers;

pub trait Attaching<Group: Fold>: Fold {
    const ATTACHED: &'static [Kind];

    fn attaching(&self, found: Self::Found) -> Drawn<Self::Found, Vec<Attachment<Group>>>;

    fn detached(&self, found: Self::Found, group: Group::Found) -> Self::Found;
}

pub trait AttachingGroups<Group>: Fold {
    const ATTACHED: &'static [Kind];

    fn attaching(&self, found: Self::Found) -> Drawn<Self::Found, Vec<Attachment<Group>>>;

    fn detached(&self, found: Self::Found, group: Group) -> Self::Found;
}

#[derive(Debug)]
pub struct Grouped<Group: Fold>(Folding<Group>);

impl<Group: Fold> Gathers<Grouped<Group>> for Grouped<Group> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        std::iter::once::<&mut Watching<'_>>(&mut self.0)
    }

    fn gathered(self) -> Grouped<Group> {
        self
    }
}

impl<Group: Fold> From<Attachment<Group>> for Attachment<Grouped<Group>> {
    fn from(attachment: Attachment<Group>) -> Attachment<Grouped<Group>> {
        attachment.regrouped(|group| Grouped(Folding::from(group)))
    }
}

impl<Parent: Attaching<Group>, Group: Fold> AttachingGroups<Grouped<Group>> for Parent {
    const ATTACHED: &'static [Kind] = <Parent as Attaching<Group>>::ATTACHED;

    fn attaching(&self, found: Parent::Found) -> Drawn<Parent::Found, Vec<Attachment<Grouped<Group>>>> {
        Attaching::attaching(self, found).mapped(|attached| attached.into_iter().map(Attachment::from).collect())
    }

    fn detached(&self, found: Parent::Found, group: Grouped<Group>) -> Parent::Found {
        Attaching::detached(self, found, group.0.gathered())
    }
}

impl<Parent: AttachingGroups<Group>, Group: Gathers<Group>> Attaches<Group> for Folding<Parent> {
    const ATTACHED: &'static [Kind] = Parent::ATTACHED;

    fn attaching(&mut self) -> Vec<Attachment<Group>> {
        self.drawn(AttachingGroups::attaching)
    }

    fn detached(&mut self, group: Group) {
        self.left(|fold, found| fold.detached(found, group));
    }
}
