use design_model::Sense;
use named_derive::Named;

use crate::misread::Wanted;
use crate::reading::{Call, Term};
use crate::{Directive, Misread};

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    #[name("place")]
    Place,
    #[name("sense")]
    Sense,
    #[name("link")]
    Link,
    #[name("propose")]
    Propose,
    #[name("supersede")]
    Supersede,
    #[name("conversational")]
    Conversational,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    #[name("place(<segment>, <node>, <span-relation>)")]
    Place,
    #[name("sense(<segment>, <kind>, <specificity>, <confidence>, <explicitness>)")]
    Sense,
    #[name("link(<segment>, <segment>, <segment-relation>)")]
    Link,
    #[name("propose(new [<label>], refines(<node>), <facet>, \"<title>\")")]
    Propose,
    #[name("supersede(<node>, by(<node>))")]
    Supersede,
    #[name("conversational(<segment>)")]
    Conversational,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum New {
    #[name("new")]
    New,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Refines {
    #[name("refines")]
    Refines,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum By {
    #[name("by")]
    By,
}

impl Verb {
    #[must_use]
    pub const fn form(self) -> Form {
        match self {
            Verb::Place => Form::Place,
            Verb::Sense => Form::Sense,
            Verb::Link => Form::Link,
            Verb::Propose => Form::Propose,
            Verb::Supersede => Form::Supersede,
            Verb::Conversational => Form::Conversational,
        }
    }

    pub(crate) fn directive(self, call: &Call<'_>) -> Result<Directive, Misread> {
        match self {
            Verb::Place => call.exactly(self).and_then(|[segment, node, relation]: &[Term<'_>; _]| {
                Ok(Directive::Place {
                    segment: segment.segment()?,
                    node: node.node()?,
                    relation: relation.named(Wanted::SpanRelation)?,
                })
            }),
            Verb::Sense => {
                call.exactly(self).and_then(|[segment, kind, specificity, confidence, explicitness]: &[Term<'_>; _]| {
                    let sense = Sense::new(
                        kind.named(Wanted::SenseKind)?,
                        specificity.named(Wanted::Specificity)?,
                        confidence.named(Wanted::Confidence)?,
                        explicitness.named(Wanted::Explicitness)?,
                    );
                    Ok(Directive::Sense { segment: segment.segment()?, sense })
                })
            }
            Verb::Link => call.exactly(self).and_then(|[source, target, relation]: &[Term<'_>; _]| {
                Ok(Directive::Link {
                    source: source.segment()?,
                    target: target.segment()?,
                    relation: relation.named(Wanted::SegmentRelation)?,
                })
            }),
            Verb::Propose => call.exactly(self).and_then(|[new, refines, facet, title]: &[Term<'_>; _]| {
                Ok(Directive::Propose {
                    fresh: new.fresh()?,
                    refines: refines.within::<Refines>(Wanted::Refines)?.node()?,
                    facet: facet.named(Wanted::Facet)?,
                    title: title.title()?,
                })
            }),
            Verb::Supersede => {
                call.exactly(self).and_then(|[old, by]: &[Term<'_>; _]| old.replaced_by(by).map(Directive::Supersede))
            }
            Verb::Conversational => call
                .exactly(self)
                .and_then(|[segment]: &[Term<'_>; _]| Ok(Directive::Conversational { segment: segment.segment()? })),
        }
    }
}
