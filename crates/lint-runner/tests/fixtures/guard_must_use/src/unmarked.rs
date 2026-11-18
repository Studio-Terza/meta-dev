pub struct Lock;
impl Drop for Lock {
    fn drop(&mut self) {}
}

pub mod plain {
    pub struct Lock;
}
