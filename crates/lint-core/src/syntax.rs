use proc_macro2::{Group, Literal, Punct, TokenTree};
use syn::punctuated::Punctuated;
use syn::{
    AngleBracketedGenericArguments, Expr, ExprLit, ExprPath, Field, Fields, FnArg, GenericArgument, GenericParam,
    Ident, ImplItem, ImplItemFn, Item, ItemImpl, ItemMod, ItemUse, Lit, LitStr, Member, Pat, PatIdent, PatType,
    PathArguments, ReturnType, Stmt, Token, TraitBound, TraitItem, TraitItemFn, Type, TypeParam, TypeParamBound,
    TypePath,
};

#[derive(Debug)]
pub struct Syntax;

impl Syntax {
    #[must_use]
    pub fn impl_fn(member: &ImplItem) -> Option<&ImplItemFn> {
        match member {
            ImplItem::Fn(function) => Some(function),
            _ => None,
        }
    }

    #[must_use]
    pub fn trait_fn(member: &TraitItem) -> Option<&TraitItemFn> {
        match member {
            TraitItem::Fn(function) => Some(function),
            _ => None,
        }
    }

    #[must_use]
    pub fn typed(argument: &FnArg) -> Option<&PatType> {
        match argument {
            FnArg::Typed(typed) => Some(typed),
            FnArg::Receiver(_) => None,
        }
    }

    #[must_use]
    pub fn returned(output: &ReturnType) -> Option<&Type> {
        match output {
            ReturnType::Type(_, returned) => Some(returned),
            ReturnType::Default => None,
        }
    }

    #[must_use]
    pub fn type_path(ty: &Type) -> Option<&TypePath> {
        match ty {
            Type::Path(path) => Some(path),
            _ => None,
        }
    }

    #[must_use]
    pub fn angled(arguments: &PathArguments) -> Option<&AngleBracketedGenericArguments> {
        match arguments {
            PathArguments::AngleBracketed(angled) => Some(angled),
            _ => None,
        }
    }

    #[must_use]
    pub fn type_argument(argument: &GenericArgument) -> Option<&Type> {
        match argument {
            GenericArgument::Type(ty) => Some(ty),
            _ => None,
        }
    }

    #[must_use]
    pub fn item_use(item: &Item) -> Option<&ItemUse> {
        match item {
            Item::Use(used) => Some(used),
            _ => None,
        }
    }

    #[must_use]
    pub fn item_mod(item: &Item) -> Option<&ItemMod> {
        match item {
            Item::Mod(module) => Some(module),
            _ => None,
        }
    }

    #[must_use]
    pub fn bounds(ty: &Type) -> Option<&Punctuated<TypeParamBound, Token![+]>> {
        match ty {
            Type::ImplTrait(bounded) => Some(&bounded.bounds),
            Type::TraitObject(bounded) => Some(&bounded.bounds),
            _ => None,
        }
    }

    #[must_use]
    pub fn trait_bound(bound: &TypeParamBound) -> Option<&TraitBound> {
        match bound {
            TypeParamBound::Trait(traited) => Some(traited),
            _ => None,
        }
    }

    #[must_use]
    pub fn named_member(member: &Member) -> Option<&Ident> {
        match member {
            Member::Named(named) => Some(named),
            Member::Unnamed(_) => None,
        }
    }

    #[must_use]
    pub fn expr_path(expr: &Expr) -> Option<&ExprPath> {
        match expr {
            Expr::Path(path) => Some(path),
            _ => None,
        }
    }

    #[must_use]
    pub fn literal(expr: &Expr) -> Option<&Lit> {
        match expr {
            Expr::Lit(literal) => Some(&literal.lit),
            _ => None,
        }
    }

    #[must_use]
    pub fn string_literal(value: &Expr) -> Option<&LitStr> {
        match value {
            Expr::Lit(ExprLit { lit: Lit::Str(text), .. }) => Some(text),
            _ => None,
        }
    }

    #[must_use]
    pub fn tail(statement: &Stmt) -> Option<&Expr> {
        match statement {
            Stmt::Expr(expr, None) => Some(expr),
            _ => None,
        }
    }

    #[must_use]
    pub fn pat_ident(pattern: &Pat) -> Option<&PatIdent> {
        match pattern {
            Pat::Ident(binding) => Some(binding),
            _ => None,
        }
    }

    #[must_use]
    pub fn type_param(param: &GenericParam) -> Option<&TypeParam> {
        match param {
            GenericParam::Type(parameter) => Some(parameter),
            _ => None,
        }
    }

    #[must_use]
    pub fn newtype_field(fields: &Fields) -> Option<&Field> {
        match fields {
            Fields::Unnamed(fields) => {
                Some(fields).filter(|fields| fields.unnamed.len() == 1).and_then(|fields| fields.unnamed.first())
            }
            Fields::Named(_) | Fields::Unit => None,
        }
    }

    #[must_use]
    pub fn inherent(item: &ItemImpl) -> Option<&ItemImpl> {
        match item.trait_ {
            None => Some(item),
            Some(_) => None,
        }
    }

    #[must_use]
    pub fn group(tree: &TokenTree) -> Option<&Group> {
        match tree {
            TokenTree::Group(group) => Some(group),
            _ => None,
        }
    }

    #[must_use]
    pub fn punct(tree: &TokenTree) -> Option<&Punct> {
        match tree {
            TokenTree::Punct(punct) => Some(punct),
            _ => None,
        }
    }

    #[must_use]
    pub fn token_ident(tree: &TokenTree) -> Option<&Ident> {
        match tree {
            TokenTree::Ident(ident) => Some(ident),
            _ => None,
        }
    }

    #[must_use]
    pub fn token_literal(tree: &TokenTree) -> Option<&Literal> {
        match tree {
            TokenTree::Literal(literal) => Some(literal),
            _ => None,
        }
    }
}
