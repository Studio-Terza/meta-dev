use std::path::Path;
use std::sync::Arc;

use syntax::{
    Attribute, Block, ExprCall, ExprMacro, ExprMethodCall, ExprPath, ForeignItemFn, ImplItemConst, ImplItemFn,
    ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemType, ItemUnion,
    Macro, Signature as SynSignature, TraitItemFn, Visibility as SynVisibility,
};

use algebra::{Monoid, Semigroup};
use graph::{Evaluation, Reach};
use lint_core::SourceFile;
use lint_layout::{Name, Position};
use lint_sweep::{Address, Around, Contribute, Contributed, Contribution, Descend, Facts, Kind, Scope, Sweep};

use super::attributes::Attributes;
use super::definition::{BodyGroup, BodyPart, Branches, Mutations};
use super::frames::Enclosure;
use super::held::{Holder, Link};
use super::regions::{Context, Regions};
use super::seat::Seat;
use super::signature::{InScope, LoweredParameter};
use super::{AliasRow, FileRows, FileShape, FnRow, Frame, Framed, Grains, Lowered, Slot};
use crate::namespace::{Domain, Entries, Exposing, Openings, ScopeKey, ScopeKind, ScopeOp, UsePath, Visibility};
use graph::Held;

#[derive(Debug)]
pub(crate) struct Lowering<'read> {
    top: Frame<'read>,
}

type Grown<'read> = Contributed<Framed<'read>, Grains>;

impl<'read> Frame<'read> {
    pub(super) fn in_table(&self) -> Frame<'read> {
        Frame { tabled: Some(()), ..self.clone() }
    }

    pub(super) fn framed(self, grains: Grains, descend: Descend) -> Grown<'read> {
        Contributed::new(Framed(Some(self)), grains, descend)
    }

    pub(super) fn with_context(&self, context: Context) -> Frame<'read> {
        Frame { context, ..self.clone() }
    }

    fn entered(&self, node: Address, kind: ScopeKind, entries: Entries, children: Arc<Path>) -> Opened<'read> {
        let key = ScopeKey::reached(&self.within, node);
        let op = ScopeOp::Opened { key: key.clone(), parent: self.here.scope().clone(), kind, entries };
        let frame = Frame { here: Seat::new(key, children), context: self.context.entered(kind), ..self.clone() };
        Opened { frame, op }
    }

    pub(super) fn function(
        &self,
        attributes: &[Attribute],
        visibility: &SynVisibility,
        signature: &SynSignature,
    ) -> Functioned {
        let generics = self.generics.extended(&signature.generics);
        let name = Name::from(&signature.ident);
        let row = FnRow {
            located: self.here.located(Position::from(signature.ident.span())),
            exposure: Exposing::declared_by(visibility),
            evaluation: signature.constness.map_or(Evaluation::Runtime, |_| Evaluation::Constant),
            owner: self.enclosure.owner(),
            entry: self.enclosure.entry(attributes, &name),
            inputs: generics.inputs(signature),
            output: generics.output(&signature.output),
            constraints: generics.constraints(&signature.generics),
            parameters: LoweredParameter::declared(&generics, &signature.generics),
            mutation: Mutations::signed(signature),
            definition: None,
            region: self.context.region(),
            place: self.context.place(),
            name,
        };
        Functioned { row, generics }
    }

    pub(super) fn method_entered(
        &self,
        attributes: &[Attribute],
        visibility: &SynVisibility,
        signature: &SynSignature,
    ) -> Grown<'read> {
        let inner = self.with_context(self.context.within_item(attributes));
        let Functioned { row, generics } = inner.function(attributes, visibility, signature);
        let frame = Frame { enclosure: Enclosure::Body, generics: Arc::new(generics), ..inner };
        frame.framed(Grains::from(FileRows { functions: vec![row], ..FileRows::default() }), Descend::Into)
    }

    pub(super) fn module_entered(&self, item: &ItemMod, items: &[syntax::Item]) -> Grown<'read> {
        let name = Name::from(&item.ident);
        let children = Arc::from(self.here.children().join(&name));
        let Opened { frame: inner, op } =
            self.entered(Address::from(item), ScopeKind::Module, Entries::of_items(items), children);
        let declared = ScopeOp::Declared {
            key: self.here.scope().clone(),
            name,
            visibility: Visibility::from(&item.vis),
            module: inner.here.scope().clone(),
        };
        let context = inner.context.within_module(item);
        let module = Frame { enclosure: Enclosure::Module, context, ..inner };
        module.framed(Grains { ops: vec![op, declared], ..Grains::default() }, Descend::Into)
    }

    pub(super) fn slotted(&self, item: &ItemMod) -> Grown<'read> {
        let slot = Slot::new(&self.here, self.context.region(), item);
        Contributed::new(Framed::default(), Grains { slots: vec![slot], ..Grains::default() }, Descend::Over)
    }

    pub(super) fn defined(
        &self,
        signature: &SynSignature,
        body: Option<&Block>,
        inner: Contribution<Framed<'read>, Grains>,
    ) -> Grains {
        inner.concluded(|framed, grown| {
            let generics = framed.0.map_or_else(|| Arc::clone(&self.generics), |frame| frame.generics);
            let Grains { rows, ops, slots, body: part } = grown;
            let definition = body.map(|block| {
                let mut group = BodyGroup::opened(&generics, signature, block);
                Sweep::block_in(self.source, block, lint_sweep::Gathers::watching(&mut group).into_iter().collect());
                lint_sweep::Gathers::gathered(group).lowered(part)
            });
            Grains { rows: rows.defined(definition), ops, slots, body: BodyPart::default() }
        })
    }
}

