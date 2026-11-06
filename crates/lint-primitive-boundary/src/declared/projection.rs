use syn::{ImplItem, ImplItemType, PathSegment, TypePath};

use crate::owner::Owner;
use crate::reach::Reach;
use crate::spelling::Spelled;
use crate::use_aliases::UseAliases;
use lint_core::Syntax;
use lint_layout::Name;

#[derive(Debug, PartialEq, Eq)]
pub struct Slot {
    owner: Name,
    interface: Name,
    item: Name,
}

impl Slot {
    pub fn of_path(path: &TypePath) -> Option<Slot> {
        path.qself.as_ref().and_then(|qself| {
            let segments: Vec<&PathSegment> = path.path.segments.iter().collect();
            let owner = Syntax::type_path(&qself.ty).and_then(|ty| ty.path.segments.last());
            let interface = qself.position.checked_sub(1).and_then(|index| segments.get(index));
            let item = segments.get(qself.position..).and_then(|rest| rest.first().xor(rest.get(1)));
            owner.zip(interface).zip(item).map(|((owner, interface), item)| Slot {
                owner: Name::from(&owner.ident),
                interface: Name::from(&interface.ident),
                item: Name::from(&item.ident),
            })
        })
    }
}

#[derive(Debug)]
pub struct Projection {
    slot: Slot,
    spelled: Spelled,
}

impl Projection {
    pub fn of_member(owner: &Owner, interface: Option<&PathSegment>, member: &ImplItem) -> Option<Projection> {
        match member {
            ImplItem::Type(associated) => Self::of_type(owner, interface, associated),
            _ => None,
        }
    }

    pub fn slot(&self) -> &Slot {
        &self.slot
    }

    pub fn spelled(&self) -> &Spelled {
        &self.spelled
    }

    fn of_type(owner: &Owner, interface: Option<&PathSegment>, associated: &ImplItemType) -> Option<Projection> {
        let spelled =
            Syntax::type_path(&associated.ty).and_then(|target| UseAliases::default().spelled(target, Reach::Widened));
        owner.name().zip(interface).zip(spelled).map(|((owner, interface), spelled)| Projection {
            slot: Slot {
                owner: owner.clone(),
                interface: Name::from(&interface.ident),
                item: Name::from(&associated.ident),
            },
            spelled,
        })
    }
}
