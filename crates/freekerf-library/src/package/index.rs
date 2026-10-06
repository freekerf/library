//! The package index (`index.json`).

use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::{
    HazardList, LaserKind, LaserSource, Machine, Material, MaterialCategory, Operation, Severity,
};
use crate::safety::HazardMatcher;

/// Value of [`Index::format`].
pub const FORMAT: &str = "freekerf-library";
/// Value of [`Index::format_version`]; bumped on breaking index changes.
pub const FORMAT_VERSION: u32 = 1;

/// Table of contents of a release package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "FreeKerf library package index (v1)")]
pub struct Index {
    /// Always `freekerf-library`.
    pub format: String,
    /// Index format version.
    pub format_version: u32,
    /// Library release version (semver).
    pub version: String,
    /// Data schema version of the documents (`v1`).
    pub schema: String,
    /// SPDX license of the package.
    pub license: String,
    /// One entry per material, sorted by id.
    pub materials: Vec<MaterialEntry>,
    /// One entry per machine profile, sorted by id.
    pub machines: Vec<MachineEntry>,
    /// The hazard list file.
    pub hazards: FileEntry,
    /// Other files shipped (schemas, license texts).
    pub extra: Vec<FileEntry>,
}

/// A file inside the archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FileEntry {
    /// Path relative to the archive's top directory.
    pub path: String,
    /// Lowercase hex SHA-256 of the file.
    pub sha256: String,
}

/// Summary of a material, enough to list/filter without opening the file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MaterialEntry {
    /// Material id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Category.
    pub category: MaterialCategory,
    /// Thickness in millimetres, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thickness_mm: Option<f64>,
    /// Operations covered by its recipes.
    pub operations: Vec<Operation>,
    /// Laser technologies covered by its recipes.
    pub lasers: Vec<LaserKind>,
    /// Confidence levels present among its recipes.
    pub confidence: Vec<String>,
    /// Hazards matching the material (the app must alert or block).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hazards: Vec<HazardRef>,
    /// The material file (JSON).
    pub file: FileEntry,
}

/// Reference to a hazard list entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HazardRef {
    /// Hazard id.
    pub id: String,
    /// Severity.
    pub severity: Severity,
}

/// Summary of a machine profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MachineEntry {
    /// Machine id.
    pub id: String,
    /// Manufacturer.
    pub manufacturer: String,
    /// Model.
    pub model: String,
    /// Laser module.
    pub laser: LaserSource,
    /// The machine file (JSON).
    pub file: FileEntry,
}

impl MaterialEntry {
    /// Summarizes `material` stored at `file`.
    pub fn new(material: &Material, hazards: Option<&HazardList>, file: FileEntry) -> Self {
        let operations: BTreeSet<_> = material.recipes.iter().map(|r| r.operation).collect();
        let lasers: BTreeSet<_> = material.recipes.iter().map(|r| r.laser.kind).collect();
        let confidence: BTreeSet<_> = material.recipes.iter().map(|r| r.confidence.level()).collect();
        let hazards = hazards
            .map(|list| {
                HazardMatcher::new(list)
                    .matches(&material.name)
                    .into_iter()
                    .map(|h| HazardRef {
                        id: h.id.clone(),
                        severity: h.severity,
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self {
            id: material.id.clone(),
            name: material.name.clone(),
            category: material.category,
            thickness_mm: material.thickness_mm,
            operations: operations.into_iter().collect(),
            lasers: lasers.into_iter().collect(),
            confidence: confidence.into_iter().map(str::to_string).collect(),
            hazards,
            file,
        }
    }
}

impl MachineEntry {
    /// Summarizes `machine` stored at `file`.
    pub fn new(machine: &Machine, file: FileEntry) -> Self {
        Self {
            id: machine.id.clone(),
            manufacturer: machine.manufacturer.clone(),
            model: machine.model.clone(),
            laser: machine.laser.clone(),
            file,
        }
    }
}
