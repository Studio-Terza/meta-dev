use super::{Frame, Grain, Grains};
use algebra::{Act, Monoid, Semigroup};
mod tokens;

use syntax::Spanned;
use syntax::{ExprPath, ExprStruct, Macro, Pat, PatTree, QSelf, Type, TypeParamBound};

use graph::{Domain, ItemPath, Member, Mention, ModelPart, PathUse, Region, Resolved, Seat, Segment};
use lint_layout::{Name, Position, Whole, Word, words};

use crate::namespace::{Invoked, Invoker, UsePath};

use super::places::PlaceId;
use super::quotes::QuoteRow;
use super::seat::{Footing, Located, Lowers};

use lint_sweep::Address;
use tokens::TokenPaths;

#[derive(Debug)]
pub(super) struct UseRow {
    located: Located,
    written: UsePath,
    mention: Mention,
    region: Region,
    place: Option<PlaceId>,
}

impl UseRow {
    fn domain(&self) -> Domain {
        match self.mention {
            Mention::Invoked => Domain::Macros,
            Mention::Expression
            | Mention::InMacro
            | Mention::Interpolated
            | Mention::Dispatched
            | Mention::Typed
            | Mention::Matched
            | Mention::Expanded => Domain::Types,
        }
    }

    const SELF: [Word<'static>; 1] = words!["Self"];
    const OWNED_LENGTH: usize = 2;

    fn owned(&self, seat: &Seat) -> Option<Resolved> {
        let written = self.written.written();
        let segments: &[Name] = written.as_ref();
        let last = segments.get(1).filter(|_| segments.len() == Self::OWNED_LENGTH)?;
        segments.first()?.matches::<Whole>(&Self::SELF)?;
        Some(Resolved::Member(Member::new(seat.owner()?, last.clone())))
    }

    fn expanded(&self, footing: Footing<'_>, resolved: &Resolved, invoked: &Invoked) -> ModelPart {
        match self.mention {
            Mention::Invoked => footing.resolving().transcriptions().expanded(
                resolved,
                Invoker::new(footing.resolving(), footing.scopes().scope(self.located.scope()), invoked),
            ),
            Mention::Expression
            | Mention::InMacro
            | Mention::Interpolated
            | Mention::Dispatched
            | Mention::Typed
            | Mention::Matched
            | Mention::Expanded => ModelPart::default(),
        }
    }

    fn kept<'resolved>(&self, resolved: &'resolved Resolved) -> Option<&'resolved Resolved> {
        match self.mention {
            Mention::Invoked | Mention::Dispatched => Some(resolved),
            Mention::Expression
            | Mention::InMacro
            | Mention::Interpolated
            | Mention::Typed
            | Mention::Matched
            | Mention::Expanded => self.written.tail().first().map(|_| resolved).or_else(|| resolved.known()),
        }
    }
}

impl Lowers for UseRow {
    fn lowered(self, footing: Footing<'_>) -> ModelPart {
        let resolving = footing.resolving();
        let scope = footing.scopes().scope(self.located.scope());
        let direct = resolving.path(scope, &self.written, self.domain());
        let head = direct
            .unresolved()
            .and_then(|_| Some(resolving.path(scope, &self.written.qualifier()?, Domain::Types)))
            .zip(self.written.tail().last());
        let declared = footing.aliases();
        let target = |parent: &Resolved| declared.target(parent).map(AsRef::<Resolved>::as_ref);
        let aliased = head
            .as_ref()
            .and_then(|(parent, last)| target(parent)?.clone().acted(ItemPath::from(Segment::from(*last))));
        let seat = footing.places().seat(self.region.combine(footing.outer()), self.place.as_ref());
        let member = head.as_ref().and_then(|(parent, last)| {
            let owner = target(parent).and_then(Resolved::crate_item).or_else(|| parent.crate_item())?;
            Some(Resolved::Member(Member::new(owner, (*last).clone())))
        });
        let valued = || {
            direct.unresolved().and(Some(self.domain())).filter(|domain| *domain == Domain::Types)?;
            Some(resolving.path(scope, &self.written, Domain::Values)).filter(|resolved| resolved.known().is_some())
        };
        let resolved = self.owned(&seat).or(aliased).or(member).or_else(valued).unwrap_or(direct);
        let written = self.written.written();
        let line = self.located.position().line();
        let invoked = Invoked::new(footing.file(), line, seat.clone());
        let expanded = self.expanded(footing, &resolved, &invoked);
        let named = self.kept(&resolved).map(|_| ()).map(|()| resolved);
        let used = named.map(|kept| PathUse::new(written, kept, self.mention, seat));
        expanded.combine(used.map_or_default(|found| ModelPart::path_use(footing.file(), line, found)))
    }
}
#[derive(Debug)]
struct Dispatch;

impl Dispatch {
    const OWNED_MEMBER: usize = 2;

    fn objects(ty: &Type) -> Vec<UsePath> {
        match ty {
            Type::TraitObject(object) => object.bounds.iter().filter_map(Self::traited).collect(),
            _ => Vec::new(),
        }
    }

    fn traited(bound: &TypeParamBound) -> Option<UsePath> {
        match bound {
            TypeParamBound::Trait(traited) => Some(UsePath::from(&traited.path)),
            _ => None,
        }
    }

    fn qualified(qself: &QSelf, path: &syntax::Path) -> Option<UsePath> {
        let segments: syntax::Punctuated<syntax::PathSegment, syntax::Token![::]> =
            path.segments.iter().take(qself.position).cloned().collect();
        let present = segments.first().map(|_| ());
        present.map(|()| UsePath::from(&syntax::Path { leading_colon: path.leading_colon, segments }))
    }
}

