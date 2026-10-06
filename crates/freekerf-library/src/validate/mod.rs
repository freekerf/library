//! Semantic validation rules applied to a loaded [`Library`].
//!
//! Each rule has one responsibility and implements [`Rule`]; the
//! [`Validator`] composes them. New checks are new `Rule` types — the
//! validator never needs to change (Open/Closed).

mod confidence;
mod hazards;
mod machines;
mod process;
mod references;
mod sources;
mod unique;

pub use confidence::ConfidenceEvidence;
pub use hazards::HazardScreen;
pub use machines::MachineSettings;
pub use process::{PlausibleProcess, PowerRange, RasterParams};
pub use references::MachineReferences;
pub use sources::SourceLicense;
pub use unique::UniqueIds;

use crate::diagnostics::Diagnostics;
use crate::layout::Layout;
use crate::loader::Library;

/// Everything a rule may look at.
pub struct Context<'a> {
    /// Loaded documents.
    pub library: &'a Library,
    /// Checkout layout (for evidence files).
    pub layout: &'a Layout,
}

/// A semantic validation rule.
pub trait Rule {
    /// Stable rule name, shown in diagnostics.
    fn name(&self) -> &'static str;
    /// Records findings into `out`.
    fn check(&self, ctx: &Context<'_>, out: &mut Diagnostics);
}

/// An ordered set of rules.
#[derive(Default)]
pub struct Validator {
    rules: Vec<Box<dyn Rule>>,
}

impl Validator {
    /// Validator without rules.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a rule.
    pub fn with_rule(mut self, rule: impl Rule + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    /// Every rule shipped with the library.
    pub fn standard() -> Self {
        Self::new()
            .with_rule(UniqueIds)
            .with_rule(PowerRange)
            .with_rule(PlausibleProcess)
            .with_rule(RasterParams)
            .with_rule(MachineReferences)
            .with_rule(MachineSettings)
            .with_rule(ConfidenceEvidence)
            .with_rule(SourceLicense)
            .with_rule(HazardScreen)
    }

    /// Names of the configured rules.
    pub fn rule_names(&self) -> Vec<&'static str> {
        self.rules.iter().map(|r| r.name()).collect()
    }

    /// Runs every rule.
    pub fn run(&self, ctx: &Context<'_>) -> Diagnostics {
        let mut out = Diagnostics::new();
        for rule in &self.rules {
            rule.check(ctx, &mut out);
        }
        out
    }
}

/// Iterates `(entry path, material, recipe)` over the whole library.
pub(crate) fn recipes(
    library: &Library,
) -> impl Iterator<Item = (&std::path::Path, &crate::model::Material, &crate::model::Recipe)> {
    library
        .materials
        .iter()
        .flat_map(|e| e.doc.recipes.iter().map(move |r| (e.path.as_path(), &e.doc, r)))
}

#[cfg(test)]
pub(crate) mod fixtures {
    //! Builders shared by rule tests.

    use std::path::PathBuf;

    use crate::loader::{Entry, Library};
    use crate::model::*;

    pub fn recipe(id: &str) -> Recipe {
        Recipe {
            id: id.into(),
            operation: Operation::Cut,
            laser: LaserSource {
                kind: LaserKind::Diode,
                power_w: 10.0,
            },
            machine: None,
            speed_mm_min: 300,
            power_min_pct: 100.0,
            power_max_pct: 100.0,
            passes: 2,
            air_assist: Some(true),
            focus_offset_mm: None,
            line_interval_mm: None,
            dpi: None,
            dithering: None,
            confidence: Confidence::Community { reported_by: None },
            source: None,
            notes: None,
        }
    }

    pub fn material(id: &str, recipes: Vec<Recipe>) -> Entry<Material> {
        Entry {
            path: PathBuf::from(format!("data/materials/{id}.toml")),
            doc: Material {
                id: id.into(),
                name: id.into(),
                category: MaterialCategory::Wood,
                thickness_mm: Some(3.0),
                supplier: None,
                notes: None,
                recipes,
            },
        }
    }

    pub fn machine(id: &str) -> Entry<Machine> {
        Entry {
            path: PathBuf::from(format!("data/machines/{id}.toml")),
            doc: Machine {
                id: id.into(),
                manufacturer: "Generic".into(),
                model: id.into(),
                work_area: WorkArea {
                    x_mm: 400.0,
                    y_mm: 400.0,
                    z_mm: None,
                },
                firmware: Firmware {
                    kind: FirmwareKind::Grbl,
                    min_version: None,
                },
                origin: Origin::FrontLeft,
                features: Features {
                    air_assist: true,
                    rotary: false,
                    camera: false,
                    z_axis: false,
                },
                laser: LaserSource {
                    kind: LaserKind::Diode,
                    power_w: 10.0,
                },
                electrical_power_w: None,
                max_speed_mm_min: None,
                settings: vec![FirmwareSetting {
                    key: "$32".into(),
                    value: "1".into(),
                    description: "laser mode".into(),
                }],
                notes: None,
            },
        }
    }

    pub fn hazards() -> Entry<HazardList> {
        Entry {
            path: PathBuf::from("data/safety/hazards.toml"),
            doc: HazardList {
                hazards: vec![
                    Hazard {
                        id: "pvc".into(),
                        name: "PVC".into(),
                        severity: Severity::Block,
                        keywords: vec!["pvc".into()],
                        unless: vec![],
                        reason: "HCl".into(),
                        emissions: vec![],
                    },
                    Hazard {
                        id: "abs".into(),
                        name: "ABS".into(),
                        severity: Severity::Warn,
                        keywords: vec!["abs".into()],
                        unless: vec![],
                        reason: "HCN".into(),
                        emissions: vec![],
                    },
                ],
            },
        }
    }

    pub fn library(materials: Vec<Entry<Material>>, machines: Vec<Entry<Machine>>) -> Library {
        Library {
            materials,
            machines,
            hazards: Some(hazards()),
        }
    }

    /// Runs `rule` on `library`, returning `(errors, warnings)` messages.
    pub fn run(rule: impl super::Rule, library: &Library) -> (Vec<String>, Vec<String>) {
        let layout = crate::layout::Layout::new(env!("CARGO_MANIFEST_DIR"));
        let mut out = crate::diagnostics::Diagnostics::new();
        rule.check(
            &super::Context {
                library,
                layout: &layout,
            },
            &mut out,
        );
        let pick = |level| {
            out.iter()
                .filter(|d| d.level == level)
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
        };
        (
            pick(crate::diagnostics::Level::Error),
            pick(crate::diagnostics::Level::Warning),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_has_all_rules() {
        let v = Validator::standard();
        assert_eq!(
            v.rule_names(),
            [
                "unique-ids",
                "power-range",
                "plausible-process",
                "raster-params",
                "machine-references",
                "machine-settings",
                "confidence-evidence",
                "source-license",
                "hazards"
            ]
        );
        let lib = fixtures::library(
            vec![fixtures::material("a", vec![fixtures::recipe("r")])],
            vec![fixtures::machine("m")],
        );
        let layout = Layout::new(".");
        let out = v.run(&Context {
            library: &lib,
            layout: &layout,
        });
        assert!(out.is_empty(), "{out:?}");
        assert!(Validator::new().rule_names().is_empty());
    }
}
