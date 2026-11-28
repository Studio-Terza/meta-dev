use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, Expr, ExprAssign, ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprMethodCall, ExprWhile, Field,
    ImplItemConst, ImplItemFn, ImplItemType, Item, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod,
    ItemStatic, ItemStruct, ItemTrait, ItemType, ItemUnion, ItemUse, LifetimeParam, Lit, Local, Macro, Path, Signature,
    TraitItemFn, TraitItemType, Type, TypePath, TypeTraitObject, TypeTuple, VisRestricted,
};

use crate::budget::TestScan;
use crate::expansion::Expansion;
use crate::hooks::{Descent, Expanded, Hooks, nodes};
use crate::lints::{
    BoolMethods, DeadCode, Emitters, ExpressionFlow, ManualIteration, ModuleCycles, NamedLifetimes, NoDyn, NoFreeFns,
    NoTestOnlyApi, PrimitiveBoundary, ProcessStarters, PublicApiTests, TestShape, TypeShape, VecOutParams,
};
use crate::terms::Depth;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Statements(Option<()>),
    Substituted,
    Path,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Awake,
    PausedAt(Depth),
    Off,
}

#[derive(Debug)]
pub(crate) struct Pausable<Lint> {
    lint: Lint,
    state: State,
}

#[derive(Debug)]
pub(crate) struct Walker<'file> {
    lineup: Lineup<'file>,
    depth: Depth,
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

impl<Lint> Pausable<Lint> {
    pub(crate) fn admitted(lint: Lint, admitted: Option<()>) -> Pausable<Lint> {
        Pausable { lint, state: admitted.map_or(State::Off, |()| State::Awake) }
    }

    pub(crate) fn woken(self) -> Option<Lint> {
        self.active().map(|()| self.lint)
    }

    fn active(&self) -> Option<()> {
        (self.state == State::Awake).then_some(())
    }

    fn entered(&mut self, depth: Depth, hook: impl FnOnce(&mut Lint) -> Descent) {
        let over = self.active().map(|()| hook(&mut self.lint)).filter(|descent| *descent == Descent::Over);
        self.state = over.map_or(self.state, |_| State::PausedAt(depth));
    }

    fn left(&mut self, depth: Depth, hook: impl FnOnce(&mut Lint)) {
        match self.state {
            State::Awake => hook(&mut self.lint),
            State::PausedAt(at) => self.state = if at == depth { State::Awake } else { self.state },
            State::Off => {}
        }
    }

    fn chose(&mut self, hook: impl FnOnce(&mut Lint) -> Expanded) -> Option<Expanded> {
        self.active().map(|()| hook(&mut self.lint))
    }

    fn held(&mut self, depth: Depth, wanted: Option<()>) {
        let unwanted = self.active().and(wanted.xor(Some(())));
        self.state = unwanted.map_or(self.state, |()| State::PausedAt(depth));
    }
}

impl Walker<'_> {
    pub(crate) fn walked<'file>(lineup: Lineup<'file>, file: &syn::File) -> Lineup<'file> {
        let mut walker = Walker { lineup, depth: Depth::default() };
        walker.visit_file(file);
        walker.lineup
    }

    fn deeper(&mut self, walk: impl FnOnce(&mut Self, Depth)) {
        self.depth = self.depth.deeper();
        walk(self, self.depth);
        self.depth = self.depth.shallower();
    }
}

macro_rules! each {
    ($lineup:expr, [$($lint:ident: $kind:ty),* $(,)?], $method:ident $arguments:tt) => {
        $( $lineup.$lint.$method $arguments; )*
    };
}

macro_rules! lineup {
    ($life:lifetime [$($lint:ident: $kind:ty),* $(,)?]) => {
        #[derive(Debug)]
        pub(crate) struct Lineup<$life> {
            $(pub $lint: Pausable<$kind>,)*
        }

        #[derive(Clone, Copy, Debug)]
        struct Chosen {
            $($lint: Option<Expanded>,)*
        }

        impl Lineup<'_> {
            fn awake(&self) -> Option<()> {
                [$(self.$lint.active()),*].into_iter().flatten().next()
            }

            fn chose(&mut self, invocation: &Macro) -> Chosen {
                Chosen { $($lint: self.$lint.chose(|lint| Hooks::macro_called(lint, invocation)),)* }
            }

            fn held(&mut self, depth: Depth, chosen: Chosen, phase: Phase) {
                $(self.$lint.held(depth, chosen.$lint.and_then(|choice| phase.admitted(choice)));)*
            }

            fn released(&mut self, depth: Depth) {
                $(self.$lint.left(depth, |_| {});)*
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
        no_free_fns: NoFreeFns,
        vec_out_params: VecOutParams,
        named_lifetimes: NamedLifetimes,
        type_shape: TypeShape,
        expression_flow: ExpressionFlow,
        manual_iteration: ManualIteration,
        process_starters: ProcessStarters,
        bool_methods: BoolMethods,
        no_dyn: NoDyn,
        public_api_tests: PublicApiTests<'file>,
        emitters: Emitters,
        no_test_only_api: NoTestOnlyApi<'file>,
        test_scan: TestScan,
        primitive_boundary: PrimitiveBoundary,
        test_shape: TestShape,
        dead_code: DeadCode<'file>,
        module_cycles: ModuleCycles<'file>,
    ]
});
