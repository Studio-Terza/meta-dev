use std::sync::Arc;

use pipeline::Measure;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, Expr, ExprAssign, ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprMethodCall, ExprWhile, Field,
    ImplItemConst, ImplItemFn, Item, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStruct, ItemTrait,
    ItemType, ItemUse, LifetimeParam, Local, Macro, Path, Signature, TraitItemFn, Type, TypePath, TypeTraitObject,
    TypeTuple, VisRestricted,
};

use crate::budget::{Budget, TestBudget};
use crate::expansion::Expansion;
use crate::finding::{Finding, Findings};
use crate::gate::Gate;
use crate::hooks::{Descent, Expanded, Hooks, nodes};
use crate::lint::{Fact, Lint, Reads};
use crate::lints::{
    AdapterFacts, AmbientReads, ApiFacts, BoolMethods, ExpressionFlow, HandMeasures, ManualIteration, ModuleCycles,
    ModuleFacts, NamedLifetimes, NoComments, NoDyn, NoFreeFns, NoTestOnlyApi, OptoutFacts, ProcessStarters,
    PublicApiTests, PublicSurface, RealAdapters, SurfaceFacts, TargetOptouts, TestOnlyFacts, TypeShape, VecOutParams,
};
use crate::source::Source;
use crate::workspace::Package;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Statements(Option<()>),
    Substituted,
    Path,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Awake,
    PausedAt(usize),
    Off,
}

#[derive(Debug)]
pub(crate) struct Pausable<Lint> {
    lint: Lint,
    state: State,
    skips: Option<()>,
}

#[derive(Debug)]
pub(crate) struct Walker<'file> {
    lineup: Lineup<'file>,
    depth: usize,
}

impl Phase {
    fn admitted(self, expanded: Expanded) -> Option<()> {
        match (expanded, self) {
            (Expanded::Statements, Phase::Statements(_))
            | (Expanded::Surface, Phase::Statements(None) | Phase::Substituted | Phase::Path)
            | (Expanded::Path, Phase::Path) => Some(()),
            _ => None,
        }
    }
}

impl<Walked> Pausable<Walked> {
    fn admitted(lint: Walked, reads: Reads, source: &Source) -> Pausable<Walked> {
        let state = reads.admits(source).map_or(State::Off, |()| State::Awake);
        Pausable { lint, state, skips: reads.skips() }
    }

    fn woken(self) -> Option<Walked> {
        self.active().map(|()| self.lint)
    }

    fn active(&self) -> Option<()> {
        (self.state == State::Awake).then_some(())
    }

    fn entered(&mut self, depth: usize, hook: impl FnOnce(&mut Walked) -> Descent) {
        let over = self.active().map(|()| hook(&mut self.lint)).filter(|descent| *descent == Descent::Over);
        self.state = over.map_or(self.state, |_| State::PausedAt(depth));
    }

    fn gated(&mut self, depth: usize, gate: Gate, hook: impl FnOnce(&mut Walked) -> Descent) {
        let skipped = self.skips.and(Reads::skipped(gate)).and(self.active());
        match skipped {
            Some(()) => self.state = State::PausedAt(depth),
            None => self.entered(depth, hook),
        }
    }

    fn left(&mut self, depth: usize, hook: impl FnOnce(&mut Walked)) {
        match self.state {
            State::Awake => hook(&mut self.lint),
            State::PausedAt(at) => self.state = if at == depth { State::Awake } else { self.state },
            State::Off => {}
        }
    }

    fn chose(&mut self, hook: impl FnOnce(&mut Walked) -> Expanded) -> Option<Expanded> {
        self.active().map(|()| hook(&mut self.lint))
    }

    fn held(&mut self, depth: usize, wanted: Option<()>) {
        let unwanted = self.active().and(wanted.xor(Some(())));
        self.state = unwanted.map_or(self.state, |()| State::PausedAt(depth));
    }
}

impl<'file> Walker<'file> {
    pub(crate) fn walked(source: &'file Source, file: &syn::File) -> Facts {
        let mut walker = Walker { lineup: Lineup::admitted(source), depth: 0 };
        walker.visit_file(file);
        walker.lineup.facts(source)
    }

    pub(crate) fn headlines() -> Vec<&'static str> {
        Lineup::headlines()
    }

    fn deeper(&mut self, walk: impl FnOnce(&mut Self, usize)) {
        self.depth += 1;
        walk(self, self.depth);
        self.depth -= 1;
    }
}

macro_rules! each {
    ($lineup:expr, [$($lint:ident: $kind:ty => $fact:ty),* $(,)?], $method:ident $arguments:tt) => {
        $( $lineup.$lint.$method $arguments; )*
    };
}

