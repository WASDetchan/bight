use std::path::Path;

use rkyv::{
    Archive, Deserialize, Serialize, access, deserialize,
    rancor::{self},
    to_bytes,
};

use crate::evaluator::SourceTable;

#[derive(Archive, Serialize, Deserialize)]
#[repr(C)]
pub struct BightHeader {
    version: u64,
}

const PADDED_HEADER_SIZE: usize = 1024;
const RESERVED_SIZE: usize = PADDED_HEADER_SIZE - std::mem::size_of::<BightHeader>();

#[derive(Archive, Serialize, Deserialize)]
#[repr(C)]
struct BightHeaderPadded {
    header: BightHeader,
    _reserved: [u8; RESERVED_SIZE],
}

impl BightHeaderPadded {
    fn new(version: u64) -> Self {
        Self {
            header: BightHeader { version },
            _reserved: [0; RESERVED_SIZE],
        }
    }
}

#[derive(Archive, Serialize, Deserialize)]
pub struct BightFileV1 {
    source: SourceTable,
}

impl BightFileV1 {
    const VERSION: u64 = 2;
}

#[derive(Debug, thiserror::Error)]
pub enum FileLoadError {
    #[error(transparent)]
    IoErrror(#[from] std::io::Error),
    // #[error("The given file was invalid")]
    #[error(transparent)]
    DeserializationError(#[from] rancor::Error),
    #[error("Given data is not a valid Bight file")]
    DataError,
    #[error("Bight file version {0} is not supported")]
    UnsupportedVersion(u64),
}

pub fn load(path: &Path) -> Result<SourceTable, FileLoadError> {
    let bytes = std::fs::read(path)?;

    if bytes.is_empty() {
        return Ok(SourceTable::new());
    }

    if bytes.len() < PADDED_HEADER_SIZE {
        return Err(FileLoadError::DataError);
    }

    let Some((header_bytes, data_bytes)) = bytes.split_at_checked(PADDED_HEADER_SIZE) else {
        return Err(FileLoadError::DataError);
    };

    let archived_header = access::<ArchivedBightHeaderPadded, rancor::Error>(header_bytes)?;
    let version = archived_header.header.version.to_native();

    match version {
        BightFileV1::VERSION => {
            let archived = access::<ArchivedBightFileV1, rancor::Error>(data_bytes)?;
            let data = deserialize::<BightFileV1, rancor::Error>(archived)?;
            Ok(data.source)
        }
        _ => Err(FileLoadError::UnsupportedVersion(version)),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FileSaveError {
    #[error(transparent)]
    IoErrror(#[from] std::io::Error),
}

pub fn save(path: &Path, table: SourceTable) -> Result<(), std::io::Error> {
    let header = BightHeaderPadded::new(BightFileV1::VERSION);
    let mut bytes = to_bytes::<rancor::Error>(&header).unwrap();

    let data = BightFileV1 { source: table };
    bytes.extend_from_slice(&to_bytes::<rancor::Error>(&data).unwrap());
    std::fs::write(path, bytes)?;
    Ok(())
}
