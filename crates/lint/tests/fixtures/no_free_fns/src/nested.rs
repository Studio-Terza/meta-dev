pub struct A;
impl A {
    pub fn run() {
        fn helper() {}
        helper()
    }
}
const _: () = {
    fn hidden() {}
};
