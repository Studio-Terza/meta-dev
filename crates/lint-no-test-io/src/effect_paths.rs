use std::collections::BTreeMap;

use graph::{External, ItemId, ItemPath, Model, PathUse, Resolved};
use lint_layout::{Name, PathTail, Word, words};
use newtype::Wrapping;

#[derive(Debug)]
pub struct EffectPaths {
    declared: BTreeMap<ItemId, ItemPath>,
}

impl From<&Model> for EffectPaths {
    fn from(model: &Model) -> EffectPaths {
        let declared = model
            .items()
            .filter_map(|placed| {
                let item = model.declared(placed.entity().declared())?;
                Some((placed.entity().declared(), [placed.krate(), item.as_ref()].into_iter().cloned().collect()))
            })
            .collect();
        EffectPaths { declared }
    }
}

impl EffectPaths {
    const EFFECTS: [&[Word<'static>]; 8] = [
        &words!["std", "fs"],
        &words!["std", "net"],
        &words!["std", "env"],
        &words!["std", "process"],
        &words!["std", "os", "unix", "fs"],
        &words!["std", "os", "unix", "net"],
        &words!["std", "os", "windows", "fs"],
        &words!["tempfile"],
    ];
    const WORLD: [&[Word<'static>]; 7] = [
        &words!["std", "io", "stdin"],
        &words!["std", "io", "stdout"],
        &words!["std", "io", "stderr"],
        &words!["file_ports", "Disk"],
        &words!["console_ports", "Process"],
        &words!["golden", "Disk"],
        &words!["golden", "Process"],
    ];
    const VALUES: [&[Word<'static>]; 1] = [&words!["std", "process", "ExitCode"]];

    pub fn effect<'used>(&'used self, used: &'used PathUse) -> Option<&'used ItemPath> {
        let candidates = self.candidates(used);
        let effect = Self::EFFECTS
            .iter()
            .find_map(|root| candidates.under(root))
            .filter(|_| Self::VALUES.iter().all(|value| candidates.under(value).is_none()));
        effect.or_else(|| Self::WORLD.iter().find_map(|world| candidates.named(PathTail::new(world))))
    }

    fn candidates<'used>(&'used self, used: &'used PathUse) -> Candidates<'used> {
        let resolved: &Resolved = used.resolved();
        let declared = resolved.crate_item().and_then(|id| self.declared.get(&id));
        Candidates(
            resolved
                .std()
                .into_iter()
                .chain(resolved.external().map(External::path))
                .chain(declared)
                .chain([used.written()])
                .collect(),
        )
    }
}

#[derive(Debug)]
struct Candidates<'used>(Vec<&'used ItemPath>);

impl<'used> Candidates<'used> {
    fn under(&self, root: &[Word<'_>]) -> Option<&'used ItemPath> {
        self.0.iter().copied().filter(|candidate| AsRef::<[Name]>::as_ref(*candidate).len() > root.len()).find(
            |candidate| {
                AsRef::<[Name]>::as_ref(*candidate)
                    .iter()
                    .zip(root)
                    .all(|(segment, spelled)| *segment == Name::from(*spelled))
            },
        )
    }

    fn named(&self, world: PathTail<'_>) -> Option<&'used ItemPath> {
        let spelled = self.0.iter().copied().find_map(|candidate| candidate.ends_with(world));
        spelled.or_else(|| self.under(world.into_inner()))
    }
}
