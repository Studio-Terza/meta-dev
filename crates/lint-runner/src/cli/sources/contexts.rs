use super::gathering::Gathering;
use lint_core::SourceFile;
use lint_core::{Parsed, Unwalked};
use lint_derived_tables::DerivedTables;
use lint_enum_predicates::EnumPredicates;
use lint_file::{ContextLint, Gleaning, PrimedFacts};
use lint_guard_must_use::GuardMustUse;
use lint_module_nesting::ModuleNesting;
use lint_named_numbers::NamedNumbers;
use lint_no_forwarders::NoForwarders;
use lint_no_test_only_api::NoTestOnlyApi;
use lint_port_dependencies::RoleReading;
use lint_public_api_tests::PublicApiTests;
use lint_public_surface::PublicSurface;
use lint_repeated_shapes::RepeatedShapes;
use lint_state_graph::StateGraph;
use lint_sweep::{Gathers, Sweep};
use lint_unconsumed_traits::UnconsumedTraits;

pub(crate) trait Holding<'run, Lint: ContextLint> {
    fn held(&self) -> &Lint::Context<'run>;
}

pub(crate) trait Priming<'run, Lint: PrimedFacts> {
    fn primer(&self) -> &Lint::Primer<'run>;

    fn primed(&self) -> &Lint::Context<'run>;
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Settled<'settled, 'run> {
    contexts: &'settled Contexts<'run>,
}