#[derive(Debug)]
struct Typed;

impl Typed {
    fn path(ty: &Type) -> Option<&syntax::Path> {
        match ty {
            Type::Path(inner) => inner.qself.as_ref().map_or(Some(&inner.path), |_| None),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Qualified;

impl Qualified {
    fn written(expression: &ExprPath) -> Option<UsePath> {
        expression
            .qself
            .as_ref()
            .map_or_else(|| Some(UsePath::from(&expression.path)), |qself| Self::under_type(qself, &expression.path))
    }

    fn under_type(qself: &QSelf, path: &syntax::Path) -> Option<UsePath> {
        let trait_named = qself.position.checked_sub(1);
        let own = trait_named.map_or(Some(&*qself.ty), |_| None);
        own.and_then(Self::type_path).map(|inner| UsePath::from(inner).combine(UsePath::from(path)))
    }

    fn type_path(ty: &Type) -> Option<&syntax::Path> {
        match ty {
            Type::Path(inner) => Some(&inner.path),
            _ => None,
        }
    }
}

impl Frame<'_> {
    pub(super) fn path_used(&self, expression: &ExprPath) -> Grains {
        let position = expression.path.segments.first().map(|first| Position::from(first.ident.span()));
        let called = self.calling.filter(|at| *at == Address::from(expression));
        let tabled = called.map_or(Some(()), |_| None).and(self.tabled);
        let dispatched = tabled.map_or_default(|()| self.dispatched(expression));
        let mentioned = dispatched.into_iter().zip(position).map(|(path, at)| self.used(path, at, Mention::Dispatched));
        let found =
            Qualified::written(expression).zip(position).map(|(path, at)| self.used(path, at, Mention::Expression));
        Monoid::concat(mentioned.chain(found))
    }

    pub(super) fn pattern_used(&self, pat: &Pat) -> Grains {
        let reached = PatTree::reached(pat, &|_, _| true);
        let used = reached
            .iter()
            .filter_map(|found| Matched::path(found.pat()))
            .map(|path| self.used(UsePath::from(path), Position::from(path.span()), Mention::Matched));
        Monoid::concat(used)
    }

    pub(super) fn struct_used(&self, literal: &ExprStruct) -> Grains {
        let position = literal.path.segments.first().map(|first| Position::from(first.ident.span()));
        let written = literal.qself.as_ref().map_or(Some(&literal.path), |_| None).zip(position);
        written.map_or_else(Grains::default, |(path, at)| self.used(UsePath::from(path), at, Mention::Expression))
    }

    pub(super) fn types_used(&self, ty: &Type) -> Grains {
        let position = Position::from(ty.span());
        let typed =
            self.context.in_body().and_then(|_| Typed::path(ty)).map(|path| (UsePath::from(path), Mention::Typed));
        let traits = Dispatch::objects(ty).into_iter().map(|path| (path, Mention::Dispatched));
        Monoid::concat(traits.chain(typed).map(|(path, mention)| self.used(path, position, mention)))
    }

    fn dispatched(&self, expression: &ExprPath) -> Vec<UsePath> {
        let member = expression.path.segments.last().map(|last| UsePath::from(&last.ident));
        let qualified = expression.qself.as_ref().and_then(|qself| Dispatch::qualified(qself, &expression.path));
        let head =
            expression.path.segments.first().filter(|_| expression.path.segments.len() == Dispatch::OWNED_MEMBER);
        let bounded: Vec<UsePath> = head.map_or_default(|first| {
            self.generics.traits_of(&lint_layout::Name::from(&first.ident)).into_iter().cloned().collect()
        });
        let traits = qualified.into_iter().chain(bounded);
        member.map_or_default(|named| traits.map(|traited| traited.combine(named.clone())).collect())
    }

    pub(super) fn macro_used(&self, invoked: &Macro) -> Grains {
        let position = invoked
            .path
            .segments
            .last()
            .map_or_else(|| Position::from(invoked.path.span()), |last| Position::from(last.ident.span()));
        let law = self.law_row(invoked);
        let quotes = QuoteRow::found(self.source, invoked, &self.here);
        let lawful: Grains = Monoid::concat(
            law.into_iter().map(Grain::Law).chain(quotes.into_iter().map(Grain::Quote)).map(Grains::from),
        );
        let invoking = self.used(UsePath::from(&invoked.path), position, Mention::Invoked);
        let plain = TokenPaths::within(invoked.tokens.clone()).into_iter().map(|spelled| (spelled, Mention::InMacro));
        let interpolated =
            TokenPaths::interpolated(invoked).into_iter().map(|spelled| (spelled, Mention::Interpolated));
        let spelled =
            plain.chain(interpolated).map(|(spelled, mention)| spelled.handed(|path, at| self.used(path, at, mention)));
        lawful.combine(invoking).combine(Monoid::concat(spelled))
    }

    pub(super) fn used(&self, written: UsePath, position: Position, mention: Mention) -> Grains {
        let row = UseRow {
            located: self.here.located(position),
            written,
            mention,
            region: self.context.region(),
            place: self.context.place(),
        };
        Grains::from(Grain::Use(row))
    }
}

#[derive(Clone, Copy, Debug)]
struct Matched;

impl Matched {
    fn path(pat: &Pat) -> Option<&syntax::Path> {
        match pat {
            Pat::Path(path) => Some(&path.path),
            Pat::TupleStruct(variant) => Some(&variant.path),
            Pat::Struct(variant) => Some(&variant.path),
            _ => None,
        }
    }
}
