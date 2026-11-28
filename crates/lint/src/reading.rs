use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use file_ports::{FileFailure, Files, Kind};
use invariants::{Ignoring, Unpoisoned};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use syn::{Expr, ExprLit, Item, ItemMod, Lit, Meta};
use timer_ports::Timer;

use crate::budget::Budget;
use crate::failure::Failure;
use crate::finding::Finding;
use crate::gate::Gate;
use crate::ignored::Ignored;
use crate::lints::{Facts, Lints};
use crate::source::{ModulePath, Source};
use crate::spent::{Spent, Stage};
use crate::terms::{Line, Name, Text};
use crate::workspace::{CrateKind, CrateRoot, Package, Tree};

#[derive(Debug, Default)]
pub(crate) struct Read {
    pub failures: Vec<Failure>,
    pub findings: Vec<Finding>,
    pub facts: Facts,
    pub budget: Budget,
    pub spent: Spent,
}

impl Read {
    pub(crate) fn failed(failure: Failure) -> Read {
        Read { failures: vec![failure], ..Read::default() }
    }

    pub(crate) fn joined(self, later: Read) -> Read {
        Read {
            failures: self.failures.into_iter().chain(later.failures).collect(),
            findings: self.findings.into_iter().chain(later.findings).collect(),
            facts: self.facts.joined(later.facts),
            budget: self.budget + later.budget,
            spent: self.spent.joined(later.spent),
        }
    }
}

#[derive(Debug)]
struct Module {
    real: Arc<Path>,
    path: ModulePath,
    package: Arc<Package>,
    kind: CrateKind,
    gate: Gate,
    nesting: Nesting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Nesting {
    Beside,
    Below,
}

#[derive(Debug)]
struct Declared {
    name: Name,
    inline: Vec<Name>,
    path: Option<PathBuf>,
    gate: Gate,
}

#[derive(Debug)]
pub(crate) struct Reader<'run, F, T> {
    files: &'run F,
    timer: &'run T,
    tree: &'run Tree,
    seen: Mutex<BTreeSet<Arc<Path>>>,
}

impl<'run, F: Files + Sync, T: Timer + Sync> Reader<'run, F, T> {
    const MODULE_FILE: &'static str = "mod.rs";
    const PATH_ATTRIBUTE: &'static str = "path";