#[derive(Debug)]
struct Opened<'read> {
    frame: Frame<'read>,
    op: ScopeOp,
}

#[derive(Debug)]
pub(super) struct Functioned {
    row: FnRow,
    generics: InScope,
}

impl<'read> Lowering<'read> {
    pub(crate) fn of_file(source: &'read SourceFile, reach: Reach) -> Lowering<'read> {
        let shape = FileShape::from(source);
        let top = Frame {
            source,
            here: Seat::new(ScopeKey::default(), shape.children),
            enclosure: shape.enclosure,
            generics: Arc::new(InScope::outside()),
            context: Context::outermost(Regions::of_file(source, reach)),
            calling: None,
            tabled: None,
            within: Arc::from([]),
        };
        Lowering { top }
    }

    pub(crate) fn lowered(self, grains: Grains) -> Lowered {
        let replayed = Openings::replayed(Entries::of_items(&self.top.source.syntax().items), grains.ops);
        replayed.concluded(|openings, scopes| Lowered { openings, scopes, slots: grains.slots, rows: grains.rows })
    }

    fn frame<'around>(&'around self, around: &Around<'around, Framed<'read>, Grains>) -> &'around Frame<'read> {
        around.nearest().find_map(|framed| framed.0.as_ref()).unwrap_or(&self.top)
    }

