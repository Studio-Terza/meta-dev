use std::env;
use std::path::PathBuf;

use file_doubles::Memory;
use file_ports::Disk;
use file_ports::files_ops::{Call, Reply};
use fixture::{Failed, Fixture, Laws, Rows};
use port_core::Run;
use serde::Deserialize;
use tempfile::TempDir;

#[derive(Fixture, Deserialize, Debug)]
struct Script {
    calls: Vec<Call>,
    #[serde(default)]
    canonical: Vec<PathBuf>,
}

#[test]
fn disk_answers_every_script_as_memory_does_and_resolves_a_canonical_path_to_itself() -> Result<(), Failed> {
    Rows::<Script>::read()?.into_iter().try_for_each(|script| {
        let scratch = TempDir::new()?;
        env::set_current_dir(scratch.path())?;
        let run = Run::root();
        let (disk, memory) = (run.port(Disk), run.port(Memory::default()));
        Laws::agreed(&script.calls, |call| call.replayed(&disk), |call| call.replayed(&memory))?;
        let resolved: Vec<(Reply, Reply)> = script
            .canonical
            .into_iter()
            .filter_map(|path| match Call::Canonical(path).replayed(&disk) {
                Reply::Canonical(Some(once)) => {
                    let again = Call::Canonical(once.clone()).replayed(&disk);
                    Some((again, Reply::Canonical(Some(once))))
                }
                Reply::Canonical(None) => None,
                unasked => Some((unasked, Reply::Canonical(None))),
            })
            .collect();
        let unsettled: Vec<&(Reply, Reply)> = resolved.iter().filter(|(again, once)| again != once).collect();
        assert_eq!(unsettled, Vec::<&(Reply, Reply)>::new());
        drop(memory);
        run.finish();
        Ok(())
    })
}
