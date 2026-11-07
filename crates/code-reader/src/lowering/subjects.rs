use code_model::{Region, Seat, TypeRef};

use crate::namespace::{Resolving, ScopeId};

use super::Lowering;
use super::signature::Written;

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
    pub(super) fn settled(self, resolving: &Resolving<'_, '_, '_>) -> Subjects {
        Subjects(self.0.into_iter().map(|row| Written::typed(&row.ty, resolving, row.scope)).collect())
    }
}

#[derive(Debug)]
pub(super) struct Subjects(Vec<TypeRef>);

impl Subjects {
    pub(super) fn seat(&self, region: Region, subject: Option<SubjectId>) -> Seat {
        Seat::new(region, subject.and_then(|subject| self.0.get(subject.0)).cloned())
    }
}

impl Lowering<'_, '_> {
    pub(super) fn subjected(&mut self, ty: TypeRef<Written>) -> SubjectId {
        let rows = &mut self.lowered.subjects.0;
        let subject = SubjectId(rows.len());
        rows.push(SubjectRow { scope: self.here.scope(), ty });
        subject
    }
}
