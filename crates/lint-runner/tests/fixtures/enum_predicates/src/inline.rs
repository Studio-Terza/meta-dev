mod inner {
    pub enum Kind { A }
    impl Kind {
        pub fn is_a(&self) -> bool { true }
    }
}
