use std::ffi::OsString;
use std::iter;

use super::crates::Crates;
use super::manifest::Manifest;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::changed::Changed;
use crate::cli::ports::{Invocation, Ports};
use text_lines::Lines;

#[derive(Debug)]
pub(super) struct Host(OsString);

impl Host {
    const ASKED: [&'static str; 2] = ["--print", "host-tuple"];
    const METADATA: [&'static str; 5] = ["metadata", "--locked", "--format-version", "1", "--filter-platform"];

    pub(super) fn asked(ports: &impl Ports) -> Result<Host, Failure<'static>> {
        let named = Changed::asked(ports, &Invocation::new("rustc", Self::ASKED))?;
        let lines: Vec<String> = named.as_ref().parse::<Lines>().into_iter().flatten().collect();
        <[String; 1]>::try_from(lines).map(|[host]| Host(OsString::from(host))).map_err(|_lines| Failure::NoHost(named))
    }

    pub(super) fn crates(self, ports: &impl Ports) -> Result<Crates, Failure<'static>> {
        let arguments = Self::METADATA.into_iter().map(OsString::from).chain(iter::once(self.0));
        let listed = Changed::asked(ports, &Invocation::new("cargo", arguments))?;
        serde_json::from_str::<Manifest>(listed.as_ref()).map(Manifest::crates).map_err(Failure::Metadata)
    }
}
