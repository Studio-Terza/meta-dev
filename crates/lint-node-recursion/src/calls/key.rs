use graph::{Function, Invocation, Named, Placed, Seat, TypeRef};
use lint_layout::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Key<'model> {
    krate: &'model Name,
    owner: Option<&'model Name>,
    name: &'model Name,
}

impl<'model> From<Placed<'model, &'model Function>> for Key<'model> {
    fn from(placed: Placed<'model, &'model Function>) -> Key<'model> {
        let function = placed.entity();
        let seat: &Seat = function.as_ref();
        let owner = seat.subject().and_then(TypeRef::named).and_then(Named::last_spelled);
        Key { krate: placed.krate(), owner, name: function.as_ref() }
    }
}

impl<'model> Key<'model> {
    pub(super) fn called(self, invocation: &'model Invocation) -> Option<Key<'model>> {
        let owner = invocation.owner(self.owner, &[]).known()?;
        Some(Key { krate: self.krate, owner: owner.type_name(), name: invocation.as_ref() })
    }
}
