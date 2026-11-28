mod boundary;
mod caller;
mod delegation;
mod filler;
#[cfg(feature = "laws")]
mod laws;
mod method;
mod names;
mod ops;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{ItemTrait, Token, TraitItem, TraitItemFn, TypeParamBound, Visibility, parse_quote};

pub(crate) use boundary::{Constant, Spelling};

use caller::Caller;
use delegation::Delegation;
use filler::Filler;
#[cfg(feature = "laws")]
use laws::Laws;
use method::Method;
use names::{CallerName, FillerName, OpsName};
use ops::Ops;

pub(crate) trait Rendered {
    fn rendered(declared: &Declared) -> TokenStream;

    fn rendered_after(declared: &Declared, earlier: &TokenStream) -> TokenStream {
        let own = Self::rendered(declared);
        quote! {
            #earlier
            #own
        }
    }
}

mod keyword {
    syn::custom_keyword!(laws);
}

enum Scripting {
    Unscripted,
    Scripted,
}

pub(crate) struct Declared {
    item: ItemTrait,
    caller: CallerName,
    filler: FillerName,
    ops: OpsName,
    methods: Vec<Method>,
    scripting: Scripting,
}

impl AsRef<[Method]> for Declared {
    fn as_ref(&self) -> &[Method] {
        &self.methods
    }
}

impl Declared {
    pub(crate) fn expanded(attributes: TokenStream, item: TokenStream) -> TokenStream {
        let expansion = syn::parse2::<Scripting>(attributes).and_then(|scripting| {
            syn::parse2::<ItemTrait>(item)
                .map(|declared| Declared::parsed(declared, scripting))
                .map(|declared| declared.expansion())
        });
        match expansion {
            Ok(expanded) => expanded,
            Err(error) => error.to_compile_error(),
        }
    }

    pub(crate) fn expansion(&self) -> TokenStream {
        let caller = Caller::rendered(self);
        let filler = Filler::rendered_after(self, &caller);
        let ops = Ops::rendered_after(self, &filler);
        let delegated = Delegation::rendered_after(self, &ops);
        let scripted = self.scripted();
        quote! {
            #delegated
            #scripted
        }
    }
}

impl Parse for Scripting {
    fn parse(input: ParseStream<'_>) -> syn::Result<Scripting> {
        if input.is_empty() {
            Ok(Scripting::Unscripted)
        } else {
            input.parse::<keyword::laws>().map(|_| Scripting::Scripted)
        }
    }
}

impl Declared {
    pub(crate) fn sealed_trait(&self) -> &ItemTrait {
        &self.item
    }

    pub(crate) fn visibility(&self) -> &Visibility {
        &self.item.vis
    }

    pub(crate) fn caller_name(&self) -> &CallerName {
        &self.caller
    }

    pub(crate) fn filler_name(&self) -> &FillerName {
        &self.filler
    }

    pub(crate) fn ops_name(&self) -> &OpsName {
        &self.ops
    }

    pub(crate) fn laws(&self) -> TokenStream {
        match self.scripting {
            Scripting::Unscripted => TokenStream::new(),
            #[cfg(feature = "laws")]
            Scripting::Scripted => Laws::named(self),
            #[cfg(not(feature = "laws"))]
            Scripting::Scripted => TokenStream::new(),
        }
    }

    fn scripted(&self) -> TokenStream {
        match self.scripting {
            Scripting::Unscripted => TokenStream::new(),
            #[cfg(feature = "laws")]
            Scripting::Scripted => Laws::rendered(self),
            #[cfg(not(feature = "laws"))]
            Scripting::Scripted => TokenStream::new(),
        }
    }

    fn parsed(declared: ItemTrait, scripting: Scripting) -> Declared {
        let mut item = Self::sealed(declared);
        let methods = item.items.iter_mut().filter_map(Self::required).map(Method::declared).collect();
        Declared {
            caller: CallerName::from(&item.ident),
            filler: FillerName::from(&item.ident),
            ops: OpsName::from(&item.ident),
            methods,
            item,
            scripting,
        }
    }

    fn sealed(mut item: ItemTrait) -> ItemTrait {
        let seal: TypeParamBound = parse_quote!(::port_core::Metered);
        item.colon_token.get_or_insert_with(<Token![:]>::default);
        item.supertraits.push(seal);
        item
    }

    fn required(item: &mut TraitItem) -> Option<&mut TraitItemFn> {
        match item {
            TraitItem::Fn(function) => Some(function),
            _ => None,
        }
    }
}
