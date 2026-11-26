use syntax::{
    AngleBracketedGenericArguments, Expr, ExprLit, ExprPath, FnArg, GenericArgument, Item, ItemImpl, ItemMod, ItemUse,
    Lit, LitStr, PatType, PathArguments, ReturnType, Type, TypePath,
};
use syntax::{Group, Ident, Literal, Punct, TokenTree};

#[derive(Debug)]
pub struct Syntax;

impl Syntax {
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
    pub fn expr_path(expr: &Expr) -> Option<&ExprPath> {
        match expr {
            Expr::Path(path) => Some(path),
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
