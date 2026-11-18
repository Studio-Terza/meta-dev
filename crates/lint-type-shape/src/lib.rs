use algebra::{Monoid, Semigroup};
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{
    Address, Concludes, Contribute, Contributed, Contributing, Contribution, Descend, Facts, Finds, Kind, Scope,
};
use std::path::Path;
use syntax::Span;
use syntax::Spanned;

use syntax::{
    Field, Fields, GenericArgument, Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, Macro, Signature, Type,
    TypePath, TypeTuple,
};

use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Syntax;
use lint_core::Violation;
use lint_layout::LineNumber;

#[derive(Debug)]
pub struct TypeShape;

impl TypeShape {
    const TUPLE: Message = Message::new(
        Problem::new("tuple in a signature or field"),
        Fix::new("values that travel together are a struct"),
    );

    fn string_error(path: &TypePath) -> Option<Message> {
        path.path
            .segments
            .last()
            .filter(|segment| segment.ident == "Result")
            .and_then(|segment| Syntax::angled(&segment.arguments))
            .and_then(|arguments| arguments.args.iter().nth(1))
            .filter(|argument| {
                matches!(
                    argument,
                    GenericArgument::Type(Type::Path(error))
                        if error.path.segments.last().is_some_and(|segment| segment.ident == "String")
                )
            })
            .map(|_| {
                Message::new(Problem::new("Result with a String error"), Fix::new("errors are an enum with thiserror"))
            })
    }
}

impl SweptLint for TypeShape {
    const HEADLINE: Headline =
        Headline::new("types carry their meaning: no bool fields, no tuples in signatures, no String errors");
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(Contributing::new(TypeShapes { path: AsRef::<Path>::as_ref(file.head()) }))
    }
}

#[derive(Debug)]
struct TypeShapes<'scan> {
    path: &'scan Path,
}

#[derive(Debug)]
struct Located {
    at: Address,
    span: Span,
    messages: Vec<Message>,
}

#[derive(Debug, Default)]
struct Shapes<'scan> {
    found: Vec<Violation<'scan>>,
    loose: Vec<Message>,
    located: Vec<Located>,
}

impl Semigroup for Shapes<'_> {
    fn combine(self, other: Self) -> Self {
        Shapes {
            found: self.found.combine(other.found),
            loose: self.loose.combine(other.loose),
            located: self.located.combine(other.located),
        }
    }
}

impl Monoid for Shapes<'_> {}

impl<'scan> Shapes<'scan> {
    fn within(self, ty: &Type) -> Shapes<'scan> {
        let messages: Vec<Message> =
            self.loose.into_iter().chain(self.located.into_iter().flat_map(|located| located.messages)).collect();
        let located = messages.first().copied().map(|_| Located { at: Address::from(ty), span: ty.span(), messages });
        Shapes { found: self.found, loose: Vec::new(), located: located.into_iter().collect() }
    }

    fn rooted<'root>(self, roots: impl IntoIterator<Item = &'root Type>, shapes: &TypeShapes<'scan>) -> Shapes<'scan> {
        let addresses: Vec<Address> = roots.into_iter().map(Address::from).collect();
        let reached = self.located.into_iter().filter(|located| addresses.contains(&located.at));
        let found = reached.flat_map(|located| {
            located.messages.into_iter().map(move |message| shapes.violation(located.span, message))
        });
        Shapes { found: self.found.into_iter().chain(found).collect(), loose: Vec::new(), located: Vec::new() }
    }
}

impl<'scan> TypeShapes<'scan> {
    fn violation(&self, span: Span, message: Message) -> Violation<'scan> {
        Violation::new(Site::new(self.path, LineNumber::from(span)), message)
    }

    fn bool_field(&self, field: &Field) -> Option<Violation<'scan>> {
        matches!(
            &field.ty,
            Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "bool")
        )
        .then(|| {
            self.violation(
                field.ident.as_ref().map_or_else(|| field.ty.span(), Ident::span),
                Message::new(
                    Problem::new("bool field"),
                    Fix::new("a stored flag is a stored mode, make it an enum or a type parameter"),
                ),
            )
        })
    }

    fn fields(&self, fields: &Fields) -> Contributed<(), Shapes<'scan>> {
        Contributed::parted(Shapes {
            found: fields.iter().filter_map(|field| self.bool_field(field)).collect(),
            ..Shapes::default()
        })
    }

    fn loose(message: Option<Message>) -> Contributed<(), Shapes<'scan>> {
        Contributed::parted(Shapes { loose: message.into_iter().collect(), ..Shapes::default() })
    }
}

