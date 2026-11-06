mod admitted;
mod bound;
mod bounded;
mod complexity;
mod failure;
mod gate;
mod growth;
mod harness;
mod passed;
mod scale;
mod verified;
mod within;
mod workload;

pub use admitted::Admitted;
pub use bound::Bound;
pub use bounded::Bounded;
pub use complexity::Complexity;
pub use failure::Failure;
pub use gate::Gate;
pub use growth::Growth;
pub use harness::Harness;
pub use passed::Passed;
pub use scale::Scale;
pub use verified::Verified;
pub use within::Within;
pub use workload::Workload;

#[macro_export]
macro_rules! workload {
    ($($workload:ty),+ $(,)?) => {
        fn main() -> ::std::process::ExitCode {
            $crate::Gate::answer(|gate| gate$(.passing::<$workload>())+)
        }
    };
}
