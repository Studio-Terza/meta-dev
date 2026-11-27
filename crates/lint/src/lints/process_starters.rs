use proc_macro2::{TokenStream, TokenTree};
use syn::spanned::Spanned;
use syn::{ItemMod, ItemUse, Macro, Path};

use crate::finding::{Found, Rule};
use crate::source::Source;
use crate::style::{Reads, Style};
use crate::terms::{Count, Fix, Line, Name};
use crate::use_leaf::UseLeaf;
use crate::walk::{Descent, Expanded, Hooks};

#[derive(Debug)]
pub(crate) struct ProcessStarters {
    open: Vec<Module>,
    closed: Vec<Module>,
}

#[derive(Debug, Default)]
struct Module {
    ported: Option<()>,
    imports: Vec<Bound>,
    spelled: Vec<Spelled>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Bound {
    Command(Name),
    Process(Name),
}

#[derive(Debug)]
struct Spelled {
    line: Line,
    segments: Vec<Name>,
}

#[derive(Debug, Default)]
struct Run {
    segments: Vec<Name>,
    line: Option<Line>,
    colons: Count,
    found: Vec<Spelled>,
}

impl ProcessStarters {
    const PORTS: &'static str = "ports";
    const PROCESS: &'static str = "process";
    const COMMAND: &'static str = "Command";
    const NEW: &'static str = "new";
    const STD: &'static str = "std";
    const OWN: &'static str = "self";
    const FIX: Fix = Fix("move it behind a port: a crate whose name ends in ports, or a module named ports");

    fn scanning(ported: Option<()>) -> ProcessStarters {
        ProcessStarters { open: vec![Module { ported, ..Module::default() }], closed: Vec::new() }
    }

    fn current(&mut self) -> Option<&mut Module> {
        self.open.last_mut()
    }

    fn admitted(&mut self) -> Option<&mut Module> {
        self.current().filter(|module| module.ported.is_none())
    }

    fn starts(&self) -> Vec<Line> {
        self.closed
            .iter()
            .chain(&self.open)
            .flat_map(|module| module.spelled.iter().filter_map(|spelled| spelled.started(&module.imports)))
            .collect()
    }
}

impl Spelled {
    fn started(&self, imports: &[Bound]) -> Option<Line> {
        let (last, rest) = self.segments.split_last()?;
        (last == ProcessStarters::NEW).then_some(())?;
        let (owner, ahead) = rest.split_last()?;
        let imported = imports.contains(&Bound::Command(owner.clone())).then_some(());
        let qualified = ahead
            .last()
            .filter(|_| owner == ProcessStarters::COMMAND)
            .filter(|module| {
                *module == ProcessStarters::PROCESS || imports.contains(&Bound::Process((*module).clone()))
            })
            .map(|_| ());
        imported.or(qualified).map(|()| self.line)
    }
}

impl Bound {
    fn of_leaf(leaf: &UseLeaf<'_>) -> Option<Bound> {
        let parent = Name::of(leaf.path().last()?);
        let end = leaf.end();
        let spelled = end.original().zip(end.bound());
        spelled.map_or_else(
            || (parent == *ProcessStarters::PROCESS).then(|| Bound::Command(Name::spelled(ProcessStarters::COMMAND))),
            |(original, bound)| Bound::named(&parent, &Name::of(original), Name::of(bound)),
        )
    }

    fn named(parent: &Name, original: &Name, bound: Name) -> Option<Bound> {
        let in_process = (parent == ProcessStarters::PROCESS).then_some(());
        let command = in_process.filter(|()| original == ProcessStarters::COMMAND);
        let itself = in_process.filter(|()| original == ProcessStarters::OWN);
        let from_std = (parent == ProcessStarters::STD && original == ProcessStarters::PROCESS).then_some(());
        let module_name =
            if bound == *ProcessStarters::OWN { Name::spelled(ProcessStarters::PROCESS) } else { bound.clone() };
        let module = itself.or(from_std).map(|()| Bound::Process(module_name));
        command.map(|()| Bound::Command(bound)).or(module)
    }
}

impl Run {
    const SEPARATOR: usize = 2;
    const COLON: char = ':';

    fn scanned(tokens: TokenStream) -> Vec<Spelled> {
        tokens.into_iter().fold(Run::default(), Run::read).ended().found
    }

    fn read(self, tree: TokenTree) -> Run {
        match tree {
            TokenTree::Group(group) => {
                let Run { found, .. } = self.ended();
                Run { found: found.into_iter().chain(Run::scanned(group.stream())).collect(), ..Run::default() }
            }
            TokenTree::Ident(ident) => {
                let joined =
                    self.segments.first().map_or(Some(()), |_| (self.colons.number() == Run::SEPARATOR).then_some(()));
                let run = match joined {
                    Some(()) => self,
                    None => self.ended(),
                };
                Run {
                    line: run.line.or(Some(Line::of(ident.span()))),
                    segments: run.segments.into_iter().chain([Name::of(&ident)]).collect(),
                    colons: Count::default(),
                    found: run.found,
                }
            }
            TokenTree::Punct(punct) if punct.as_char() == Run::COLON => {
                Run { colons: self.colons.incremented(), ..self }
            }
            TokenTree::Punct(_) | TokenTree::Literal(_) => self.ended(),
        }
    }

    fn ended(self) -> Run {
        let Run { segments, line, colons, found } = self;
        let whole = line.filter(|_| colons == Count::default() && segments.len() >= Run::SEPARATOR);
        let spelled = whole.map(|at| Spelled { line: at, segments });
        Run { found: found.into_iter().chain(spelled).collect(), ..Run::default() }
    }
}

impl Hooks for ProcessStarters {
    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        let named = (module.ident == ProcessStarters::PORTS).then_some(());
        let ported = named.or(self.current().and_then(|enclosing| enclosing.ported));
        self.open.push(Module { ported, ..Module::default() });
        Descent::Into
    }

    fn left_item_mod(&mut self, _: &ItemMod) {
        self.closed.extend(self.open.pop());
    }

    fn item_use(&mut self, item: &ItemUse) -> Descent {
        let imports = UseLeaf::of_tree(&item.tree).iter().filter_map(Bound::of_leaf).collect::<Vec<Bound>>();
        self.current().map_or_default(|module| module.imports.extend(imports));
        Descent::Over
    }

    fn path(&mut self, path: &Path) -> Descent {
        let spelled = Spelled {
            line: Line::of(path.span()),
            segments: path.segments.iter().map(|segment| Name::of(&segment.ident)).collect(),
        };
        self.admitted().map_or_default(|module| module.spelled.push(spelled));
        Descent::Into
    }

    fn macro_called(&mut self, invocation: &Macro) -> Expanded {
        let spoken = Run::scanned(invocation.tokens.clone());
        self.admitted().map_or_default(|module| module.spelled.extend(spoken));
        Expanded::Nothing
    }
}

impl<'file> Style<'file> for ProcessStarters {
    const RULE: Rule = Rule::ProcessStarters;
    const READS: Reads = Reads::Every;

    fn new(source: &'file Source) -> ProcessStarters {
        ProcessStarters::scanning(source.ported())
    }

    fn found(self) -> Vec<Found> {
        let lines = self.starts();
        lines.into_iter().map(|line| Found::at(line, "starts a std::process::Command", ProcessStarters::FIX)).collect()
    }
}
