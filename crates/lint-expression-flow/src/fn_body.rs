use syn::{Block, Signature};

use crate::Flows;

pub enum FnBody<'ast> {
    Scanned(&'ast Block),
    Skipped,
}

impl<'ast> FnBody<'ast> {
    pub fn scanned_unless_const(signature: &Signature, block: Option<&'ast Block>) -> FnBody<'ast> {
        match (signature.constness, block) {
            (None, Some(block)) => FnBody::Scanned(block),
            _ => FnBody::Skipped,
        }
    }

    pub fn walked_by(self, flows: &mut Flows<'_>) {
        match self {
            FnBody::Scanned(body) => flows.visit_body(body),
            FnBody::Skipped => {}
        }
    }
}
