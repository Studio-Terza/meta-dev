use std::any::Any;

pub struct Held(pub Box<dyn Any>);

impl Held {
    pub fn named(&self) -> std::any::TypeId {
        self.0.type_id()
    }

    pub fn read(self) -> Option<u8> {
        self.0.downcast::<u8>().ok().map(|boxed| *boxed)
    }
}

pub const SPELLED: &str = "Any";
