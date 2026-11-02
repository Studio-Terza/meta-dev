mod ports;
#[test]
fn reads_the_environment() {
    std::env::var("A").ok();
}
