use syn::ItemImpl;
use syn::visit;

use code_model::{Region, Seat, TypeRef};

use crate::namespace::{Resolving, ScopeId};

use super::signature::Written;
use super::{Enclosure, ImplRow, Lowering};

#[derive(Clone, Copy, Debug)]
pub(super) struct SubjectId(usize);

#[derive(Debug)]
struct SubjectRow {
    scope: ScopeId,
    ty: TypeRef<Written>,
}

#[derive(Debug, Default)]
pub(super) struct SubjectRows(Vec<SubjectRow>);

impl SubjectRows {
    pub(super) fn settled(&self, resolving: &Resolving<'_, '_, '_>) -> Subjects {
        Subjects(self.0.iter().map(|row| Written::typed(&row.ty, resolving, row.scope)).collect())
    }
}

#[derive(Debug)]
pub(super) struct Subjects(Vec<TypeRef>);

impl Subjects {
    pub(super) fn seat(&self, region: Region, subject: Option<SubjectId>) -> Seat {
        Seat::new(region, subject.and_then(|id| self.0.get(id.0)).cloned())
    }
}

impl Lowering<'_> {
    pub(super) fn subjected(&mut self, ty: TypeRef<Written>) -> SubjectId {
        let rows = &mut self.lowered.rows.subjects.0;
        let subject = SubjectId(rows.len());
        rows.push(SubjectRow { scope: self.here.scope(), ty });
        subject
    }

    pub(super) fn impl_visited(&mut self, block: &ItemImpl) {
        let declared = self.generics.extended(&block.generics);
        let subject = declared.lowered(&block.self_ty);
        let generics = declared.subject(subject.clone());
        self.lowered.rows.impls.push(ImplRow::written(&self.here, self.context.region(), block, subject.clone()));
        let subjected = self.subjected(subject);
        let context = self.context.of_impl(block, subjected);
        self.contextual(context, |lowering| {
            lowering.enclosed(Enclosure::of_impl(block), generics, |enclosed| visit::visit_item_impl(enclosed, block));
        });
    }
}