macro_rules! lineup {
    ($life:lifetime [$($lint:ident: $kind:ty => $fact:ty),* $(,)?]) => {
        #[derive(Debug)]
        struct Lineup<$life> {
            $($lint: Pausable<$kind>,)*
        }

        #[derive(Clone, Copy, Debug)]
        struct Chosen {
            $($lint: Option<Expanded>,)*
        }

        #[derive(Debug, Measure)]
        pub(crate) struct Facts {
            $(pub $lint: $fact,)*
        }

        impl<$life> Lineup<$life> {
            fn admitted(source: &$life Source) -> Lineup<$life> {
                Lineup {
                    $($lint: Pausable::admitted(
                        <$kind as Lint<$life>>::new(source),
                        <$kind as Lint<$life>>::READS,
                        source,
                    ),)*
                }
            }

            fn facts(self, source: &$life Source) -> Facts {
                let Lineup { $($lint),* } = self;
                Facts { $($lint: $lint.woken().map_or_default(|lint| lint.fact(source)),)* }
            }

            fn headlines() -> Vec<&'static str> {
                vec![$(<$kind as Lint<$life>>::HEADLINE),*]
            }

            fn awake(&self) -> Option<()> {
                [$(self.$lint.active()),*].into_iter().flatten().next()
            }

            fn chose(&mut self, invocation: &Macro) -> Chosen {
                Chosen { $($lint: self.$lint.chose(|lint| Hooks::macro_called(lint, invocation)),)* }
            }

            fn held(&mut self, depth: usize, chosen: Chosen, phase: Phase) {
                $(self.$lint.held(depth, chosen.$lint.and_then(|choice| phase.admitted(choice)));)*
            }

            fn released(&mut self, depth: usize) {
                $(self.$lint.left(depth, |_| {});)*
            }
        }

        impl Facts {
            pub(crate) fn findings(self, packages: &[Arc<Package>]) -> Vec<Finding> {
                let Facts { $($lint),* } = self;
                [$(Fact::findings($lint, packages)),*].into_iter().flatten().collect()
            }
        }

        impl Chosen {
            fn wanting(self, phase: Phase) -> Option<()> {
                [$(self.$lint.and_then(|choice| phase.admitted(choice))),*].into_iter().flatten().next()
            }
        }
    };
}

macro_rules! walker {
    (
        $life:lifetime;
        lints $lints:tt
        nodes [$($visit:ident($node:ty) $hook:ident $left:ident),* $(,)?]
        gated [$($gated_visit:ident($gated_node:ty) $gated_hook:ident $gated_left:ident),* $(,)?]
    ) => {
        lineup!($life $lints);

        impl Walker<'_> {
            fn phased(&mut self, chosen: Chosen, phase: Phase, walk: impl FnOnce(&mut Self)) {
                self.deeper(|walker, depth| {
                    walker.lineup.held(depth, chosen, phase);
                    walk(walker);
                    walker.lineup.released(depth);
                });
            }
        }

        impl<'ast> Visit<'ast> for Walker<'_> {
            $(
                fn $visit(&mut self, node: &'ast $node) {
                    self.deeper(|walker, depth| {
                        each!(walker.lineup, $lints, entered(depth, |lint| Hooks::$hook(lint, node)));
                        walker.lineup.awake().map_or_default(|()| visit::$visit(walker, node));
                        each!(walker.lineup, $lints, left(depth, |lint| Hooks::$left(lint, node)));
                    });
                }
            )*

            $(
                fn $gated_visit(&mut self, node: &'ast $gated_node) {
                    let gate = Gate::of(&node.attrs);
                    self.deeper(|walker, depth| {
                        each!(walker.lineup, $lints, gated(depth, gate, |lint| Hooks::$gated_hook(lint, node)));
                        walker.lineup.awake().map_or_default(|()| visit::$gated_visit(walker, node));
                        each!(walker.lineup, $lints, left(depth, |lint| Hooks::$gated_left(lint, node)));
                    });
                }
            )*

            fn visit_macro(&mut self, node: &'ast Macro) {
                let chosen = self.lineup.chose(node);
                let defining = Expansion::defining(node);
                let statements = Phase::Statements(defining);
                chosen.wanting(statements).map_or_default(|()| {
                    let expansion = Expansion::of(node);
                    self.phased(chosen, statements, |walker| expansion.walk(walker));
                });
                chosen.wanting(Phase::Substituted).and(defining).map_or_default(|()| {
                    let expansion = Expansion::substituted(node);
                    self.phased(chosen, Phase::Substituted, |walker| expansion.walk(walker));
                });
                chosen.wanting(Phase::Path).map_or_default(|()| {
                    self.phased(chosen, Phase::Path, |walker| visit::visit_macro(walker, node));
                });
            }
        }
    };
}

nodes!(walker {
    'file;
    lints [
        no_comments: NoComments => Findings,
        no_free_fns: NoFreeFns => Findings,
        vec_out_params: VecOutParams => Findings,
        named_lifetimes: NamedLifetimes => Findings,
        type_shape: TypeShape => Findings,
        expression_flow: ExpressionFlow => Findings,
        manual_iteration: ManualIteration => Findings,
        process_starters: ProcessStarters => Findings,
        public_surface: PublicSurface<'file> => SurfaceFacts,
        public_api_tests: PublicApiTests<'file> => ApiFacts,
        no_test_only_api: NoTestOnlyApi<'file> => TestOnlyFacts,
        bool_methods: BoolMethods => Findings,
        no_dyn: NoDyn => Findings,
        real_adapters: RealAdapters<'file> => AdapterFacts,
        module_cycles: ModuleCycles<'file> => ModuleFacts,
        hand_measures: HandMeasures => Findings,
        target_optouts: TargetOptouts => OptoutFacts,
        ambient_reads: AmbientReads => Findings,
        test_budget: TestBudget => Budget,
    ]
});
