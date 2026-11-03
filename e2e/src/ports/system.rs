use std::ffi::OsStr;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use console_ports::{Environment, Process};
use file_ports::{Bytes, Disk, FileFailure, Files, Kind, Memory};

#[derive(Debug)]
pub struct Checkout;

impl Checkout {
    const MANIFEST: &'static str = "Cargo.toml";

    pub fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().map(Path::to_path_buf).unwrap_or_default()
    }

    pub fn manifests() -> Result<Memory, FileFailure> {
        let root = Self::root();
        let tops = Self::directories(std::slice::from_ref(&root))?;
        let below = Self::directories(&tops)?;
        let candidates: Vec<PathBuf> =
            [root].into_iter().chain(tops).chain(below).map(|directory| directory.join(Self::MANIFEST)).collect();
        let manifests: Vec<PathBuf> = candidates
            .iter()
            .zip(Disk.kinds(&candidates))
            .filter(|(_, kind)| *kind == Kind::File)
            .map(|(manifest, _)| manifest.clone())
            .collect();
        let mirrored = Memory::default();
        manifests.iter().zip(Disk.read_many(&manifests)).try_for_each(|(manifest, read)| {
            manifest.parent().map_or(Ok(()), |parent| mirrored.create_directories(parent))?;
            mirrored.write(manifest, &read?)
        })?;
        Ok(mirrored)
    }

    fn directories(parents: &[PathBuf]) -> Result<Vec<PathBuf>, FileFailure> {
        let children: Vec<PathBuf> =
            Disk.entries_many(parents).into_iter().collect::<Result<Vec<Vec<PathBuf>>, FileFailure>>()?.concat();
        Ok(children
            .iter()
            .zip(Disk.kinds(&children))
            .filter(|(_, kind)| *kind == Kind::Directory)
            .map(|(child, _)| child.clone())
            .collect())
    }
}

#[derive(Debug)]
pub struct Measurements;

impl Measurements {
    const TARGET: &'static str = "E2E_REPORT";

    pub fn record(line: &impl fmt::Display) -> io::Result<()> {
        print!("{line}");
        Process
            .variable(OsStr::new(Self::TARGET))
            .map_or(Ok(()), |path| Disk.append(Path::new(&path), &Bytes::from(line)))
            .map_err(io::Error::from)
    }
}
