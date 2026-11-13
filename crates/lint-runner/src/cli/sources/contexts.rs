use lint_core::Parsed;
use lint_enum_predicates::EnumPredicates;
use lint_file::ContextLint;
use lint_guard_must_use::GuardMustUse;
use lint_module_nesting::ModuleNesting;
use lint_named_numbers::NamedNumbers;
use lint_no_forwarders::NoForwarders;
use lint_no_parameter_bags::NoParameterBags;
use lint_public_surface::PublicSurface;
use lint_repeated_methods::RepeatedMethods;
use lint_unconsumed_traits::UnconsumedTraits;

pub(crate) trait Holding<'run, Lint: ContextLint> {
    fn held(&self) -> &Lint::Context<'run>;
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

        impl<'run> From<Parsed<'_, 'run>> for Contributed<'run> {
            fn from(parsed: Parsed<'_, 'run>) -> Contributed<'run> {
                Contributed { $($field: <$lint as ContextLint>::contributed(parsed),)* }
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
            impl<'run> Holding<'run, $lint> for Contexts<'run> {
                fn held(&self) -> &<$lint as ContextLint>::Context<'run> {
                    &self.$field
                }
            }
        )*
    };
}

contexts! {
    named_numbers: NamedNumbers,
    public_surface: PublicSurface,
    no_forwarders: NoForwarders,
    unconsumed_traits: UnconsumedTraits,
    guard_must_use: GuardMustUse,
    enum_predicates: EnumPredicates,
    module_nesting: ModuleNesting,
    no_parameter_bags: NoParameterBags,
    repeated_methods: RepeatedMethods,
}
