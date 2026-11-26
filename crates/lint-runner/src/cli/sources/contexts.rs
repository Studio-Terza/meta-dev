use super::gathering::Gathering;
use lint_core::SourceFile;
use lint_core::{Parsed, Unwalked};
use lint_enum_predicates::EnumPredicates;
use lint_file::ContextLint;
use lint_named_numbers::NamedNumbers;
use lint_no_forwarders::NoForwarders;
use lint_no_test_only_api::NoTestOnlyApi;
use lint_public_api_tests::PublicApiTests;
use lint_public_surface::PublicSurface;
use lint_sweep::{Gathers, Sweep};

pub(crate) trait Holding<'run, Lint: ContextLint> {
    fn held(&self) -> &Lint::Context<'run>;
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Settled<'settled, 'run> {
    contexts: &'settled Contexts<'run>,
}

macro_rules! contexts {
    ($($field:ident: $lint:ty),* $(,)?) => {
        #[derive(Debug)]
        pub(crate) struct Contexts<'run> {
            $($field: <$lint as ContextLint>::Context<'run>,)*
        }

        #[derive(Debug)]
        pub(crate) struct Contributed<'run> {
            $($field: <$lint as ContextLint>::Part<'run>,)*
        }

        #[derive(Debug, Default)]
        struct Parts<'run> {
            $($field: Vec<<$lint as ContextLint>::Part<'run>>,)*
        }

        struct Gatherings<'run> {
            $($field: <$lint as ContextLint>::Gathering<'run>,)*
        }

        impl<'run> Contributed<'run> {
            pub(crate) fn joined(parsed: Parsed<'_, 'run>, unwalked: Unwalked) -> Contributed<'run> {
                let gatherings = Gatherings { $($field: <$lint as ContextLint>::gathering(parsed),)* };
                Sweep::once(unwalked, <&SourceFile>::from(parsed), gatherings)
            }
        }

        impl<'run> Gathers<Contributed<'run>> for Gatherings<'run> {
            fn gathered(self, file: &SourceFile) -> Contributed<'run> {
                Contributed { $($field: self.$field.gathered(file),)* }
            }
        }

        impl<'run> FromIterator<Contributed<'run>> for Contexts<'run> {
            fn from_iter<Found: IntoIterator<Item = Contributed<'run>>>(found: Found) -> Contexts<'run> {
                let split = found.into_iter().fold(Parts::default(), |mut split, part| {
                    $(split.$field.push(part.$field);)*
                    split
                });
                Contexts { $($field: split.$field.into_iter().collect(),)* }
            }
        }

        $(
            impl<'run> Holding<'run, $lint> for Settled<'_, 'run> {
                fn held(&self) -> &<$lint as ContextLint>::Context<'run> {
                    &self.contexts.$field
                }
            }
        )*
    };
}

contexts! {
    named_numbers: NamedNumbers,
    public_surface: PublicSurface,
    no_forwarders: NoForwarders,
    enum_predicates: EnumPredicates,
    public_api_tests: PublicApiTests,
    no_test_only_api: NoTestOnlyApi,
}

impl<'settled, 'run> From<&'settled Gathering<'run>> for Settled<'settled, 'run> {
    fn from(gathering: &'settled Gathering<'run>) -> Settled<'settled, 'run> {
        Settled { contexts: gathering.contexts() }
    }
}
