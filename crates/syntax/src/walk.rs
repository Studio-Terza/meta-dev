use syn::visit::{self, Visit};
use syn::{File, Ident, Item, Macro, Stmt};

#[macro_export]
macro_rules! node_kinds {
    ($callback:ident) => {
        $callback! {
            Arm on_arm off_arm visit_arm Arm;
            Attribute on_attribute off_attribute visit_attribute Attribute;
            Block on_block off_block visit_block Block;
            ExprAssign on_expr_assign off_expr_assign visit_expr_assign ExprAssign;
            ExprBinary on_expr_binary off_expr_binary visit_expr_binary ExprBinary;
            ExprBreak on_expr_break off_expr_break visit_expr_break ExprBreak;
            ExprCall on_expr_call off_expr_call visit_expr_call ExprCall;
            ExprMacro on_expr_macro off_expr_macro visit_expr_macro ExprMacro;
            ForeignItemFn on_foreign_item_fn off_foreign_item_fn visit_foreign_item_fn ForeignItemFn;
            ExprCast on_expr_cast off_expr_cast visit_expr_cast ExprCast;
            ExprClosure on_expr_closure off_expr_closure visit_expr_closure ExprClosure;
            Expr on_expr off_expr visit_expr Expr;
            ExprConst on_expr_const off_expr_const visit_expr_const ExprConst;
            ExprContinue on_expr_continue off_expr_continue visit_expr_continue ExprContinue;
            ExprField on_expr_field off_expr_field visit_expr_field ExprField;
            ExprForLoop on_expr_for_loop off_expr_for_loop visit_expr_for_loop ExprForLoop;
            ExprIf on_expr_if off_expr_if visit_expr_if ExprIf;
            ExprLet on_expr_let off_expr_let visit_expr_let ExprLet;
            ExprLoop on_expr_loop off_expr_loop visit_expr_loop ExprLoop;
            ExprMatch on_expr_match off_expr_match visit_expr_match ExprMatch;
            ExprMethodCall on_expr_method_call off_expr_method_call visit_expr_method_call ExprMethodCall;
            ExprPath on_expr_path off_expr_path visit_expr_path ExprPath;
            ExprRepeat on_expr_repeat off_expr_repeat visit_expr_repeat ExprRepeat;
            ExprReturn on_expr_return off_expr_return visit_expr_return ExprReturn;
            ExprStruct on_expr_struct off_expr_struct visit_expr_struct ExprStruct;
            ExprTry on_expr_try off_expr_try visit_expr_try ExprTry;
            ExprUnary on_expr_unary off_expr_unary visit_expr_unary ExprUnary;
            ExprWhile on_expr_while off_expr_while visit_expr_while ExprWhile;
            Field on_field off_field visit_field Field;
            ImplItemConst on_impl_item_const off_impl_item_const visit_impl_item_const ImplItemConst;
            ImplItemFn on_impl_item_fn off_impl_item_fn visit_impl_item_fn ImplItemFn;
            Item on_item off_item visit_item Item;
            ItemConst on_item_const off_item_const visit_item_const ItemConst;
            ItemEnum on_item_enum off_item_enum visit_item_enum ItemEnum;
            ItemFn on_item_fn off_item_fn visit_item_fn ItemFn;
            ItemImpl on_item_impl off_item_impl visit_item_impl ItemImpl;
            ItemMacro on_item_macro off_item_macro visit_item_macro ItemMacro;
            ItemMod on_item_mod off_item_mod visit_item_mod ItemMod;
            ItemStatic on_item_static off_item_static visit_item_static ItemStatic;
            ItemStruct on_item_struct off_item_struct visit_item_struct ItemStruct;
            ItemTrait on_item_trait off_item_trait visit_item_trait ItemTrait;
            ItemType on_item_type off_item_type visit_item_type ItemType;
            ItemUnion on_item_union off_item_union visit_item_union ItemUnion;
            ItemUse on_item_use off_item_use visit_item_use ItemUse;
            Lifetime on_lifetime off_lifetime visit_lifetime Lifetime;
            LifetimeParam on_lifetime_param off_lifetime_param visit_lifetime_param LifetimeParam;
            LitFloat on_lit_float off_lit_float visit_lit_float LitFloat;
            LitInt on_lit_int off_lit_int visit_lit_int LitInt;
            Local on_local off_local visit_local Local;
            PatIdent on_pat_ident off_pat_ident visit_pat_ident PatIdent;
            Path on_path off_path visit_path Path;
            Signature on_signature off_signature visit_signature Signature;
            Stmt on_stmt off_stmt visit_stmt Stmt;
            TraitBound on_trait_bound off_trait_bound visit_trait_bound TraitBound;
            TraitItemConst on_trait_item_const off_trait_item_const visit_trait_item_const TraitItemConst;
            TraitItemFn on_trait_item_fn off_trait_item_fn visit_trait_item_fn TraitItemFn;
            TypeArray on_type_array off_type_array visit_type_array TypeArray;
            Type on_type off_type visit_type Type;
            TypePath on_type_path off_type_path visit_type_path TypePath;
            TypeReference on_type_reference off_type_reference visit_type_reference TypeReference;
            TypeTuple on_type_tuple off_type_tuple visit_type_tuple TypeTuple;
            Variant on_variant off_variant visit_variant Variant;
            Visibility on_visibility off_visibility visit_visibility Visibility;
        }
    };
}

macro_rules! nodes {
    ($($kind:ident $on:ident $off:ident $visit:ident $node:ident;)*) => {
        #[derive(Clone, Copy, Debug)]
        pub enum Node<'ast> {
            Macro(&'ast Macro),
            Ident(&'ast Ident),
            $($kind(&'ast syn::$node),)*
        }

        impl<'ast, Visitor: Around + ?Sized> Visit<'ast> for Walk<'_, Visitor> {
            fn visit_macro(&mut self, node: &'ast Macro) {
                self.0.around(Node::Macro(node), &mut |around| visit::visit_macro(&mut Walk(around), node));
            }

            fn visit_ident(&mut self, node: &'ast Ident) {
                self.0.around(Node::Ident(node), &mut |_| {});
            }

            $(
                fn $visit(&mut self, node: &'ast syn::$node) {
                    self.0.around(Node::$kind(node), &mut |around| visit::$visit(&mut Walk(around), node));
                }
            )*
        }
    };
}

node_kinds!(nodes);

pub trait Around {
    fn around(&mut self, node: Node<'_>, descend: &mut dyn FnMut(&mut Self));
}

#[derive(Debug)]
pub struct Walk<'walk, Visitor: ?Sized>(&'walk mut Visitor);

impl<'walk, Visitor: Around + ?Sized> Walk<'walk, Visitor> {
    pub fn file(visitor: &'walk mut Visitor, file: &File) {
        Walk(visitor).visit_file(file);
    }

    pub fn stmt(visitor: &'walk mut Visitor, stmt: &Stmt) {
        Walk(visitor).visit_stmt(stmt);
    }

    pub fn item_node(visitor: &'walk mut Visitor, item: &Item) {
        Walk(visitor).visit_item(item);
    }
}
