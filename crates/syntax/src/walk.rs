mod boundary;

use std::mem;

use syn::visit::{self, Visit};

pub use boundary::Entered;
use syn::{File, Ident, Macro};

#[macro_export]
macro_rules! node_kinds {
    ($callback:ident) => {
        $callback! {
            Attribute on_attribute off_attribute visit_attribute Attribute;
            Block on_block off_block visit_block Block;
            ExprAssign on_expr_assign off_expr_assign visit_expr_assign ExprAssign;
            ExprBinary on_expr_binary off_expr_binary visit_expr_binary ExprBinary;
            ExprMacro on_expr_macro off_expr_macro visit_expr_macro ExprMacro;
            ForeignItemFn on_foreign_item_fn off_foreign_item_fn visit_foreign_item_fn ForeignItemFn;
            ExprClosure on_expr_closure off_expr_closure visit_expr_closure ExprClosure;
            Expr on_expr off_expr visit_expr Expr;
            ExprForLoop on_expr_for_loop off_expr_for_loop visit_expr_for_loop ExprForLoop;
            ExprIf on_expr_if off_expr_if visit_expr_if ExprIf;
            ExprLoop on_expr_loop off_expr_loop visit_expr_loop ExprLoop;
            ExprMethodCall on_expr_method_call off_expr_method_call visit_expr_method_call ExprMethodCall;
            ExprPath on_expr_path off_expr_path visit_expr_path ExprPath;
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
            LifetimeParam on_lifetime_param off_lifetime_param visit_lifetime_param LifetimeParam;
            Local on_local off_local visit_local Local;
            PatIdent on_pat_ident off_pat_ident visit_pat_ident PatIdent;
            Path on_path off_path visit_path Path;
            Signature on_signature off_signature visit_signature Signature;
            Stmt on_stmt off_stmt visit_stmt Stmt;
            TraitItemConst on_trait_item_const off_trait_item_const visit_trait_item_const TraitItemConst;
            TraitItemFn on_trait_item_fn off_trait_item_fn visit_trait_item_fn TraitItemFn;
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

        impl<'ast> Visit<'ast> for Children<'ast> {
            fn visit_macro(&mut self, node: &'ast Macro) {
                self.0.push(Node::Macro(node));
            }

            fn visit_ident(&mut self, node: &'ast Ident) {
                self.0.push(Node::Ident(node));
            }

            $(
                fn $visit(&mut self, node: &'ast syn::$node) {
                    self.0.push(Node::$kind(node));
                }
            )*
        }

        impl<'ast, Acc: Default, Step: FnMut(Acc, Node<'ast>) -> Acc> Visit<'ast> for Folding<Acc, Step> {
            fn visit_macro(&mut self, node: &'ast Macro) {
                self.acc = (self.step)(mem::take(&mut self.acc), Node::Macro(node));
            }

            fn visit_ident(&mut self, node: &'ast Ident) {
                self.acc = (self.step)(mem::take(&mut self.acc), Node::Ident(node));
            }

            $(
                fn $visit(&mut self, node: &'ast syn::$node) {
                    self.acc = (self.step)(mem::take(&mut self.acc), Node::$kind(node));
                }
            )*
        }

        impl<'ast> Node<'ast> {
            #[must_use]
            pub fn children(self) -> Vec<Node<'ast>> {
                match self {
                    Node::Macro(node) => Children::collected(|children| visit::visit_macro(children, node)),
                    Node::Ident(_) => Vec::new(),
                    $(Node::$kind(node) => Children::collected(|children| visit::$visit(children, node)),)*
                }
            }

            pub fn folded<Acc: Default>(self, init: Acc, step: impl FnMut(Acc, Node<'ast>) -> Acc) -> Acc {
                let mut folding = Folding { acc: init, step };
                match self {
                    Node::Macro(node) => visit::visit_macro(&mut folding, node),
                    Node::Ident(_) => (),
                    $(Node::$kind(node) => visit::$visit(&mut folding, node),)*
                }
                folding.acc
            }
        }
    };
}

struct Folding<Acc, Step> {
    acc: Acc,
    step: Step,
}

struct Children<'ast>(Vec<Node<'ast>>);

node_kinds!(nodes);

impl<'ast> Children<'ast> {
    fn collected(visiting: impl FnOnce(&mut Children<'ast>)) -> Vec<Node<'ast>> {
        let mut children = Children(Vec::new());
        visiting(&mut children);
        children.0
    }
}

impl<'ast> Node<'ast> {
    #[must_use]
    pub fn top_level(file: &'ast File) -> Vec<Node<'ast>> {
        Children::collected(|children| visit::visit_file(children, file))
    }
}