macro_rules! contexts {
    (judged: { $($field:ident: $lint:ty),* $(,)? } primed: { $($primed:ident: $prime:ty),* $(,)? } gleaned: { $($gleaned:ident: $glean:ty),* $(,)? }) => {
        #[derive(Debug)]
        pub(crate) struct Contexts<'run> {
            $($field: <$lint as ContextLint>::Context<'run>,)*
            $($primed: <$prime as PrimedFacts>::Primer<'run>,)*
            $($gleaned: <$glean as Gleaning>::Gleaned<'run>,)*
            primings: Primings<'run>,
        }

        #[derive(Debug)]
        pub(crate) struct Contributed<'run> {
            $($field: <$lint as ContextLint>::Part<'run>,)*
            $($primed: <$prime as PrimedFacts>::PrimerPart<'run>,)*
            $($gleaned: <$glean as Gleaning>::Part<'run>,)*
            recorded: Recorded<'run>,
        }

        #[derive(Debug, Default)]
        struct Parts<'run> {
            $($field: Vec<<$lint as ContextLint>::Part<'run>>,)*
            $($primed: Vec<<$prime as PrimedFacts>::PrimerPart<'run>>,)*
            $($gleaned: Vec<<$glean as Gleaning>::Part<'run>>,)*
            raws: Raws<'run>,
        }

        #[derive(Debug)]
        pub(crate) struct Recorded<'run> {
            $($primed: <$prime as PrimedFacts>::Raw<'run>,)*
        }

        #[derive(Debug, Default)]
        struct Raws<'run> {
            $($primed: Vec<<$prime as PrimedFacts>::Raw<'run>>,)*
        }

        struct Recordings<'file, 'run: 'file> {
            $($primed: <$prime as PrimedFacts>::Recording<'file, 'run>,)*
        }

        struct Gatherings<'file, 'run: 'file> {
            $($field: <$lint as ContextLint>::Gathering<'run>,)*
            $($primed: <$prime as PrimedFacts>::Priming<'run>,)*
            $($gleaned: <$glean as Gleaning>::Gleaner<'file, 'run>,)*
        }

        impl<'run> Contributed<'run> {
            pub(crate) fn joined(parsed: Parsed<'_, 'run>, unwalked: Unwalked) -> Contributed<'run> {
                let gatherings = Gatherings {
                    $($field: <$lint as ContextLint>::gathering(parsed),)*
                    $($primed: <$prime as PrimedFacts>::priming(parsed),)*
                    $($gleaned: <$glean as Gleaning>::gleaner(parsed),)*
                };
                let recordings = Recordings { $($primed: <$prime as PrimedFacts>::recording(parsed),)* };
                Sweep::once(unwalked, <&SourceFile>::from(parsed), Joint { gatherings, recordings })
            }
        }

        struct Joint<'file, 'run: 'file> {
            gatherings: Gatherings<'file, 'run>,
            recordings: Recordings<'file, 'run>,
        }

        impl<'file, 'run: 'file> Gathers<Contributed<'run>> for Joint<'file, 'run> {
            fn gathered(self, file: &SourceFile) -> Contributed<'run> {
                let Joint { gatherings, recordings } = self;
                Contributed {
                    $($field: gatherings.$field.gathered(file),)*
                    $($primed: gatherings.$primed.gathered(file),)*
                    $($gleaned: gatherings.$gleaned.gathered(file),)*
                    recorded: Recorded { $($primed: recordings.$primed.gathered(file),)* },
                }
            }
        }

        impl<'run> FromIterator<Contributed<'run>> for Contexts<'run> {
            fn from_iter<Found: IntoIterator<Item = Contributed<'run>>>(found: Found) -> Contexts<'run> {
                let split = found.into_iter().fold(Parts::default(), |mut split, part| {
                    $(split.$field.push(part.$field);)*
                    $(split.$primed.push(part.$primed);)*
                    $(split.$gleaned.push(part.$gleaned);)*
                    $(split.raws.$primed.push(part.recorded.$primed);)*
                    split
                });
                $(let $primed: <$prime as PrimedFacts>::Primer<'run> = split.$primed.into_iter().collect();)*
                let primings = Primings {
                    $($primed: split.raws.$primed.into_iter().map(|raw| <$prime as PrimedFacts>::contribution(raw, &$primed)).collect(),)*
                };
                Contexts {
                    $($field: split.$field.into_iter().collect(),)*
                    $($primed,)*
                    $($gleaned: split.$gleaned.into_iter().collect(),)*
                    primings,
                }
            }
        }

        $(
            impl<'run> Holding<'run, $lint> for Settled<'_, 'run> {
                fn held(&self) -> &<$lint as ContextLint>::Context<'run> {
                    &self.contexts.$field
                }
            }
        )*

        impl<'settled, 'run> Settled<'settled, 'run> {
            $(
                pub(crate) fn $gleaned(self) -> &'settled <$glean as Gleaning>::Gleaned<'run> {
                    &self.contexts.$gleaned
                }
            )*
        }

        #[derive(Debug)]
        pub(crate) struct Primings<'run> {
            $($primed: <$prime as PrimedFacts>::Context<'run>,)*
        }

        $(
            impl<'run> Priming<'run, $prime> for Settled<'_, 'run> {
                fn primer(&self) -> &<$prime as PrimedFacts>::Primer<'run> {
                    &self.contexts.$primed
                }

                fn primed(&self) -> &<$prime as PrimedFacts>::Context<'run> {
                    &self.contexts.primings.$primed
                }
            }
        )*
    };
}

contexts! {
    judged: {
        named_numbers: NamedNumbers,
        public_surface: PublicSurface,
        no_forwarders: NoForwarders,
        unconsumed_traits: UnconsumedTraits,
        guard_must_use: GuardMustUse,
        derived_tables: DerivedTables,
        enum_predicates: EnumPredicates,
        module_nesting: ModuleNesting,
        public_api_tests: PublicApiTests,
        no_test_only_api: NoTestOnlyApi,
        repeated_shapes: RepeatedShapes,
    }
    primed: {
        state_graph: StateGraph,
    }
    gleaned: {
        roles: RoleReading,
    }
}

impl<'settled, 'run> From<&'settled Gathering<'run>> for Settled<'settled, 'run> {
    fn from(gathering: &'settled Gathering<'run>) -> Settled<'settled, 'run> {
        Settled { contexts: gathering.contexts() }
    }
}
