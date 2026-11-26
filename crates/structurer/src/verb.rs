use design_model::Sense;
use newtype::Listed;
use strum::{Display, IntoStaticStr, VariantArray};

use crate::misread::Wanted;
use crate::reading::{Call, Term};
use crate::{Directive, Misread};

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Verb {
    Place,
    Sense,
    Link,
    Propose,
    Supersede,
    Conversational,
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    #[strum(serialize = "place(<segment>, <node>, <span-relation>)")]
    Place,
    #[strum(serialize = "sense(<segment>, <kind>, <specificity>, <confidence>, <explicitness>)")]
    Sense,
    #[strum(serialize = "link(<segment>, <segment>, <segment-relation>)")]
    Link,
    #[strum(serialize = "propose(new <label>, refines(<node>), <facet>, \"<title>\")")]
    Propose,
    #[strum(serialize = "supersede(<node>, by(<node>))")]
    Supersede,
    #[strum(serialize = "conversational(<segment>)")]
    Conversational,
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum New {
    New,
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum Refines {
    Refines,
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum By {
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
                    edge: target.ending(source.segment()?, target.segment()?)?,
                    relation: relation.named(Wanted::SegmentRelation)?,
                })
            }),
            Verb::Propose => call.exactly(self).and_then(|[new, refines, facet, title]: &[Term<'_>; _]| {
                Ok(Directive::Propose {
                    proposal: refines.within::<Refines>(Wanted::Refines)?.proposal(new.fresh()?)?,
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
