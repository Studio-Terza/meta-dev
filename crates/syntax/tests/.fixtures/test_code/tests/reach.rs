mod outer {
    #[path = "../../src/lib.rs"]
    mod inner;
}
#[path = "helpers/common.rs"]
mod common;
#[path = "./../src/lib.rs"]
mod direct;
macro_rules! listed {
    () => {
        #[test_case]
        fn listed() {}
    };
}
