//! Building and reading the `.tar.zst` archive.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use semver::Version;

use super::index::{FORMAT, FORMAT_VERSION, FileEntry, Index, MachineEntry, MaterialEntry};
use super::{PackageError, sha256_hex};
use crate::layout::Layout;
use crate::loader::Library;
use crate::schema::SchemaKind;

/// zstd compression level used for releases.
const ZSTD_LEVEL: i32 = 19;

/// Options for [`build`].
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// Release version.
    pub version: Version,
    /// Output directory (created if missing).
    pub out_dir: PathBuf,
    /// Modification time written in every tar header (Unix seconds), for reproducible builds.
    pub mtime: u64,
}

/// Files produced by [`build`].
#[derive(Debug, Clone)]
pub struct PackageOutput {
    /// `library-vX.Y.Z.tar.zst`.
    pub archive: PathBuf,
    /// `library-vX.Y.Z.index.json` (same content as `index.json` inside the archive).
    pub index_file: PathBuf,
    /// The index.
    pub index: Index,
}

/// Base name shared by the archive, its top directory and the index file.
pub fn base_name(version: &Version) -> String {
    format!("library-v{version}")
}

fn to_json<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, PackageError> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Builds the release package of a (validated) library.
pub fn build(
    library: &Library,
    layout: &Layout,
    options: &BuildOptions,
) -> Result<PackageOutput, PackageError> {
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let add = |files: &mut BTreeMap<String, Vec<u8>>, path: String, data: Vec<u8>| {
        let entry = FileEntry {
            path: path.clone(),
            sha256: sha256_hex(&data),
        };
        files.insert(path, data);
        entry
    };

    let hazard_list = library.hazards.as_ref().map(|h| &h.doc);
    let mut materials: Vec<_> = library.materials.iter().map(|e| &e.doc).collect();
    materials.sort_by(|a, b| a.id.cmp(&b.id));
    let mut material_entries = Vec::with_capacity(materials.len());
    for m in materials {
        let file = add(&mut files, format!("materials/{}.json", m.id), to_json(m)?);
        material_entries.push(MaterialEntry::new(m, hazard_list, file));
    }

    let mut machines: Vec<_> = library.machines.iter().map(|e| &e.doc).collect();
    machines.sort_by(|a, b| a.id.cmp(&b.id));
    let mut machine_entries = Vec::with_capacity(machines.len());
    for m in machines {
        let file = add(&mut files, format!("machines/{}.json", m.id), to_json(m)?);
        machine_entries.push(MachineEntry::new(m, file));
    }

    let hazards = match hazard_list {
        Some(list) => add(&mut files, "hazards.json".into(), to_json(list)?),
        None => return Err(PackageError::Integrity("library has no hazard list".into())),
    };

    let mut extra = Vec::new();
    for kind in SchemaKind::ALL {
        extra.push(add(
            &mut files,
            format!("schema/v1/{}", kind.file_name()),
            kind.render().into_bytes(),
        ));
    }
    for name in ["LICENSE", "NOTICE"] {
        let path = layout.root().join(name);
        if path.is_file() {
            extra.push(add(&mut files, name.into(), fs::read(path)?));
        }
    }

    let index = Index {
        format: FORMAT.into(),
        format_version: FORMAT_VERSION,
        version: options.version.to_string(),
        schema: "v1".into(),
        license: "GPL-3.0-or-later".into(),
        materials: material_entries,
        machines: machine_entries,
        hazards,
        extra,
    };
    let index_bytes = to_json(&index)?;
    files.insert("index.json".into(), index_bytes.clone());

    fs::create_dir_all(&options.out_dir)?;
    let base = base_name(&options.version);
    let archive = options.out_dir.join(format!("{base}.tar.zst"));
    let index_file = options.out_dir.join(format!("{base}.index.json"));
    write_archive(&archive, &base, &files, options.mtime)?;
    fs::write(&index_file, index_bytes)?;
    Ok(PackageOutput {
        archive,
        index_file,
        index,
    })
}

fn write_archive(
    path: &Path,
    top: &str,
    files: &BTreeMap<String, Vec<u8>>,
    mtime: u64,
) -> Result<(), PackageError> {
    let encoder = zstd::Encoder::new(fs::File::create(path)?, ZSTD_LEVEL)?;
    let mut tar = tar::Builder::new(encoder);
    for (name, data) in files {
        let mut header = tar::Header::new_ustar();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(mtime);
        header.set_uid(0);
        header.set_gid(0);
        header.set_entry_type(tar::EntryType::Regular);
        tar.append_data(&mut header, format!("{top}/{name}"), data.as_slice())?;
    }
    tar.into_inner()?.finish()?;
    Ok(())
}

/// A package read back from disk, verified against its index.
#[derive(Debug, Clone)]
pub struct Package {
    /// The index.
    pub index: Index,
    /// Every file, keyed by its path relative to the top directory.
    pub files: BTreeMap<String, Vec<u8>>,
}

impl Package {
    /// Deserializes every material listed in the index.
    pub fn materials(&self) -> Result<Vec<crate::model::Material>, PackageError> {
        self.index
            .materials
            .iter()
            .map(|e| Ok(serde_json::from_slice(&self.files[&e.file.path])?))
            .collect()
    }
}

