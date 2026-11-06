mod bound;
mod complexity;
mod failure;
mod harness;
mod scale;
mod workload;

pub use bound::Bound;
pub use complexity::Complexity;
pub use failure::Failure;
pub use harness::Harness;
pub use scale::Scale;
pub use workload::Workload;

#[macro_export]
macro_rules! workload {
    ($($workload:ty),+ $(,)?) => {
        fn main() -> ::core::result::Result<(), $crate::Failure> {
            $($crate::Harness::<$workload>::verified()?;)+
            Ok(())
        }
    };
}
