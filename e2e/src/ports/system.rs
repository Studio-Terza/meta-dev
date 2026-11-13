use std::ffi::OsStr;
use std::fmt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use console_ports::{Environment, Process};
use file_ports::{Bytes, Disk, FileFailure, Files, Kind, Memory};

#[derive(Debug)]
pub struct System;

impl System {
    const MANIFEST: &'static str = "Cargo.toml";
    const TARGET: &'static str = "E2E_REPORT";

    pub fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().map_or_default(Path::to_path_buf)
    }

    pub fn manifests() -> Result<Memory, FileFailure> {
        let root = Self::root();
        let tops = Self::directories(std::slice::from_ref(&root))?;
        let below = Self::directories(&tops)?;
        let candidates: Vec<PathBuf> =
            [root].into_iter().chain(tops).chain(below).map(|directory| directory.join(Self::MANIFEST)).collect();
        let kinds = Disk.kinds(&candidates);
        let manifests: Vec<PathBuf> = candidates
            .into_iter()
            .zip(kinds)
            .filter(|(_, kind)| *kind == Kind::File)
            .map(|(manifest, _)| manifest)
            .collect();
        let mirrored = Memory::default();
        manifests.iter().zip(Disk.read_many(&manifests)).try_for_each(|(manifest, read)| {
            manifest.parent().map_or(Ok(()), |parent| mirrored.create_directories(parent))?;
            mirrored.write(manifest, read?)
        })?;
        Ok(mirrored)
    }

    fn directories(parents: &[PathBuf]) -> Result<Vec<PathBuf>, FileFailure> {
        let children: Vec<PathBuf> = Disk
            .entries_many(parents)
            .into_iter()
            .collect::<Result<Vec<Vec<PathBuf>>, FileFailure>>()?
            .into_iter()
            .flatten()
            .collect();
        let kinds = Disk.kinds(&children);
        Ok(children
            .into_iter()
            .zip(kinds)
            .filter(|(_, kind)| *kind == Kind::Directory)
            .map(|(child, _)| child)
            .collect())
    }

    pub fn record(line: &impl fmt::Display) -> io::Result<()> {
        write!(io::stdout(), "{line}")?;
        Process
            .variable(OsStr::new(Self::TARGET))
            .map_or(Ok(()), |path| Disk.append(Path::new(&path), &Bytes::from(line)))
            .map_err(io::Error::from)
    }
}
