use syn::{
    Attribute, Block, Expr, ExprAssign, ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprMethodCall, ExprWhile, Field,
    ImplItemConst, ImplItemFn, Item, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStruct, ItemTrait,
    ItemType, ItemUse, LifetimeParam, Local, Macro, Path, Signature, TraitItemFn, Type, TypePath, TypeTraitObject,
    TypeTuple, VisRestricted,
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

impl From<Option<()>> for Descent {
    fn from(entered: Option<()>) -> Descent {
        entered.map_or(Descent::Over, |()| Descent::Into)
    }
}

macro_rules! nodes {
    ($callback:ident { $($given:tt)* }) => {
        $callback! {
            $($given)*
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
                visit_vis_restricted(VisRestricted) vis_restricted left_vis_restricted,
            ]
        }
    };
}

macro_rules! hooks {
    (nodes [$($visit:ident($node:ty) $hook:ident $left:ident),* $(,)?]) => {
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
    };
}

pub(crate) use nodes;

nodes!(hooks {});
