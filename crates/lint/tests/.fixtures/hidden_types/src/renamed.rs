pub type Count = u8;

pub struct Counted(pub Count);

pub trait Counter {
    type Each;
}

impl Counter for Counted {
    type Each = Count;
}