    pub(crate) fn new(files: &'run F, timer: &'run T, tree: &'run Tree) -> Reader<'run, F, T> {
        Reader { files, timer, tree, seen: Mutex::default() }
    }

    pub(crate) fn read(&self, roots: Vec<CrateRoot>) -> Read {
        let (binaries, others): (Vec<Module>, Vec<Module>) = roots
            .into_iter()
            .map(|root| Module {
                real: Arc::from(root.path()),
                path: ModulePath::root(Arc::from(root.path())),
                package: Arc::clone(root.package()),
                kind: root.kind(),
                gate: Gate::Entered,
                nesting: Nesting::Beside,
            })
            .partition(|module| module.kind == CrateKind::Binary);
        let libraries = self.claimed_all(others);
        libraries.joined(self.claimed_all(binaries))
    }

    fn claimed_all(&self, modules: Vec<Module>) -> Read {
        modules.into_par_iter().map(|module| self.claimed(&module)).reduce(Read::default, Read::joined)
    }

    fn claimed(&self, module: &Module) -> Read {
        let fresh = self.seen.lock().unpoisoned().insert(Arc::clone(&module.real));
        if fresh { self.fresh(module) } else { Read::default() }
    }

    fn fresh(&self, module: &Module) -> Read {
        let shown = self.tree.shown(&module.real);
        match self.files.read_text(&module.real) {
            Ok(text) => self.parsed(module, shown, Text::read(text.as_ref().to_owned())),
            Err(source) => Read::failed(Failure::Read { path: shown.to_path_buf(), source }),
        }
    }

    fn parsed(&self, module: &Module, shown: Arc<Path>, text: Text) -> Read {
        let mut spent = Spent::default();
        let parsed = spent.timed(self.timer, Stage::Parse, || syn::parse_file(text.as_str()));
        match parsed {
            Ok(file) => {
                let source = Source::new(
                    Arc::clone(&module.package),
                    Arc::clone(&module.real),
                    shown,
                    module.path.clone(),
                    module.kind,
                    text,
                );
                self.linted(module, &source, &file, spent)
            }
            Err(source) => {
                Read::failed(Failure::Parse { path: shown.to_path_buf(), line: Line::of(source.span()), source })
            }
        }
    }

    fn linted(&self, module: &Module, source: &Source, file: &syn::File, mut spent: Spent) -> Read {
        let declared = Reader::<F, T>::declared(&file.items, &[], module.gate, module.package.test_features());
        let children: Vec<Result<Module, Failure>> =
            declared.iter().map(|declaration| self.child(module, declaration)).collect();
        let gate = module.gate;
        let linted = spent.timed(self.timer, Stage::Visit, || Lints::linted(source, file, gate));
        let here =
            Read { failures: Vec::new(), findings: linted.found, facts: linted.facts, budget: linted.budget, spent };
        let below = children
            .into_par_iter()
            .map(|child| match child {
                Ok(found) => self.claimed(&found),
                Err(failure) => Read::failed(failure),
            })
            .reduce(Read::default, Read::joined);
        here.joined(below)
    }

    fn declared(items: &[Item], inline: &[Name], gate: Gate, tested: &BTreeSet<Name>) -> Vec<Declared> {
        let modules = items.iter().filter_map(|item| match item {
            Item::Mod(module) => Some(module),
            _ => None,
        });
        modules
            .flat_map(|module| {
                let gated = gate.and(Gate::of(&module.attrs)).and(Gate::featured(&module.attrs, tested));
                let name = Name::of(&module.ident);
                match &module.content {
                    Some((_, inner)) => {
                        let deeper: Vec<Name> = inline.iter().cloned().chain([name]).collect();
                        Reader::<F, T>::declared(inner, &deeper, gated, tested)
                    }
                    None => vec![Declared {
                        name,
                        inline: inline.to_vec(),
                        path: Reader::<F, T>::path_attribute(module),
                        gate: gated,
                    }],
                }
            })
            .collect()
    }

    fn path_attribute(module: &ItemMod) -> Option<PathBuf> {
        module.attrs.iter().filter(|attribute| attribute.path().is_ident(Reader::<F, T>::PATH_ATTRIBUTE)).find_map(
            |attribute| match &attribute.meta {
                Meta::NameValue(pair) => match &pair.value {
                    Expr::Lit(ExprLit { lit: Lit::Str(path), .. }) => Some(PathBuf::from(path.value())),
                    _ => None,
                },
                Meta::Path(_) | Meta::List(_) => None,
            },
        )
    }

    fn child(&self, parent: &Module, declared: &Declared) -> Result<Module, Failure> {
        let real = &parent.real;
        let directory = real.parent().map_or_default(Path::to_path_buf);
        let own = match parent.nesting {
            Nesting::Beside => directory.clone(),
            Nesting::Below => directory.join(real.file_stem().unwrap_or_default()),
        };
        let nested = declared.inline.iter().fold(own, |inside, name| inside.join(name.text()));
        let candidates: Vec<PathBuf> = match &declared.path {
            Some(path) if declared.inline.is_empty() => vec![directory.join(path)],
            Some(path) => vec![nested.join(path)],
            None => vec![
                nested.join(format!("{}.rs", declared.name)),
                nested.join(declared.name.text()).join(Reader::<F, T>::MODULE_FILE),
            ],
        };
        let chosen = candidates
            .iter()
            .filter_map(|candidate| self.files.canonical(candidate).ignoring(Ignored::Missing))
            .find(|resolved| self.files.kind(resolved) == Kind::File)
            .ok_or_else(|| Failure::Read {
                path: candidates.first().map_or_default(|first| self.tree.shown(first).to_path_buf()),
                source: FileFailure::from(io::Error::from(io::ErrorKind::NotFound)),
            })?;
        let nesting = if chosen.file_name() == Some(OsStr::new(Reader::<F, T>::MODULE_FILE)) {
            Nesting::Beside
        } else {
            Nesting::Below
        };
        Ok(Module {
            real: Arc::from(chosen),
            path: parent.path.child(declared.inline.iter().cloned().chain([declared.name.clone()])),
            package: Arc::clone(&parent.package),
            kind: parent.kind,
            gate: declared.gate,
            nesting,
        })
    }
}
