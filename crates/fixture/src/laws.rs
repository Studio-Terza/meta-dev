use std::fmt::Debug;

use test_failure::Failed;

use crate::boundary::Disagreed;

#[derive(Debug)]
pub struct Laws;

impl Laws {
    pub fn agreed<Call: Debug, Reply: Debug + PartialEq>(
        calls: &[Call],
        implementor: impl Fn(&Call) -> Reply,
        reference: impl Fn(&Call) -> Reply,
    ) -> Result<(), Failed> {
        calls
            .iter()
            .map(|call| (call, implementor(call), reference(call)))
            .find(|(_, implemented, referenced)| implemented != referenced)
            .map_or(Ok(()), |(call, implemented, referenced)| {
                Err(Failed::from(Disagreed::between(call, &implemented, &referenced)))
            })
    }
}
