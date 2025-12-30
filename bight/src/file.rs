use std::{fs::File, io, path::Path};

use rkyv::{Archive, Deserialize, Serialize, access, deserialize, rancor};

use crate::evaluator::SourceTable;

#[derive(Archive, Serialize, Deserialize)]
pub struct BightHeader {
    version: u64,
}

#[derive(Archive, Serialize, Deserialize)]
pub struct BightFileV1 {
    header: BightHeader,
    source: SourceTable,
}

impl BightFileV1 {
    const VERSION: u64 = 1;
}

#[derive(Debug, thiserror::Error)]
pub enum FileLoadError {
    #[error(transparent)]
    IoErrror(#[from] std::io::Error),
    // #[error("The given file was invalid")]
    #[error(transparent)]
    DeserializationError(#[from] rancor::Error),
    #[error("Bight file version {0} is not supported")]
    UnsupportedVersion(u64),
}

pub fn load(path: &Path) -> Result<SourceTable, FileLoadError> {
    let bytes = std::fs::read(path)?;

    let archived_header = access::<ArchivedBightHeader, rancor::Error>(&bytes)?;
    let version = archived_header.version.to_native();
    match version {
        BightFileV1::VERSION => {
            let archived = access::<ArchivedBightFileV1, rancor::Error>(&bytes)?;
            let data = deserialize::<BightFileV1, rancor::Error>(archived)?;
            Ok(data.source)
        }
        _ => Err(FileLoadError::UnsupportedVersion(version)),
    }
}

pub fn save(path: &Path) {}
