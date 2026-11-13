use syn::Macro;

use super::Descend;

macro_rules! hooks {
    ($($kind:ident $on:ident $off:ident $visit:ident $node:ty;)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub enum Kind {
            Macro,
            $($kind,)*
        }

        impl Kind {
            pub const ALL: &'static [Kind] = &[Kind::Macro, $(Kind::$kind,)*];
        }

        #[derive(Debug)]
        pub struct ByKind<Value> {
            on_macro: Value,
            $($on: Value,)*
        }

        impl<Value> ByKind<Value> {
            pub fn filled(mut fill: impl FnMut(Kind) -> Value) -> ByKind<Value> {
                ByKind { on_macro: fill(Kind::Macro), $($on: fill(Kind::$kind),)* }
            }

            pub fn get(&self, kind: Kind) -> &Value {
                match kind {
                    Kind::Macro => &self.on_macro,
                    $(Kind::$kind => &self.$on,)*
                }
            }
        }

        pub trait Watch<'ast> {
            fn kinds(&self) -> &'static [Kind];

            fn on_macro(&mut self, _: &'ast Macro) -> Descend {
                Descend::Into
            }

            fn off_macro(&mut self, _: &'ast Macro) {}

            $(
                fn $on(&mut self, _: &'ast $node) -> Descend {
                    Descend::Into
                }

                fn $off(&mut self, _: &'ast $node) {}
            )*
        }

        #[macro_export]
        macro_rules! sweep_visits {
            () => {
                $(
                    fn $visit(&mut self, node: &'ast $node) {
                        self.around(
                            $crate::Kind::$kind,
                            |watcher| watcher.$on(node),
                            |sweep| syn::visit::$visit(sweep, node),
                            |watcher| watcher.$off(node),
                        );
                    }
                )*
            };
        }
    };
}

hooks! {
    Arm on_arm off_arm visit_arm syn::Arm;
    Attribute on_attribute off_attribute visit_attribute syn::Attribute;
    Block on_block off_block visit_block syn::Block;
    ExprAssign on_expr_assign off_expr_assign visit_expr_assign syn::ExprAssign;
    ExprBinary on_expr_binary off_expr_binary visit_expr_binary syn::ExprBinary;
    ExprBreak on_expr_break off_expr_break visit_expr_break syn::ExprBreak;
    ExprCast on_expr_cast off_expr_cast visit_expr_cast syn::ExprCast;
    ExprClosure on_expr_closure off_expr_closure visit_expr_closure syn::ExprClosure;
    Expr on_expr off_expr visit_expr syn::Expr;
    ExprConst on_expr_const off_expr_const visit_expr_const syn::ExprConst;
    ExprContinue on_expr_continue off_expr_continue visit_expr_continue syn::ExprContinue;
    ExprForLoop on_expr_for_loop off_expr_for_loop visit_expr_for_loop syn::ExprForLoop;
    ExprIf on_expr_if off_expr_if visit_expr_if syn::ExprIf;
    ExprLoop on_expr_loop off_expr_loop visit_expr_loop syn::ExprLoop;
    ExprMatch on_expr_match off_expr_match visit_expr_match syn::ExprMatch;
    ExprMethodCall on_expr_method_call off_expr_method_call visit_expr_method_call syn::ExprMethodCall;
    ExprPath on_expr_path off_expr_path visit_expr_path syn::ExprPath;
    ExprRepeat on_expr_repeat off_expr_repeat visit_expr_repeat syn::ExprRepeat;
    ExprReturn on_expr_return off_expr_return visit_expr_return syn::ExprReturn;
    ExprTry on_expr_try off_expr_try visit_expr_try syn::ExprTry;
    ExprUnary on_expr_unary off_expr_unary visit_expr_unary syn::ExprUnary;
    ExprWhile on_expr_while off_expr_while visit_expr_while syn::ExprWhile;
    Field on_field off_field visit_field syn::Field;
    Ident on_ident off_ident visit_ident syn::Ident;
    ImplItemConst on_impl_item_const off_impl_item_const visit_impl_item_const syn::ImplItemConst;
    ImplItemFn on_impl_item_fn off_impl_item_fn visit_impl_item_fn syn::ImplItemFn;
    Item on_item off_item visit_item syn::Item;
    ItemConst on_item_const off_item_const visit_item_const syn::ItemConst;
    ItemEnum on_item_enum off_item_enum visit_item_enum syn::ItemEnum;
    ItemFn on_item_fn off_item_fn visit_item_fn syn::ItemFn;
    ItemImpl on_item_impl off_item_impl visit_item_impl syn::ItemImpl;
    ItemMacro on_item_macro off_item_macro visit_item_macro syn::ItemMacro;
    ItemMod on_item_mod off_item_mod visit_item_mod syn::ItemMod;
    ItemStatic on_item_static off_item_static visit_item_static syn::ItemStatic;
    ItemStruct on_item_struct off_item_struct visit_item_struct syn::ItemStruct;
    ItemTrait on_item_trait off_item_trait visit_item_trait syn::ItemTrait;
    ItemType on_item_type off_item_type visit_item_type syn::ItemType;
    ItemUnion on_item_union off_item_union visit_item_union syn::ItemUnion;
    ItemUse on_item_use off_item_use visit_item_use syn::ItemUse;
    Lifetime on_lifetime off_lifetime visit_lifetime syn::Lifetime;
    LifetimeParam on_lifetime_param off_lifetime_param visit_lifetime_param syn::LifetimeParam;
    LitFloat on_lit_float off_lit_float visit_lit_float syn::LitFloat;
    LitInt on_lit_int off_lit_int visit_lit_int syn::LitInt;
    Local on_local off_local visit_local syn::Local;
    PatStruct on_pat_struct off_pat_struct visit_pat_struct syn::PatStruct;
    PatTupleStruct on_pat_tuple_struct off_pat_tuple_struct visit_pat_tuple_struct syn::PatTupleStruct;
    Path on_path off_path visit_path syn::Path;
    Signature on_signature off_signature visit_signature syn::Signature;
    Stmt on_stmt off_stmt visit_stmt syn::Stmt;
    TraitBound on_trait_bound off_trait_bound visit_trait_bound syn::TraitBound;
    TraitItemConst on_trait_item_const off_trait_item_const visit_trait_item_const syn::TraitItemConst;
    TraitItemFn on_trait_item_fn off_trait_item_fn visit_trait_item_fn syn::TraitItemFn;
    TypeArray on_type_array off_type_array visit_type_array syn::TypeArray;
    TypePath on_type_path off_type_path visit_type_path syn::TypePath;
    TypeReference on_type_reference off_type_reference visit_type_reference syn::TypeReference;
    TypeTuple on_type_tuple off_type_tuple visit_type_tuple syn::TypeTuple;
    Variant on_variant off_variant visit_variant syn::Variant;
    Visibility on_visibility off_visibility visit_visibility syn::Visibility;
}
