use lint_core::Parsed;
use lint_enum_predicates::EnumPredicates;
use lint_fakes_at_ports::FakesAtPorts;
use lint_file::{ContextLint, PrimedLint};
use lint_guard_must_use::GuardMustUse;
use lint_mandatory_laws::MandatoryLaws;
use lint_module_nesting::ModuleNesting;
use lint_named_numbers::NamedNumbers;
use lint_no_forwarders::NoForwarders;
use lint_no_parameter_bags::NoParameterBags;
use lint_no_test_only_api::NoTestOnlyApi;
use lint_port_batching::PortBatching;
use lint_port_dependencies::EffectsInFillers;
use lint_public_api_tests::PublicApiTests;
use lint_public_surface::PublicSurface;
use lint_repeated_methods::RepeatedMethods;
use lint_repeated_shapes::RepeatedShapes;
use lint_state_graph::StateGraph;
use lint_unconsumed_traits::UnconsumedTraits;

pub(crate) trait Holding<'run, Lint: ContextLint> {
    fn held(&self) -> &Lint::Context<'run>;
}

pub(crate) trait Priming<'run, Lint: PrimedLint> {
    fn primer(&self) -> &Lint::Primer<'run>;

    fn primed(&self) -> &Lint::Context<'run>;
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Settled<'settled, 'run> {
    contexts: &'settled Contexts<'run>,
    primings: &'settled Primings<'run>,
}

impl<'settled, 'run> Settled<'settled, 'run> {
    pub(crate) fn new(
        contexts: &'settled Contexts<'run>,
        primings: &'settled Primings<'run>,
    ) -> Settled<'settled, 'run> {
        Settled { contexts, primings }
    }
}

macro_rules! contexts {
    (judged: { $($field:ident: $lint:ty),* $(,)? } primed: { $($primed:ident: $prime:ty),* $(,)? }) => {
        #[derive(Debug)]
        pub(crate) struct Contexts<'run> {
            $($field: <$lint as ContextLint>::Context<'run>,)*
            $($primed: <$prime as PrimedLint>::Primer<'run>,)*
        }

        #[derive(Debug)]
        pub(crate) struct Contributed<'run> {
            $($field: <$lint as ContextLint>::Part<'run>,)*
            $($primed: <$prime as PrimedLint>::PrimerPart<'run>,)*
        }

        #[derive(Debug, Default)]
        struct Parts<'run> {
            $($field: Vec<<$lint as ContextLint>::Part<'run>>,)*
            $($primed: Vec<<$prime as PrimedLint>::PrimerPart<'run>>,)*
        }

        impl<'run> From<Parsed<'_, 'run>> for Contributed<'run> {
            fn from(parsed: Parsed<'_, 'run>) -> Contributed<'run> {
                Contributed {
                    $($field: <$lint as ContextLint>::contributed(parsed),)*
                    $($primed: <$prime as PrimedLint>::primed(parsed),)*
                }
            }
        }

        impl<'run> FromIterator<Contributed<'run>> for Contexts<'run> {
            fn from_iter<Found: IntoIterator<Item = Contributed<'run>>>(found: Found) -> Contexts<'run> {
                let split = found.into_iter().fold(Parts::default(), |mut split, part| {
                    $(split.$field.push(part.$field);)*
                    $(split.$primed.push(part.$primed);)*
                    split
                });
                Contexts {
                    $($field: split.$field.into_iter().collect(),)*
                    $($primed: split.$primed.into_iter().collect(),)*
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

        #[derive(Debug)]
        pub(crate) struct Primings<'run> {
            $($primed: <$prime as PrimedLint>::Context<'run>,)*
        }

        #[derive(Debug)]
        pub(crate) struct PrimedParts<'run> {
            $($primed: <$prime as PrimedLint>::Part<'run>,)*
        }

        #[derive(Debug, Default)]
        struct Primeds<'run> {
            $($primed: Vec<<$prime as PrimedLint>::Part<'run>>,)*
        }

        impl<'run> Contexts<'run> {
            pub(crate) fn priming(&self, parsed: Parsed<'_, 'run>) -> PrimedParts<'run> {
                PrimedParts { $($primed: <$prime as PrimedLint>::contributed(parsed, &self.$primed),)* }
            }
        }

        impl<'run> FromIterator<PrimedParts<'run>> for Primings<'run> {
            fn from_iter<Found: IntoIterator<Item = PrimedParts<'run>>>(found: Found) -> Primings<'run> {
                let split = found.into_iter().fold(Primeds::default(), |mut split, part| {
                    $(split.$primed.push(part.$primed);)*
                    split
                });
                Primings { $($primed: split.$primed.into_iter().collect(),)* }
            }
        }

        $(
            impl<'run> Priming<'run, $prime> for Settled<'_, 'run> {
                fn primer(&self) -> &<$prime as PrimedLint>::Primer<'run> {
                    &self.contexts.$primed
                }

                fn primed(&self) -> &<$prime as PrimedLint>::Context<'run> {
                    &self.primings.$primed
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
        enum_predicates: EnumPredicates,
        module_nesting: ModuleNesting,
        no_parameter_bags: NoParameterBags,
        repeated_methods: RepeatedMethods,
        public_api_tests: PublicApiTests,
        no_test_only_api: NoTestOnlyApi,
        mandatory_laws: MandatoryLaws,
        repeated_shapes: RepeatedShapes,
    }
    primed: {
        state_graph: StateGraph,
        fakes_at_ports: FakesAtPorts,
        port_batching: PortBatching,
        effects_in_fillers: EffectsInFillers,
    }
}
