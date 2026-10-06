//! Hazardous material list (`data/safety/hazards.toml`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common::ID_PATTERN;

/// The hazard list document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "FreeKerf hazardous materials list (v1)")]
pub struct HazardList {
    /// Hazard entries.
    #[schemars(length(min = 1))]
    pub hazards: Vec<Hazard>,
}

/// A material (or family) that must never or should not be processed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Hazard {
    /// Unique identifier.
    #[schemars(pattern(ID_PATTERN))]
    pub id: String,
    /// Display name.
    #[schemars(length(min = 1))]
    pub name: String,
    /// What the app must do when the material is selected.
    pub severity: Severity,
    /// Lowercase keywords matched as whole words against material names (any language).
    #[schemars(length(min = 1))]
    pub keywords: Vec<String>,
    /// Keywords that cancel a match (e.g. `vegetable tanned` for leather).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unless: Vec<String>,
    /// Why it is dangerous.
    #[schemars(length(min = 1))]
    pub reason: String,
    /// Main emissions or effects (e.g. `hydrogen chloride`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emissions: Vec<String>,
}

/// Hazard severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Show a warning; processing is allowed with care.
    Warn,
    /// Block: never process. The library rejects materials matching it.
    Block,
}
