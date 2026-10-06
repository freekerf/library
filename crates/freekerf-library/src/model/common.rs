//! Types shared by materials and machine profiles.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Pattern for every identifier in the library (kebab-case, ASCII).
pub const ID_PATTERN: &str = r"^[a-z0-9]+(-[a-z0-9]+)*$";

/// Laser source technology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LaserKind {
    /// Semiconductor (blue/IR) diode module.
    Diode,
    /// CO₂ glass or RF tube.
    Co2,
    /// Fiber laser (usually galvo).
    Fiber,
}

impl LaserKind {
    /// Every variant, in declaration order.
    pub const ALL: [LaserKind; 3] = [LaserKind::Diode, LaserKind::Co2, LaserKind::Fiber];

    /// Stable identifier, identical to the serialized form.
    pub fn as_str(self) -> &'static str {
        match self {
            LaserKind::Diode => "diode",
            LaserKind::Co2 => "co2",
            LaserKind::Fiber => "fiber",
        }
    }
}

/// A laser source described by technology and real optical output power.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LaserSource {
    /// Laser technology.
    pub kind: LaserKind,
    /// Real optical output power, in watts (not the electrical/marketing figure).
    #[schemars(range(min = 0.1, max = 500.0), extend("x-unit" = "W"))]
    pub power_w: f64,
}
