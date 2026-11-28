use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, Expr, ExprAssign, ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprMethodCall, ExprWhile, Field,
    ImplItemConst, ImplItemFn, Item, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStruct, ItemTrait,
    ItemType, ItemUse, LifetimeParam, Local, Macro, Path, Signature, TraitItemFn, Type, TypePath, TypeTraitObject,
    TypeTuple,
};

use crate::budget::TestScan;
use crate::expansion::Expansion;
use crate::lints::{
    BoolMethods, Emitters, ExpressionFlow, ManualIteration, NamedLifetimes, NoDyn, NoFreeFns, NoTestOnlyApi,
    ProcessStarters, PublicApiTests, PublicSurface, RealAdapters, TypeShape, VecOutParams,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Descent {
    Into,
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Expanded {
    Path,
    Statements,
    Surface,
    Nothing,
}

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
}

#[derive(Debug)]
pub(crate) struct Walker<'file> {
    lineup: Lineup<'file>,
    depth: usize,
}

impl From<Option<()>> for Descent {
    fn from(entered: Option<()>) -> Descent {
        entered.map_or(Descent::Over, |()| Descent::Into)
    }
}

impl Expanded {
    fn admits(self, phase: Phase) -> Option<()> {
        match (self, phase) {
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

    fn entered(&mut self, depth: usize, hook: impl FnOnce(&mut Lint) -> Descent) {
        let over = self.active().map(|()| hook(&mut self.lint)).filter(|descent| *descent == Descent::Over);
        self.state = over.map_or(self.state, |_| State::PausedAt(depth));
    }

    fn left(&mut self, depth: usize, hook: impl FnOnce(&mut Lint)) {
        match self.state {
            State::Awake => hook(&mut self.lint),
            State::PausedAt(at) => self.state = if at == depth { State::Awake } else { self.state },
            State::Off => {}
        }
    }

    fn chose(&mut self, hook: impl FnOnce(&mut Lint) -> Expanded) -> Option<Expanded> {
        self.active().map(|()| hook(&mut self.lint))
    }

    fn held(&mut self, depth: usize, wanted: Option<()>) {
        let unwanted = self.active().and(wanted.xor(Some(())));
        self.state = unwanted.map_or(self.state, |()| State::PausedAt(depth));
    }
}

impl Walker<'_> {
    pub(crate) fn walked<'file>(lineup: Lineup<'file>, file: &syn::File) -> Lineup<'file> {
        let mut walker = Walker { lineup, depth: 0 };
        walker.visit_file(file);
        walker.lineup
    }

    fn deeper(&mut self, walk: impl FnOnce(&mut Self, usize)) {
        self.depth += 1;
        walk(self, self.depth);
        self.depth -= 1;
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

            fn held(&mut self, depth: usize, chosen: Chosen, phase: Phase) {
                $(self.$lint.held(depth, chosen.$lint.and_then(|choice| choice.admits(phase)));)*
            }

            fn released(&mut self, depth: usize) {
                $(self.$lint.left(depth, |_| {});)*
            }
        }

        impl Chosen {
            fn wanting(self, phase: Phase) -> Option<()> {
                [$(self.$lint.and_then(|choice| choice.admits(phase))),*].into_iter().flatten().next()
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
        pub(crate) trait Hooks {
            $(
                fn $hook(&mut self, _: &$node) -> Descent {
                    Descent::Into
                }

                fn $left(&mut self, _: &$node) {}
            )*

            fn macro_called(&mut self, _: &Macro) -> Expanded {
                Expanded::Path
            }
        }

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

walker! {
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
        public_surface: PublicSurface<'file>,
        public_api_tests: PublicApiTests<'file>,
        emitters: Emitters,
        no_test_only_api: NoTestOnlyApi<'file>,
        real_adapters: RealAdapters<'file>,
        test_scan: TestScan,
    ]
    nodes [
        visit_file(syn::File) file left_file,
        visit_item(Item) item left_item,
        visit_item_mod(ItemMod) item_mod left_item_mod,
        visit_item_impl(ItemImpl) item_impl left_item_impl,
        visit_item_trait(ItemTrait) item_trait left_item_trait,
        visit_item_fn(ItemFn) item_fn left_item_fn,
        visit_impl_item_fn(ImplItemFn) impl_item_fn left_impl_item_fn,
        visit_trait_item_fn(TraitItemFn) trait_item_fn left_trait_item_fn,
        visit_item_const(ItemConst) item_const left_item_const,
        visit_impl_item_const(ImplItemConst) impl_item_const left_impl_item_const,
        visit_item_struct(ItemStruct) item_struct left_item_struct,
        visit_item_enum(ItemEnum) item_enum left_item_enum,
        visit_item_type(ItemType) item_type left_item_type,
        visit_item_use(ItemUse) item_use left_item_use,
        visit_item_macro(ItemMacro) item_macro left_item_macro,
        visit_block(Block) block left_block,
        visit_expr(Expr) expr left_expr,
        visit_expr_if(ExprIf) expr_if left_expr_if,
        visit_expr_loop(ExprLoop) expr_loop left_expr_loop,
        visit_expr_for_loop(ExprForLoop) expr_for_loop left_expr_for_loop,
        visit_expr_while(ExprWhile) expr_while left_expr_while,
        visit_expr_assign(ExprAssign) expr_assign left_expr_assign,
        visit_expr_binary(ExprBinary) expr_binary left_expr_binary,
        visit_local(Local) local left_local,
        visit_lifetime_param(LifetimeParam) lifetime_param left_lifetime_param,
        visit_path(Path) path left_path,
        visit_field(Field) field left_field,
        visit_attribute(Attribute) attribute left_attribute,
        visit_type_trait_object(TypeTraitObject) type_trait_object left_type_trait_object,
        visit_signature(Signature) signature left_signature,
        visit_type(Type) ty left_ty,
        visit_type_tuple(TypeTuple) type_tuple left_type_tuple,
        visit_type_path(TypePath) type_path left_type_path,
        visit_expr_method_call(ExprMethodCall) expr_method_call left_expr_method_call,
    ]
}