/// Reads an archive and verifies every indexed file's SHA-256.
pub fn read_archive(path: &Path) -> Result<Package, PackageError> {
    let decoder = zstd::Decoder::new(fs::File::open(path)?)?;
    let mut tar = tar::Archive::new(decoder);
    let mut files = BTreeMap::new();
    for entry in tar.entries()? {
        let mut entry = entry?;
        let name = entry.path()?.to_string_lossy().into_owned();
        let Some((_, rel)) = name.split_once('/') else {
            return Err(PackageError::Integrity(format!(
                "entry `{name}` outside the top directory"
            )));
        };
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        files.insert(rel.to_string(), data);
    }
    let index_bytes = files
        .get("index.json")
        .ok_or_else(|| PackageError::Integrity("missing index.json".into()))?;
    let index: Index = serde_json::from_slice(index_bytes)?;
    if index.format != FORMAT || index.format_version != FORMAT_VERSION {
        return Err(PackageError::Integrity(format!(
            "unsupported format {} v{}",
            index.format, index.format_version
        )));
    }
    let listed = index
        .materials
        .iter()
        .map(|e| &e.file)
        .chain(index.machines.iter().map(|e| &e.file))
        .chain(std::iter::once(&index.hazards))
        .chain(&index.extra);
    for file in listed {
        let data = files
            .get(&file.path)
            .ok_or_else(|| PackageError::Integrity(format!("missing {}", file.path)))?;
        if sha256_hex(data) != file.sha256 {
            return Err(PackageError::Integrity(format!(
                "checksum mismatch for {}",
                file.path
            )));
        }
    }
    Ok(Package { index, files })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::fixtures;

    fn options(dir: &Path) -> BuildOptions {
        BuildOptions {
            version: Version::parse("1.2.3").unwrap(),
            out_dir: dir.join("dist"),
            mtime: 0,
        }
    }

    fn sample() -> Library {
        let mut abs = fixtures::material("abs-sheet", vec![fixtures::recipe("r")]);
        abs.doc.name = "ABS sheet".into();
        fixtures::library(
            vec![fixtures::material("b", vec![fixtures::recipe("r")]), abs],
            vec![fixtures::machine("m")],
        )
    }

    #[test]
    fn build_and_read_back() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("LICENSE"), "gpl").unwrap();
        let layout = Layout::new(dir.path());
        let out = build(&sample(), &layout, &options(dir.path())).unwrap();
        assert!(out.archive.ends_with("dist/library-v1.2.3.tar.zst"));
        assert!(out.index_file.ends_with("dist/library-v1.2.3.index.json"));
        assert_eq!(out.index.materials[0].id, "abs-sheet");
        assert_eq!(out.index.materials[0].hazards[0].id, "abs");
        assert!(out.index.materials[1].hazards.is_empty());
        assert_eq!(out.index.machines[0].id, "m");
        assert!(out.index.extra.iter().any(|f| f.path == "LICENSE"));
        assert!(!out.index.extra.iter().any(|f| f.path == "NOTICE"));

        let package = read_archive(&out.archive).unwrap();
        assert_eq!(package.index, out.index);
        assert_eq!(package.materials().unwrap().len(), 2);
        assert!(package.files.contains_key("schema/v1/material.schema.json"));

        // Reproducible: same input, same bytes.
        let first = fs::read(&out.archive).unwrap();
        let again = build(&sample(), &layout, &options(dir.path())).unwrap();
        assert_eq!(first, fs::read(again.archive).unwrap());
    }

    #[test]
    fn requires_hazards() {
        let dir = tempfile::tempdir().unwrap();
        let mut lib = sample();
        lib.hazards = None;
        let err = build(&lib, &Layout::new(dir.path()), &options(dir.path())).unwrap_err();
        assert!(err.to_string().contains("no hazard list"));
    }

    fn write_tar(path: &Path, files: &[(&str, &[u8])]) {
        let enc = zstd::Encoder::new(fs::File::create(path).unwrap(), 1).unwrap();
        let mut tar = tar::Builder::new(enc);
        for (name, data) in files {
            let mut h = tar::Header::new_ustar();
            h.set_size(data.len() as u64);
            h.set_mode(0o644);
            tar.append_data(&mut h, name, *data).unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap();
    }

    #[test]
    fn detects_tampering() {
        let dir = tempfile::tempdir().unwrap();
        let out = build(&sample(), &Layout::new(dir.path()), &options(dir.path())).unwrap();
        let package = read_archive(&out.archive).unwrap();
        let index = package.files["index.json"].clone();

        let bad = dir.path().join("bad.tar.zst");
        let mut files: Vec<(String, Vec<u8>)> = package
            .files
            .iter()
            .map(|(k, v)| (format!("top/{k}"), v.clone()))
            .collect();
        files.iter_mut().find(|(k, _)| k == "top/hazards.json").unwrap().1 = b"{}".to_vec();
        let refs: Vec<(&str, &[u8])> = files.iter().map(|(k, v)| (k.as_str(), v.as_slice())).collect();
        write_tar(&bad, &refs);
        assert!(
            read_archive(&bad)
                .unwrap_err()
                .to_string()
                .contains("checksum mismatch")
        );

        write_tar(&bad, &[("top/index.json", &index)]);
        assert!(read_archive(&bad).unwrap_err().to_string().contains("missing"));

        write_tar(&bad, &[("top/other.json", b"{}")]);
        assert!(
            read_archive(&bad)
                .unwrap_err()
                .to_string()
                .contains("missing index.json")
        );

        write_tar(&bad, &[("flat.json", b"{}")]);
        assert!(read_archive(&bad).unwrap_err().to_string().contains("outside"));

        let mut wrong: Index = serde_json::from_slice(&index).unwrap();
        wrong.format_version = 99;
        let wrong = serde_json::to_vec(&wrong).unwrap();
        write_tar(&bad, &[("top/index.json", &wrong)]);
        assert!(
            read_archive(&bad)
                .unwrap_err()
                .to_string()
                .contains("unsupported format")
        );
    }
}
