pub trait Schema: 'static {
    const SQL: &'static str;
}
