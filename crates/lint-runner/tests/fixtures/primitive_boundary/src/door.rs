struct A;
impl std::str::FromStr for A {
    type Err = ();
    fn from_str(x: &str) -> Result<A, ()> {
        Ok(A).map(|a| x.trim())
    }
}