    pub(super) fn below(frame: Frame<'read>) -> Lowering<'read> {
        Lowering { top: frame }
    }
}

impl<'read> Contribute for Lowering<'read> {
    type Open = Framed<'read>;
    type Part = Grains;

    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::Type,
            Kind::ForeignItemFn,
            Kind::ExprMethodCall,
            Kind::ExprMacro,
            Kind::ExprStruct,
            Kind::ExprPath,
            Kind::ExprMatch,
            Kind::ExprLet,
            Kind::Local,
            Kind::Macro,
            Kind::ExprIf,
            Kind::ExprWhile,
            Kind::ExprForLoop,
            Kind::ExprTry,
            Kind::ExprBinary,
            Kind::PatIdent,
        ]
    }

    fn scopes(&self) -> &'static [Scope] {
        const {
            &[
                Scope::new(Kind::ItemMod),
                Scope::new(Kind::Block),
                Scope::new(Kind::ItemImpl),
                Scope::new(Kind::ItemTrait),
                Scope::new(Kind::ItemFn),
                Scope::new(Kind::ImplItemFn),
                Scope::new(Kind::TraitItemFn),
                Scope::new(Kind::ItemType),
                Scope::new(Kind::ItemStruct),
                Scope::new(Kind::ItemEnum),
                Scope::new(Kind::ItemUnion),
                Scope::new(Kind::ItemConst),
                Scope::new(Kind::ItemStatic),
                Scope::new(Kind::ImplItemConst),
                Scope::new(Kind::ItemMacro),
                Scope::new(Kind::ExprCall),
                Scope::new(Kind::ExprMethodCall),
                Scope::new(Kind::Item),
                Scope::new(Kind::Signature),
            ]
        }
    }

    fn on_type(
        &self,
        ty: &syntax::Type,
        _: &Facts<'_, syntax::Type>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        Contributed::parted(self.frame(&around).types_used(ty))
    }

    fn on_item_mod(
        &self,
        item: &ItemMod,
        _: &Facts<'_, ItemMod>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let frame = self.frame(&around);
        item.content.as_ref().map_or_else(|| frame.slotted(item), |(_, items)| frame.module_entered(item, items))
    }

    fn on_block(&self, block: &Block, _: &Facts<'_, Block>, around: Around<'_, Framed<'read>, Grains>) -> Grown<'read> {
        let frame = self.frame(&around);
        let Opened { frame: inner, op } = frame.entered(
            Address::from(block),
            ScopeKind::Block,
            Entries::of_block(&block.stmts),
            frame.here.children(),
        );
        inner.framed(Grains { ops: vec![op], ..Grains::default() }, Descend::Into)
    }

    fn on_item_impl(
        &self,
        block: &ItemImpl,
        _: &Facts<'_, ItemImpl>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let frame = self.frame(&around);
        frame.with_context(frame.context.within_item(&block.attrs)).impl_entered(block)
    }

    fn on_item_trait(
        &self,
        item: &ItemTrait,
        _: &Facts<'_, ItemTrait>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let frame = self.frame(&around);
        let generics = frame.generics.subjectless().extended(&item.generics);
        let bounds = frame.trait_bounds(item, &generics);
        let owning = frame.trait_owning(&item.ident, &item.attrs);
        let filler = frame.filler_owning(item);
        let context = owning.within(&frame.context);
        let inner = Frame { context, enclosure: Enclosure::Trait, generics: Arc::new(generics), ..frame.clone() };
        inner.framed(bounds.combine(owning.grains).combine(filler), Descend::Into)
    }

    fn on_item_fn(
        &self,
        item: &ItemFn,
        _: &Facts<'_, ItemFn>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let frame = self.frame(&around);
        let owning = frame.owning(&item.sig.ident, Domain::Values, &item.attrs);
        let inner = frame.with_context(owning.within(&frame.context.within_item(&item.attrs)));
        let Functioned { row, generics } = inner.function(&item.attrs, &item.vis, &item.sig);
        let body = Frame { enclosure: Enclosure::Body, generics: Arc::new(generics), ..inner };
        body.framed(
            owning.grains.combine(Grains::from(FileRows { functions: vec![row], ..FileRows::default() })),
            Descend::Into,
        )
    }

    fn off_item_fn(&self, item: &ItemFn, _: &Facts<'_, ItemFn>, inner: Contribution<Framed<'read>, Grains>) -> Grains {
        self.top.defined(&item.sig, Some(&item.block), inner)
    }

    fn on_impl_item_fn(
        &self,
        item: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        self.frame(&around).method_entered(&item.attrs, &item.vis, &item.sig)
    }

    fn off_impl_item_fn(
        &self,
        item: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
        inner: Contribution<Framed<'read>, Grains>,
    ) -> Grains {
        self.top.defined(&item.sig, Some(&item.block), inner)
    }

    fn on_trait_item_fn(
        &self,
        item: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        self.frame(&around).method_entered(&item.attrs, &SynVisibility::Inherited, &item.sig)
    }

    fn off_trait_item_fn(
        &self,
        item: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
        inner: Contribution<Framed<'read>, Grains>,
    ) -> Grains {
        self.top.defined(&item.sig, item.default.as_ref(), inner)
    }

    fn on_foreign_item_fn(
        &self,
        item: &ForeignItemFn,
        _: &Facts<'_, ForeignItemFn>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let foreign = Frame { enclosure: Enclosure::Foreign, ..self.frame(&around).clone() };
        let Functioned { row, .. } = foreign.function(&item.attrs, &item.vis, &item.sig);
        Contributed::parted(Grains::from(FileRows { functions: vec![row], ..FileRows::default() }))
    }

    fn on_item_type(
        &self,
        item: &ItemType,
        _: &Facts<'_, ItemType>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let frame = self.frame(&around);
        let generics = InScope::outside().extended(&item.generics);
        let target = generics.lowered(&item.ty);
        let held = frame.hold(Holder::named(&item.ident, Held::Alias), Link::declaring(&item.ident), target.clone());
        let alias = AliasRow {
            scope: frame.here.scope().clone(),
            name: UsePath::from(&item.ident),
            parameters: generics.declarations(&item.generics),
            target,
        };
        let rows = held.combine(Grains::from(FileRows { aliases: vec![alias], ..FileRows::default() }));
        frame.owned_by(&item.ident, Domain::Types, &item.attrs, rows)
    }

    fn on_item_struct(
        &self,
        item: &ItemStruct,
        _: &Facts<'_, ItemStruct>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let frame = self.frame(&around);
        let scope = frame.generics.extended(&item.generics);
        let link = Link::declaring(&item.ident);
        let rows = frame
            .attributed(&Attributes::from(item.attrs.as_slice()), &item.ident)
            .combine(frame.parameters(&Holder::named(&item.ident, Held::StructParameter), &item.generics, &link))
            .combine(frame.struct_held(item, &scope, &link));
        frame.owned_by(&item.ident, Domain::Types, &item.attrs, rows)
    }

    fn on_item_enum(
        &self,
        item: &ItemEnum,
        _: &Facts<'_, ItemEnum>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let frame = self.frame(&around);
        let scope = frame.generics.extended(&item.generics);
        let link = Link::declaring(&item.ident);
        let variants = item.variants.iter().map(|variant| {
            let unit = frame.hold(
                Holder::named(&variant.ident, Held::Variant),
                link.clone(),
                graph::TypeRef::Tuple(Vec::new()),
            );
            let holder = Holder::named(&variant.ident, Held::EnumField);
            unit.combine(frame.fields_held(&holder, &link, variant.fields.iter(), &scope))
        });
        let rows = frame
            .attributed(&Attributes::from(item.attrs.as_slice()), &item.ident)
            .combine(frame.parameters(&Holder::named(&item.ident, Held::Bound), &item.generics, &link))
            .combine(Monoid::concat(variants));
        frame.owned_by(&item.ident, Domain::Types, &item.attrs, rows)
    }

    fn on_item_union(
        &self,
        item: &ItemUnion,
        _: &Facts<'_, ItemUnion>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let frame = self.frame(&around);
        let scope = frame.generics.extended(&item.generics);
        let link = Link::declaring(&item.ident);
        let rows = frame.attributed(&Attributes::from(item.attrs.as_slice()), &item.ident).combine(frame.fields_held(
            &Holder::named(&item.ident, Held::UnionField),
            &link,
            item.fields.named.iter(),
            &scope,
        ));
        frame.owned_by(&item.ident, Domain::Types, &item.attrs, rows)
    }

    fn on_item_const(
        &self,
        item: &ItemConst,
        _: &Facts<'_, ItemConst>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let tabled = self.frame(&around).in_table();
        tabled.valued(&item.ident, &item.ty, &item.attrs)
    }

    fn on_item_static(
        &self,
        item: &ItemStatic,
        _: &Facts<'_, ItemStatic>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let tabled = self.frame(&around).in_table();
        tabled.valued(&item.ident, &item.ty, &item.attrs)
    }

    fn on_impl_item_const(
        &self,
        _: &ImplItemConst,
        _: &Facts<'_, ImplItemConst>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let tabled = self.frame(&around).in_table();
        tabled.framed(Grains::default(), Descend::Into)
    }

    fn on_expr_method_call(
        &self,
        call: &ExprMethodCall,
        _: &Facts<'_, ExprMethodCall>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        Contributed::parted(self.frame(&around).method_called(call).combine(Grains::from(BodyPart::method_entered(call))))
    }

    fn on_expr_call(
        &self,
        call: &ExprCall,
        _: &Facts<'_, ExprCall>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let frame = self.frame(&around);
        let calling = Frame { calling: lint_core::Syntax::expr_path(&call.func).map(Address::from), ..frame.clone() };
        let called = frame.path_called(call).combine(Grains::from(BodyPart::call_entered(call)));
        calling.framed(called, Descend::Into)
    }

    fn on_expr_macro(
        &self,
        expr: &ExprMacro,
        _: &Facts<'_, ExprMacro>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        Contributed::parted(self.frame(&around).macro_called(expr))
    }

    fn on_expr_struct(
        &self,
        literal: &syntax::ExprStruct,
        _: &Facts<'_, syntax::ExprStruct>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        Contributed::parted(self.frame(&around).struct_used(literal))
    }

    fn on_expr_path(
        &self,
        expression: &ExprPath,
        _: &Facts<'_, ExprPath>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        Contributed::parted(self.frame(&around).path_used(expression))
    }

    fn on_expr_match(
        &self,
        expr: &syntax::ExprMatch,
        _: &Facts<'_, syntax::ExprMatch>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let frame = self.frame(&around);
        let patterns: Grains = Monoid::concat(expr.arms.iter().map(|arm| frame.pattern_used(&arm.pat)));
        Contributed::parted(patterns.combine(Grains::from(BodyPart::branching(Branches::matched(expr)))))
    }

    fn on_expr_let(
        &self,
        expr: &syntax::ExprLet,
        _: &Facts<'_, syntax::ExprLet>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        Contributed::parted(self.frame(&around).pattern_used(&expr.pat))
    }

    fn on_local(
        &self,
        local: &syntax::Local,
        _: &Facts<'_, syntax::Local>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let frame = self.frame(&around);
        let diverging = local.init.as_ref().filter(|init| init.diverge.is_some());
        Contributed::parted(diverging.map_or_else(Grains::default, |_| frame.pattern_used(&local.pat)))
    }

    fn on_item_macro(
        &self,
        item: &ItemMacro,
        _: &Facts<'_, ItemMacro>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        let frame = self.frame(&around);
        item.ident.as_ref().map_or_else(
            || frame.clone().framed(Grains::default(), Descend::Into),
            |ident| frame.owned_by(ident, Domain::Macros, &item.attrs, Grains::default()),
        )
    }

    fn off_item_macro(
        &self,
        item: &ItemMacro,
        _: &Facts<'_, ItemMacro>,
        inner: Contribution<Framed<'read>, Grains>,
    ) -> Grains {
        inner.concluded(|framed, grown| {
            let frame = framed.0.unwrap_or_else(|| self.top.clone());
            grown.combine(frame.macro_left(item))
        })
    }

    fn on_expr_if(&self, _: &syntax::ExprIf, _: &Facts<'_, syntax::ExprIf>, _: Around<'_, Framed<'read>, Grains>) -> Grown<'read> {
        Contributed::parted(Grains::from(BodyPart::branching(Branches::branched())))
    }

    fn on_expr_while(&self, _: &syntax::ExprWhile, _: &Facts<'_, syntax::ExprWhile>, _: Around<'_, Framed<'read>, Grains>) -> Grown<'read> {
        Contributed::parted(Grains::from(BodyPart::branching(Branches::branched())))
    }

    fn on_expr_for_loop(&self, _: &syntax::ExprForLoop, _: &Facts<'_, syntax::ExprForLoop>, _: Around<'_, Framed<'read>, Grains>) -> Grown<'read> {
        Contributed::parted(Grains::from(BodyPart::branching(Branches::branched())))
    }

    fn on_expr_try(&self, _: &syntax::ExprTry, _: &Facts<'_, syntax::ExprTry>, _: Around<'_, Framed<'read>, Grains>) -> Grown<'read> {
        Contributed::parted(Grains::from(BodyPart::branching(Branches::propagating())))
    }

    fn on_expr_binary(&self, found: &syntax::ExprBinary, _: &Facts<'_, syntax::ExprBinary>, _: Around<'_, Framed<'read>, Grains>) -> Grown<'read> {
        Contributed::parted(Grains::from(BodyPart::branching(Branches::joined(found))))
    }

    fn on_pat_ident(&self, found: &syntax::PatIdent, _: &Facts<'_, syntax::PatIdent>, _: Around<'_, Framed<'read>, Grains>) -> Grown<'read> {
        Contributed::parted(Grains::from(BodyPart::rebinding(found)))
    }

    fn off_expr_method_call(&self, call: &ExprMethodCall, _: &Facts<'_, ExprMethodCall>, inner: Contribution<Framed<'read>, Grains>) -> Grains {
        inner.concluded(|_, grown| grown.combine(Grains::from(BodyPart::method_made(call))))
    }

    fn off_expr_call(&self, call: &ExprCall, _: &Facts<'_, ExprCall>, inner: Contribution<Framed<'read>, Grains>) -> Grains {
        inner.concluded(|_, grown| grown.combine(Grains::from(BodyPart::call_made(call))))
    }

    fn off_item(&self, _: &syntax::Item, _: &Facts<'_, syntax::Item>, inner: Contribution<Framed<'read>, Grains>) -> Grains {
        inner.concluded(|_, grown| grown.unbodied())
    }

    fn off_signature(&self, _: &SynSignature, _: &Facts<'_, SynSignature>, inner: Contribution<Framed<'read>, Grains>) -> Grains {
        inner.concluded(|_, grown| grown.unbodied())
    }

    fn on_macro(
        &self,
        invoked: &Macro,
        _: &Facts<'_, Macro>,
        around: Around<'_, Framed<'read>, Grains>,
    ) -> Grown<'read> {
        Contributed::parted(self.frame(&around).macro_used(invoked))
    }
}