impl<'scan> Contribute for TypeShapes<'scan> {
    type Open = ();
    type Part = Shapes<'scan>;

    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::ItemMod,
            Kind::ItemImpl,
            Kind::ItemStruct,
            Kind::ItemEnum,
            Kind::ItemType,
            Kind::Macro,
            Kind::Signature,
            Kind::Type,
            Kind::TypeTuple,
            Kind::TypePath,
        ]
    }

    fn scopes(&self) -> &'static [Scope] {
        const {
            &[
                Scope::new(Kind::ItemStruct),
                Scope::new(Kind::ItemEnum),
                Scope::new(Kind::ItemType),
                Scope::new(Kind::Signature),
                Scope::new(Kind::Type),
            ]
        }
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Shapes<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_impl(
        &self,
        _: &ItemImpl,
        facts: &Facts<'_, ItemImpl>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Shapes<'scan>> {
        Contributed::descending(facts.inherent().map_or(Descend::Over, |_| Descend::Into))
    }

    fn on_item_struct(
        &self,
        item: &ItemStruct,
        _: &Facts<'_, ItemStruct>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Shapes<'scan>> {
        self.fields(&item.fields)
    }

    fn off_item_struct(
        &self,
        item: &ItemStruct,
        _: &Facts<'_, ItemStruct>,
        inner: Contribution<(), Shapes<'scan>>,
    ) -> Shapes<'scan> {
        inner.concluded(|(), shapes| shapes.rooted(item.fields.iter().map(|field| &field.ty), self))
    }

    fn on_item_enum(
        &self,
        item: &ItemEnum,
        _: &Facts<'_, ItemEnum>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Shapes<'scan>> {
        let found =
            item.variants.iter().flat_map(|variant| variant.fields.iter().filter_map(|field| self.bool_field(field)));
        Contributed::parted(Shapes { found: found.collect(), ..Shapes::default() })
    }

    fn off_item_enum(
        &self,
        item: &ItemEnum,
        _: &Facts<'_, ItemEnum>,
        inner: Contribution<(), Shapes<'scan>>,
    ) -> Shapes<'scan> {
        let roots = item.variants.iter().flat_map(|variant| variant.fields.iter().map(|field| &field.ty));
        inner.concluded(|(), shapes| shapes.rooted(roots, self))
    }

    fn off_item_type(
        &self,
        item: &ItemType,
        _: &Facts<'_, ItemType>,
        inner: Contribution<(), Shapes<'scan>>,
    ) -> Shapes<'scan> {
        inner.concluded(|(), shapes| shapes.rooted([&*item.ty], self))
    }

    fn on_macro(
        &self,
        _: &Macro,
        _: &Facts<'_, Macro>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Shapes<'scan>> {
        Contributed::descending(Descend::Expand)
    }

    fn off_signature(
        &self,
        signature: &Signature,
        _: &Facts<'_, Signature>,
        inner: Contribution<(), Shapes<'scan>>,
    ) -> Shapes<'scan> {
        let roots = signature
            .inputs
            .iter()
            .filter_map(Syntax::typed)
            .map(|typed| &*typed.ty)
            .chain(Syntax::returned(&signature.output));
        inner.concluded(|(), shapes| shapes.rooted(roots, self))
    }

    fn off_type(&self, ty: &Type, _: &Facts<'_, Type>, inner: Contribution<(), Shapes<'scan>>) -> Shapes<'scan> {
        inner.concluded(|(), shapes| shapes.within(ty))
    }

    fn on_type_tuple(
        &self,
        tuple: &TypeTuple,
        _: &Facts<'_, TypeTuple>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Shapes<'scan>> {
        Self::loose(tuple.elems.first().map(|_| TypeShape::TUPLE))
    }

    fn on_type_path(
        &self,
        path: &TypePath,
        _: &Facts<'_, TypePath>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Shapes<'scan>> {
        Self::loose(TypeShape::string_error(path))
    }
}

impl<'scan> Concludes<'scan> for TypeShapes<'scan> {
    fn violations(self, (): (), shapes: Shapes<'scan>) -> Vec<Violation<'scan>> {
        let mut found = shapes.found;
        found.sort_by_key(|violation| *violation.site());
        found
    }
}
