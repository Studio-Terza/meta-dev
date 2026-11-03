mod build;
mod fortnight;
mod notice;

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use super::Step;
use super::issue::Issue;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};
use build::Build;
use fortnight::Fortnight;
use notice::Notice;

#[derive(Debug)]
pub struct CompileTrend {
    today: PathBuf,
    fortnight: Fortnight,
}

impl CompileTrend {
    const TITLE: &'static str = "compile-time trend: the cold build is slower than every run of the previous 14 days";
}

impl Step for CompileTrend {
    fn from_arguments(arguments: &[OsString]) -> Result<CompileTrend, Failure> {
        match arguments {
            [today, downloads] => Ok(CompileTrend {
                today: PathBuf::from(today),
                fortnight: Fortnight::downloading_to(PathBuf::from(downloads)),
            }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        let today = Build::try_from(self.today.as_path())?;
        let earlier = self.fortnight.downloaded(ports)?;
        let notice = Notice::new(&today, &earlier, &self.fortnight);
        if let Some(slower) = notice.slower() {
            let merged = Fortnight::merged(ports)?;
            let report = slower.reported(&merged);
            write!(out, "{report}")?;
            Issue::titled(Self::TITLE, ports)?.filed(&report, ports, out)
        } else {
            writeln!(out, "{notice}")?;
            Ok(Exit::Success)
        }
    }
}
