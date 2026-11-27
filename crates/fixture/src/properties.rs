use proptest::test_runner::{Config, RngSeed};

#[derive(Debug)]
pub struct Properties;

impl Properties {
    const SEED: u64 = 0;

    #[must_use]
    pub fn settled() -> Config {
        Config { failure_persistence: None, rng_seed: RngSeed::Fixed(Properties::SEED), ..Config::default() }
    }
}
