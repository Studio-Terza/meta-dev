use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use file_ports::FileFailure;
use invariants::Unpoisoned;
use pipeline::Measure;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use syn::Item;

use crate::failure::Failure;
use crate::gate::Gate;
use crate::source::{ModulePath, Source};
use crate::spent::{Spent, Stage};
use crate::tree::Tree;
use crate::walk::{Facts, Walker};
use crate::workspace::{CrateKind, CrateRoot, Package};

#[derive(Debug, Measure)]
pub(crate) struct Read {
    pub failures: Vec<Failure>,
    pub facts: Facts,
    pub spent: Spent,
}

impl Read {
    pub(crate) fn failed(failure: Failure) -> Read {
        Read { failures: vec![failure], ..Read::default() }
    }
}

#[derive(Debug)]
struct Module {
    real: Arc<Path>,
    path: ModulePath,
    package: Arc<Package>,
    kind: CrateKind,
    nesting: Nesting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Nesting {
    Beside,
    Below,
}

#[derive(Debug)]
struct Declared {
    name: String,
    inline: Vec<String>,
    path: Option<String>,
    gate: Gate,
}

#[derive(Debug)]
pub(crate) struct Reader<'run> {
    tree: &'run Tree,
    seen: Mutex<BTreeSet<Arc<Path>>>,
}

impl<'run> Reader<'run> {
    const MODULE_FILE: &'static str = "mod.rs";

    pub(crate) fn new(tree: &'run Tree) -> Reader<'run> {
        Reader { tree, seen: Mutex::default() }
    }

    pub(crate) fn read(&self, roots: Vec<CrateRoot>) -> Read {
        let (binaries, others): (Vec<Module>, Vec<Module>) = roots
            .into_iter()
            .map(|root| Module {
                real: Arc::from(root.path()),
                path: ModulePath::root(Arc::from(root.path())),
                package: Arc::clone(root.package()),
                kind: root.kind(),
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
        if fresh { self.parsed(module) } else { Read::default() }
    }

    fn parsed(&self, module: &Module) -> Read {
        let shown = self.tree.shown(&module.real);
        let text = self.tree.text(&module.real).unwrap_or_default().to_owned();
        let mut spent = Spent::default();
        let parsed = spent.timed(Stage::Parse, || syn::parse_file(&text));
        match parsed {
            Ok(file) => {
                let declared = Reader::declared(&file.items, &[], module.path.gate());
                let children: Vec<Result<Module, Failure>> =
                    declared.iter().map(|declaration| self.child(module, declaration)).collect();
                let source = Source::new(
                    Arc::clone(&module.package),
                    Arc::clone(&module.real),
                    shown,
                    module.path.clone(),
                    module.kind,
                    text,
                );
                let facts = spent.timed(Stage::Visit, || Walker::walked(&source, &file));
                self.below(Read { failures: Vec::new(), facts, spent }, children)
            }
            Err(source) => {
                Read::failed(Failure::Parse { path: shown.to_path_buf(), line: source.span().start().line, source })
            }
        }
    }

    fn below(&self, here: Read, children: Vec<Result<Module, Failure>>) -> Read {
        let below = children
            .into_par_iter()
            .map(|child| match child {
                Ok(found) => self.claimed(&found),
                Err(failure) => Read::failed(failure),
            })
            .reduce(Read::default, Read::joined);
        here.joined(below)
    }

    fn declared(items: &[Item], inline: &[String], gate: Gate) -> Vec<Declared> {
        let modules = items.iter().filter_map(|item| match item {
            Item::Mod(module) => Some(module),
            _ => None,
        });
        modules
            .flat_map(|module| {
                let gated = gate.and(Gate::of(&module.attrs));
                let name = module.ident.to_string();
                match &module.content {
                    Some((_, inner)) => {
                        let deeper: Vec<String> = inline.iter().cloned().chain([name]).collect();
                        Reader::declared(inner, &deeper, gated)
                    }
                    None => vec![Declared {
                        name,
                        inline: inline.to_vec(),
                        path: Tree::path_attribute(&module.attrs),
                        gate: gated,
                    }],
                }
            })
            .collect()
    }

    fn child(&self, parent: &Module, declared: &Declared) -> Result<Module, Failure> {
        let real = &parent.real;
        let directory = real.parent().map_or_default(Path::to_path_buf);
        let own = match parent.nesting {
            Nesting::Beside => directory.clone(),
            Nesting::Below => directory.join(real.file_stem().unwrap_or_default()),
        };
        let nested = declared.inline.iter().fold(own, |inside, name| inside.join(name));
        let candidates: Vec<PathBuf> = match &declared.path {
            Some(path) if declared.inline.is_empty() => vec![directory.join(path)],
            Some(path) => vec![nested.join(path)],
            None => vec![
                nested.join(format!("{}.rs", declared.name)),
                nested.join(&declared.name).join(Reader::MODULE_FILE),
            ],
        };
        let chosen = candidates
            .iter()
            .map(|candidate| Tree::lexical(candidate))
            .find(|resolved| self.tree.text(resolved).is_some())
            .ok_or_else(|| Failure::Read {
                path: candidates.first().map_or_default(|first| self.tree.shown(first).to_path_buf()),
                source: FileFailure::Missing(io::Error::from(io::ErrorKind::NotFound)),
            })?;
        let nesting =
            if chosen.file_name() == Some(OsStr::new(Reader::MODULE_FILE)) { Nesting::Beside } else { Nesting::Below };
        Ok(Module {
            real: Arc::from(chosen),
            path: parent.path.child(declared.inline.iter().cloned().chain([declared.name.clone()]), declared.gate),
            package: Arc::clone(&parent.package),
            kind: parent.kind,
            nesting,
        })
    }
}
