//! Machine profile documents (`data/machines/**/*.toml`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common::{ID_PATTERN, LaserSource};

/// A machine profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "FreeKerf machine profile (v1)")]
pub struct Machine {
    /// Unique identifier; must match the file name (`<id>.toml`).
    #[schemars(pattern(ID_PATTERN))]
    pub id: String,
    /// Manufacturer (use `Generic` for reference profiles).
    #[schemars(length(min = 1))]
    pub manufacturer: String,
    /// Model name.
    #[schemars(length(min = 1))]
    pub model: String,
    /// Usable work area.
    pub work_area: WorkArea,
    /// Controller firmware.
    pub firmware: Firmware,
    /// Machine origin (home corner).
    pub origin: Origin,
    /// Optional hardware features.
    pub features: Features,
    /// Installed laser module, with its real optical power.
    pub laser: LaserSource,
    /// Electrical/marketing power of the module in watts, when different from the optical one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 0.1, max = 2000.0), extend("x-unit" = "W"))]
    pub electrical_power_w: Option<f64>,
    /// Maximum feed rate in millimetres per minute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 1, max = 1_200_000), extend("x-unit" = "mm/min"))]
    pub max_speed_mm_min: Option<u32>,
    /// Recommended Grbl/grblHAL `$$` settings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub settings: Vec<FirmwareSetting>,
    /// Free-form notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Usable work area in millimetres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkArea {
    /// X travel in millimetres.
    #[schemars(range(min = 10.0, max = 5000.0), extend("x-unit" = "mm"))]
    pub x_mm: f64,
    /// Y travel in millimetres.
    #[schemars(range(min = 10.0, max = 5000.0), extend("x-unit" = "mm"))]
    pub y_mm: f64,
    /// Z travel in millimetres, when the machine has a Z axis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 1.0, max = 1000.0), extend("x-unit" = "mm"))]
    pub z_mm: Option<f64>,
}

/// Controller firmware.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Firmware {
    /// Firmware family.
    pub kind: FirmwareKind,
    /// Minimum firmware version the profile was written for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,
}

/// Firmware family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FirmwareKind {
    /// Grbl 1.1.
    Grbl,
    /// grblHAL.
    GrblHal,
    /// Smoothieware.
    Smoothieware,
    /// Marlin.
    Marlin,
}

impl FirmwareKind {
    /// Whether the firmware uses Grbl-style `$<n>=<value>` settings.
    pub fn uses_dollar_settings(self) -> bool {
        matches!(self, FirmwareKind::Grbl | FirmwareKind::GrblHal)
    }
}

/// Machine origin corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    /// Front left (Grbl default).
    FrontLeft,
    /// Front right.
    FrontRight,
    /// Rear left.
    RearLeft,
    /// Rear right.
    RearRight,
    /// Centre (typical for galvo heads).
    Center,
}

/// Optional hardware features.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Features {
    /// Air assist available.
    pub air_assist: bool,
    /// Rotary attachment supported.
    pub rotary: bool,
    /// Camera available.
    pub camera: bool,
    /// Motorised Z axis.
    pub z_axis: bool,
}

/// A recommended firmware setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FirmwareSetting {
    /// Setting key, e.g. `$32`.
    #[schemars(pattern(r"^\$[0-9]+$"))]
    pub key: String,
    /// Value, kept as text to preserve the exact format.
    #[schemars(length(min = 1))]
    pub value: String,
    /// What the setting does / why this value.
    #[schemars(length(min = 1))]
    pub description: String,
}
