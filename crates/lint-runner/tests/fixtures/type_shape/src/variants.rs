pub enum Cmd {
    Set { on: bool },
    Pair((u8, u8)),
    Bad(Result<u8, String>),
}
