//! Release packaging: `library-vX.Y.Z.tar.zst` + `library-vX.Y.Z.index.json`.
//!
//! The archive is reproducible (sorted entries, fixed mtime/owner) and every
//! file it contains is listed in `index.json` with its SHA-256, so consumers
//! can verify it ([`read_archive`]).

mod archive;
mod index;

pub use archive::{BuildOptions, PackageOutput, build, read_archive};
pub use index::{FORMAT, FORMAT_VERSION, FileEntry, HazardRef, Index, MachineEntry, MaterialEntry};

use thiserror::Error;

/// Packaging errors.
#[derive(Debug, Error)]
pub enum PackageError {
    /// Filesystem or archive I/O failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON (de)serialization failure.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// The archive does not match its index.
    #[error("integrity error: {0}")]
    Integrity(String),
}

/// Lowercase hex SHA-256 of `data`.
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(data).iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn sha256() {
        assert_eq!(
            super::sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
