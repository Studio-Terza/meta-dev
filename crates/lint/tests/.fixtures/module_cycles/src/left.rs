use std::fmt;

pub(crate) struct Left(crate::Shared, crate::facade::Only);

mod inner {
    pub(super) struct Inner(super::Left);
}
