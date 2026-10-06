//! Material documents (`data/materials/**/*.toml`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common::{ID_PATTERN, LaserSource};

/// A material and its process recipes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "FreeKerf material (v1)")]
pub struct Material {
    /// Unique identifier; must match the file name (`<id>.toml`).
    #[schemars(pattern(ID_PATTERN))]
    pub id: String,
    /// Human readable name.
    #[schemars(length(min = 1, max = 120))]
    pub name: String,
    /// Material family.
    pub category: MaterialCategory,
    /// Nominal thickness in millimetres. Omit for surface-only materials (engraving).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 0.01, max = 100.0), extend("x-unit" = "mm"))]
    pub thickness_mm: Option<f64>,
    /// Optional supplier or product reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supplier: Option<String>,
    /// Free-form notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Process recipes for this material (at least one).
    #[schemars(length(min = 1))]
    pub recipes: Vec<Recipe>,
}

/// Material family, used for grouping and safety screening.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MaterialCategory {
    /// Solid wood (basswood, pine, bamboo…).
    Wood,
    /// Plywood, MDF, HDF.
    EngineeredWood,
    /// Cast or extruded PMMA.
    Acrylic,
    /// Leather and leather-like materials.
    Leather,
    /// Paper, card and cardboard.
    Paper,
    /// Cloth, felt, denim.
    Textile,
    /// Anodized aluminium.
    AnodizedMetal,
    /// Painted, powder coated or plated metal.
    CoatedMetal,
    /// Bare metal.
    Metal,
    /// Natural or artificial stone, slate.
    Stone,
    /// Glass and mirrors.
    Glass,
    /// Ceramics, tiles, clay.
    Ceramic,
    /// Other plastics.
    Plastic,
    /// Foams (EVA, foam board…).
    Foam,
    /// Rubber and silicone.
    Rubber,
    /// Cork.
    Cork,
    /// Anything else.
    Other,
}

impl MaterialCategory {
    /// Stable identifier, identical to the serialized form.
    pub fn as_str(self) -> &'static str {
        match self {
            MaterialCategory::Wood => "wood",
            MaterialCategory::EngineeredWood => "engineered_wood",
            MaterialCategory::Acrylic => "acrylic",
            MaterialCategory::Leather => "leather",
            MaterialCategory::Paper => "paper",
            MaterialCategory::Textile => "textile",
            MaterialCategory::AnodizedMetal => "anodized_metal",
            MaterialCategory::CoatedMetal => "coated_metal",
            MaterialCategory::Metal => "metal",
            MaterialCategory::Stone => "stone",
            MaterialCategory::Glass => "glass",
            MaterialCategory::Ceramic => "ceramic",
            MaterialCategory::Plastic => "plastic",
            MaterialCategory::Foam => "foam",
            MaterialCategory::Rubber => "rubber",
            MaterialCategory::Cork => "cork",
            MaterialCategory::Other => "other",
        }
    }
}

/// Process operation performed by a recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    /// Through cut.
    Cut,
    /// Vector (line) engraving / scoring.
    EngraveVector,
    /// Raster (image/fill) engraving.
    EngraveRaster,
}

impl Operation {
    /// Stable identifier, identical to the serialized form.
    pub fn as_str(self) -> &'static str {
        match self {
            Operation::Cut => "cut",
            Operation::EngraveVector => "engrave_vector",
            Operation::EngraveRaster => "engrave_raster",
        }
    }
}

/// Dithering algorithm recommended for raster engraving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Dithering {
    /// No dithering: pass the image through (vector-like fills).
    None,
    /// Simple black/white threshold.
    Threshold,
    /// Power modulated by grey level.
    Grayscale,
    /// Floyd–Steinberg error diffusion.
    FloydSteinberg,
    /// Jarvis–Judice–Ninke error diffusion.
    Jarvis,
    /// Stucki error diffusion.
    Stucki,
    /// Atkinson error diffusion.
    Atkinson,
    /// Ordered (Bayer) dithering.
    Ordered,
}

/// Parameters for one operation with one laser source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    /// Identifier, unique within the material.
    #[schemars(pattern(ID_PATTERN))]
    pub id: String,
    /// Operation performed.
    pub operation: Operation,
    /// Laser source the recipe was made for.
    pub laser: LaserSource,
    /// Optional machine profile id (`data/machines`) the recipe was made on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(pattern(ID_PATTERN))]
    pub machine: Option<String>,
    /// Feed rate in millimetres per minute.
    #[schemars(range(min = 1, max = 1_200_000), extend("x-unit" = "mm/min"))]
    pub speed_mm_min: u32,
    /// Minimum power, percentage of the source's maximum (S-min).
    #[schemars(range(min = 0.0, max = 100.0), extend("x-unit" = "%"))]
    pub power_min_pct: f64,
    /// Maximum power, percentage of the source's maximum (S-max).
    #[schemars(range(min = 0.0, max = 100.0), extend("x-unit" = "%"))]
    pub power_max_pct: f64,
    /// Number of passes.
    #[schemars(range(min = 1, max = 100))]
    pub passes: u32,
    /// Whether air assist is required (`true`), must be off (`false`) or does not matter (omitted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub air_assist: Option<bool>,
    /// Focus offset from the material surface in millimetres (negative = into the material).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = -50.0, max = 50.0), extend("x-unit" = "mm"))]
    pub focus_offset_mm: Option<f64>,
    /// Raster line interval in millimetres (raster engraving only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 0.005, max = 2.0), extend("x-unit" = "mm"))]
    pub line_interval_mm: Option<f64>,
    /// Raster resolution in dots per inch (raster engraving only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 10, max = 5080), extend("x-unit" = "dpi"))]
    pub dpi: Option<u32>,
    /// Recommended dithering (raster engraving only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dithering: Option<Dithering>,
    /// How much this recipe can be trusted.
    pub confidence: Confidence,
    /// Provenance when the data comes from a third party.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// Free-form notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Confidence level of a recipe. Apps must never present `estimated` as guaranteed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "level", rename_all = "snake_case", deny_unknown_fields)]
pub enum Confidence {
    /// Verified with a test grid; evidence and author are mandatory.
    Tested {
        /// Who ran the test.
        #[schemars(length(min = 1))]
        author: String,
        /// Photos of the test grid: repository paths (`evidence/...`) or https URLs.
        #[schemars(length(min = 1))]
        evidence: Vec<String>,
        /// Test date (ISO 8601, `YYYY-MM-DD`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(pattern(r"^\d{4}-\d{2}-\d{2}$"))]
        date: Option<String>,
    },
    /// Reported by the community without evidence.
    Community {
        /// Who reported it, when known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reported_by: Option<String>,
    },
    /// Derived (e.g. by power scaling) and not verified.
    Estimated {
        /// How the values were obtained.
        #[schemars(length(min = 1))]
        method: String,
        /// Recipe the values were derived from (`<material-id>/<recipe-id>`), when any.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        derived_from: Option<String>,
    },
}

impl Confidence {
    /// Serialized level name.
    pub fn level(&self) -> &'static str {
        match self {
            Confidence::Tested { .. } => "tested",
            Confidence::Community { .. } => "community",
            Confidence::Estimated { .. } => "estimated",
        }
    }
}

/// Provenance of third-party data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// Name of the origin (e.g. `LaserGRBL StandardMaterials.psh`).
    #[schemars(length(min = 1))]
    pub name: String,
    /// URL of the origin, pinned to a revision when possible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// SPDX license expression of the origin (must be GPL-3.0 compatible).
    #[schemars(length(min = 1))]
    pub license: String,
    /// Record reference inside the origin (row id, model…).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}
