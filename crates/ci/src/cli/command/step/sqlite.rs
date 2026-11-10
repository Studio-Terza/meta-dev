mod boundary;

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::Step;
use super::changed::Changed;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Invocation, Ports};
use boundary::Located;
use file_ports::Files;

#[derive(Debug)]
pub struct BuildSqlite {
    library: PathBuf,
}

impl BuildSqlite {
    const OBJECT: &'static str = "sqlite3.o";
    const ARCHIVE: &'static str = "libsqlite3.a";

    fn compiled(&self, ports: &impl Ports, source: &Path) -> Result<Exit, Failure<'_>> {
        let object = self.library.join(Self::OBJECT);
        let compile = Invocation::new("cc", Located::compile_arguments(source, &object));
        let archive =
            Invocation::new("ar", [OsString::from("rcs"), self.library.join(Self::ARCHIVE).into(), object.into()]);
        let compiled = ports.run(&compile).map_err(Failure::starting(&compile))?.exit();
        compiled.followed_by(|| ports.run(&archive).map(|run| run.exit()).map_err(Failure::starting(&archive)))
    }
}

impl Step for BuildSqlite {
    fn from_arguments(given: Vec<OsString>) -> Result<BuildSqlite, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let library = PathBuf::from(arguments.taken()?);
        arguments.ended()?;
        Ok(BuildSqlite { library })
    }

    fn carry_out(&self, ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let metadata = Changed::asked(ports, &Invocation::new("cargo", Located::METADATA))?;
        let source = metadata.as_ref().parse::<Located>()?.source()?;
        ports
            .files()
            .create_directories(&self.library)
            .map_err(|failed| Failure::unusable(self.library.as_path(), failed))?;
        self.compiled(ports, &source)
    }
}
