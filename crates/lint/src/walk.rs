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
    ProcessStarters, PublicApiTests, PublicSurface, TypeShape, VecOutParams,
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
enum Step {
    Entered,
    Skipped,
    Asleep,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hold {
    Paused,
    Kept,
}

#[derive(Debug)]
pub(crate) struct Pausable<Lint> {
    lint: Lint,
    paused: usize,
    steps: Vec<Step>,
    chosen: Vec<Option<Expanded>>,
    held: Vec<Hold>,
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
        let paused = admitted.map_or(1, |()| 0);
        Pausable { lint, paused, steps: Vec::new(), chosen: Vec::new(), held: Vec::new() }
    }

    pub(crate) fn woken(self) -> Option<Lint> {
        self.active().map(|()| self.lint)
    }

    fn active(&self) -> Option<()> {
        (self.paused == 0).then_some(())
    }

    fn entered(&mut self, hook: impl FnOnce(&mut Lint) -> Descent) {
        let step = self.active().map_or(Step::Asleep, |()| match hook(&mut self.lint) {
            Descent::Into => Step::Entered,
            Descent::Over => Step::Skipped,
        });
        self.paused += usize::from(step == Step::Skipped);
        self.steps.push(step);
    }

    fn left(&mut self, hook: impl FnOnce(&mut Lint)) {
        match self.steps.pop() {
            Some(Step::Entered) => hook(&mut self.lint),
            Some(Step::Skipped) => self.paused -= 1,
            Some(Step::Asleep) | None => {}
        }
    }

    fn chose(&mut self, hook: impl FnOnce(&mut Lint) -> Expanded) {
        let chosen = self.active().map(|()| hook(&mut self.lint));
        self.chosen.push(chosen);
    }

    fn wants(&self, phase: Phase) -> Option<()> {
        self.chosen.last().copied().flatten().and_then(|chosen| chosen.admits(phase))
    }

    fn hold(&mut self, phase: Phase) {
        let unwanted = self.active().and(self.wants(phase).xor(Some(())));
        let held = unwanted.map_or(Hold::Kept, |()| Hold::Paused);
        self.paused += usize::from(held == Hold::Paused);
        self.held.push(held);
    }

    fn release(&mut self) {
        self.paused -= usize::from(self.held.pop() == Some(Hold::Paused));
    }

    fn unchose(&mut self) {
        self.chosen.pop();
    }
}

macro_rules! each {
    ($walker:expr, [$($lint:ident: $kind:ty),* $(,)?], $method:ident $arguments:tt) => {
        $( $walker.$lint.$method $arguments; )*
    };
}

macro_rules! any {
    ($walker:expr, [$($lint:ident: $kind:ty),* $(,)?], $method:ident $arguments:tt) => {
        [$( $walker.$lint.$method $arguments ),*].into_iter().flatten().next()
    };
}

macro_rules! walker {
    (@walker $life:lifetime [$($lint:ident: $kind:ty),* $(,)?]) => {
        #[derive(Debug)]
        pub(crate) struct Walker<$life> {
            $(pub $lint: Pausable<$kind>,)*
        }
    };
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

        walker!(@walker $life $lints);

        impl Walker<'_> {
            fn awake(&self) -> Option<()> {
                any!(self, $lints, active())
            }

            fn wanting(&self, phase: Phase) -> Option<()> {
                any!(self, $lints, wants(phase))
            }

            fn phased(&mut self, phase: Phase, walk: impl FnOnce(&mut Self)) {
                each!(self, $lints, hold(phase));
                walk(self);
                each!(self, $lints, release());
            }
        }

        impl<'ast> Visit<'ast> for Walker<'_> {
            $(
                fn $visit(&mut self, node: &'ast $node) {
                    each!(self, $lints, entered(|lint| Hooks::$hook(lint, node)));
                    self.awake().map_or_default(|()| visit::$visit(self, node));
                    each!(self, $lints, left(|lint| Hooks::$left(lint, node)));
                }
            )*

            fn visit_macro(&mut self, node: &'ast Macro) {
                each!(self, $lints, chose(|lint| Hooks::macro_called(lint, node)));
                let defining = Expansion::defining(node);
                let statements = Phase::Statements(defining);
                self.wanting(statements).map_or_default(|()| {
                    let expansion = Expansion::of(node);
                    self.phased(statements, |walker| expansion.walk(walker));
                });
                self.wanting(Phase::Substituted).and(defining).map_or_default(|()| {
                    let expansion = Expansion::substituted(node);
                    self.phased(Phase::Substituted, |walker| expansion.walk(walker));
                });
                self.wanting(Phase::Path).map_or_default(|()| {
                    self.phased(Phase::Path, |walker| visit::visit_macro(walker, node));
                });
                each!(self, $lints, unchose());
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
