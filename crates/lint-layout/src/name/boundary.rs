use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    static WRITTEN: RefCell<String> = const { RefCell::new(String::new()) };
}

#[derive(Debug)]
pub(super) struct Spelling;

impl Spelling {
    pub(super) fn written(fill: impl FnOnce(&mut String)) -> Arc<str> {
        WRITTEN.with_borrow_mut(|written| {
            written.clear();
            fill(written);
            Arc::from(written.as_str())
        })
    }
}
