use std::ops::Range;
use std::sync::OnceLock;

use algebra::{Monoid, Semigroup};
use lint_layout::LineNumber;
use newtype::proptest::arbitrary::any;
use newtype::proptest::collection::vec;
use newtype::proptest::strategy::Strategy;

use crate::boundary::FileId;
use crate::call::Call;
use crate::crate_root::CrateRoots;
use crate::dispatch::Dispatch;
use crate::found::Found;
use crate::function::Function;
use crate::holding::Holding;
use crate::implementation::Implementation;
use crate::item_declaration::ItemDeclaration;
use crate::law::Law;
use crate::path_use::{Mention, PathUse};
use crate::quoted::Quoted;

use super::{Declarations, FileTable, Filed, Model};

#[derive(newtype::Arbitrary, Clone, Debug, Default, PartialEq, Eq)]
#[newtype(generate = ModelPart::sampled)]
pub struct ModelPart {
    implementations: Vec<Filed<Implementation>>,
    functions: Vec<Filed<Function>>,
    holdings: Vec<Filed<Holding>>,
    calls: Vec<Filed<Call>>,
    uses: Vec<Filed<PathUse>>,
    interpolated: Vec<Filed<PathUse>>,
    typed: Vec<Filed<PathUse>>,
    dispatched: Vec<Dispatch>,
    placed: Vec<Filed<ItemDeclaration>>,
    laws: Vec<Filed<Law>>,
    quoted: Vec<Filed<Quoted>>,
}

impl Semigroup for ModelPart {
    fn combine(self, other: ModelPart) -> ModelPart {
        ModelPart {
            implementations: self.implementations.combine(other.implementations),
            functions: self.functions.combine(other.functions),
            holdings: self.holdings.combine(other.holdings),
            calls: self.calls.combine(other.calls),
            uses: self.uses.combine(other.uses),
            interpolated: self.interpolated.combine(other.interpolated),
            typed: self.typed.combine(other.typed),
            dispatched: self.dispatched.combine(other.dispatched),
            placed: self.placed.combine(other.placed),
            laws: self.laws.combine(other.laws),
            quoted: self.quoted.combine(other.quoted),
        }
    }
}

impl Monoid for ModelPart {}

impl ModelPart {
    const SAMPLED: Range<usize> = 0..3;

    fn sampled() -> impl Strategy<Value = ModelPart> {
        Strategy::prop_map(
            (
                vec(any::<Filed<Implementation>>(), ModelPart::SAMPLED),
                vec(any::<Filed<Function>>(), ModelPart::SAMPLED),
                vec(any::<Filed<Holding>>(), ModelPart::SAMPLED),
                vec(any::<Filed<Call>>(), ModelPart::SAMPLED),
                vec(any::<Filed<PathUse>>(), ModelPart::SAMPLED),
                vec(any::<Filed<PathUse>>(), ModelPart::SAMPLED),
                vec(any::<Filed<PathUse>>(), ModelPart::SAMPLED),
                vec(any::<Dispatch>(), ModelPart::SAMPLED),
                vec(any::<Filed<ItemDeclaration>>(), ModelPart::SAMPLED),
                vec(any::<Filed<Law>>(), ModelPart::SAMPLED),
                vec(any::<Filed<Quoted>>(), ModelPart::SAMPLED),
            ),
            |(
                implementations,
                functions,
                holdings,
                calls,
                uses,
                interpolated,
                typed,
                dispatched,
                placed,
                laws,
                quoted,
            )| {
                ModelPart {
                    implementations,
                    functions,
                    holdings,
                    calls,
                    uses,
                    interpolated,
                    typed,
                    dispatched,
                    placed,
                    laws,
                    quoted,
                }
            },
        )
    }

    #[must_use]
    pub fn implementation(file: FileId, found: Found<Implementation>) -> ModelPart {
        ModelPart { implementations: vec![Filed::new(file, found)], ..ModelPart::default() }
    }

    #[must_use]
    pub fn function(file: FileId, line: LineNumber, function: Function) -> ModelPart {
        ModelPart { functions: vec![Filed::new(file, Found::new(line, function))], ..ModelPart::default() }
    }

    #[must_use]
    pub fn holding(file: FileId, line: LineNumber, holding: Holding) -> ModelPart {
        ModelPart { holdings: vec![Filed::new(file, Found::new(line, holding))], ..ModelPart::default() }
    }

    #[must_use]
    pub fn path_use(file: FileId, line: LineNumber, used: PathUse) -> ModelPart {
        match used.mention() {
            Mention::Interpolated => {
                ModelPart { interpolated: vec![Filed::new(file, Found::new(line, used))], ..ModelPart::default() }
            }
            Mention::Typed => {
                ModelPart { typed: vec![Filed::new(file, Found::new(line, used))], ..ModelPart::default() }
            }
            Mention::Dispatched => ModelPart { dispatched: vec![Dispatch::from(&used)], ..ModelPart::default() },
            Mention::Expression | Mention::Invoked | Mention::InMacro | Mention::Matched | Mention::Expanded => {
                ModelPart { uses: vec![Filed::new(file, Found::new(line, used))], ..ModelPart::default() }
            }
        }
    }

    #[must_use]
    pub fn law(file: FileId, line: LineNumber, law: Law) -> ModelPart {
        ModelPart { laws: vec![Filed::new(file, Found::new(line, law))], ..ModelPart::default() }
    }

    #[must_use]
    pub fn quoted(file: FileId, line: LineNumber, quoted: Quoted) -> ModelPart {
        ModelPart { quoted: vec![Filed::new(file, Found::new(line, quoted))], ..ModelPart::default() }
    }

    #[must_use]
    pub fn call(file: FileId, line: LineNumber, call: Call) -> ModelPart {
        ModelPart { calls: vec![Filed::new(file, Found::new(line, call))], ..ModelPart::default() }
    }

    #[must_use]
    pub fn placed(file: FileId, line: LineNumber, item: ItemDeclaration) -> ModelPart {
        ModelPart { placed: vec![Filed::new(file, Found::new(line, item))], ..ModelPart::default() }
    }
}

impl Model {
    #[must_use]
    pub fn lowered(files: FileTable, declarations: Declarations, part: ModelPart) -> Model {
        Model {
            files: files.into(),
            implementations: Filed::sorted(part.implementations),
            functions: Filed::sorted(part.functions),
            holdings: part.holdings,
            calls: part.calls,
            placed: Filed::sorted(part.placed),
            laws: Filed::sorted(part.laws),
            uses: Filed::sorted(part.uses),
            interpolated: Filed::sorted(part.interpolated),
            typed: Filed::sorted(part.typed),
            dispatched: part.dispatched,
            quoted: Filed::sorted(part.quoted),
            declarations,
            roots: CrateRoots::default(),
            references: OnceLock::new(),
        }
    }
}
