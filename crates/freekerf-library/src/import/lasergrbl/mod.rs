//! Importer for the LaserGRBL material database (`StandardMaterials.psh`,
//! an ADO.NET DataSet serialized as XML; LaserGRBL is GPL-3.0).
//!
//! Each row (model × material × thickness × action) becomes a recipe with
//! `community` confidence; rows are grouped into one material per
//! (name, thickness). Rows matching a blocked hazard or whose laser power
//! cannot be resolved are skipped and listed in the report.

mod classify;
mod psh;
mod resolve;

pub use classify::{Categorizer, KeywordCategorizer};
pub use psh::{PshRow, parse_psh};
pub use resolve::{LaserResolver, ModelOverrides, ModelResolver};

use std::collections::BTreeMap;

use super::{ImportError, ImportOutcome, ImportReport, Importer, Skipped};
use crate::model::{Confidence, HazardList, Material, Operation, Recipe, Severity, Source};
use crate::safety::HazardMatcher;
use crate::text::slugify;

/// Name used in `source.name`.
pub const SOURCE_NAME: &str = "LaserGRBL StandardMaterials.psh";
/// Default upstream URL (pinned commit used for the committed import).
pub const DEFAULT_SOURCE_URL: &str = "https://github.com/arkypita/LaserGRBL/blob/1f9337b3af27133f8b1696e41cc110f2af74d04f/LaserGRBL/StandardMaterials.psh";
/// Prefix of every generated material id.
pub const ID_PREFIX: &str = "lasergrbl-";

/// LaserGRBL importer. Collaborators are injected (Dependency Inversion).
pub struct LaserGrblImporter<'a> {
    resolver: &'a dyn LaserResolver,
    categorizer: &'a dyn Categorizer,
    hazards: &'a HazardList,
    source_url: String,
}

impl<'a> LaserGrblImporter<'a> {
    /// New importer.
    pub fn new(
        resolver: &'a dyn LaserResolver,
        categorizer: &'a dyn Categorizer,
        hazards: &'a HazardList,
        source_url: impl Into<String>,
    ) -> Self {
        Self {
            resolver,
            categorizer,
            hazards,
            source_url: source_url.into(),
        }
    }
}

/// Parsed thickness: value in mm, or the original text when not parseable.
fn parse_thickness(raw: &str) -> Result<Option<f64>, String> {
    let t = raw.trim().to_ascii_lowercase().replace(',', ".");
    let t = t.trim_end_matches("mm").trim();
    if t.is_empty() || t == "-" {
        return Ok(None);
    }
    match t.parse::<f64>() {
        Ok(v) if v > 0.0 && v <= 100.0 => Ok(Some(v)),
        _ => Err(raw.trim().to_string()),
    }
}

fn clean_name(raw: &str) -> String {
    raw.replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn clean_notes(raw: &str) -> Option<String> {
    let text = raw.replace("\r\n", "\n").replace(['\r', '\t'], " ");
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

fn thickness_suffix(t: f64) -> String {
    format!("-{}mm", t.to_string().replace('.', "-"))
}

impl Importer for LaserGrblImporter<'_> {
    fn import(&self, input: &str) -> Result<ImportOutcome, ImportError> {
        let rows = parse_psh(input)?;
        let matcher = HazardMatcher::new(self.hazards);
        let mut report = ImportReport {
            rows: rows.len(),
            ..ImportReport::default()
        };
        let mut groups: BTreeMap<String, Material> = BTreeMap::new();

        for row in rows.iter().filter(|r| r.visible) {
            let name = clean_name(&row.material);
            let reference = format!("{} — {} / {} / {}", row.id, row.model, name, row.action);
            let mut skip = |reason: String| {
                report.skipped.push(Skipped {
                    reference: reference.clone(),
                    reason,
                })
            };
            if let Some(h) = matcher.worst(&name).filter(|h| h.severity == Severity::Block) {
                skip(format!("material perigoso bloqueado (`{}`): {}", h.id, h.reason));
                continue;
            }
            let Some(laser) = self.resolver.resolve(&row.model) else {
                skip("potência do laser do modelo desconhecida (adicione em models.toml)".into());
                continue;
            };
            let operation = match row.action.trim().to_ascii_lowercase().as_str() {
                "cut" => Operation::Cut,
                "engrave" => Operation::EngraveRaster,
                other => {
                    skip(format!("ação desconhecida `{other}`"));
                    continue;
                }
            };
            if row.speed == 0 || row.power == 0 || row.power > 100 || row.cycles == 0 {
                skip("parâmetros fora de faixa (velocidade, potência ou ciclos)".into());
                continue;
            }
            let (thickness, raw_thickness) = match parse_thickness(&row.thickness) {
                Ok(t) => (t, None),
                Err(raw) => (None, Some(raw)),
            };
            let id = format!(
                "{ID_PREFIX}{}{}",
                slugify(&name),
                thickness.map(thickness_suffix).unwrap_or_default()
            );
            let material = groups.entry(id.clone()).or_insert_with(|| Material {
                id,
                name: name.clone(),
                category: self.categorizer.categorize(&name),
                thickness_mm: thickness,
                supplier: None,
                notes: None,
                recipes: Vec::new(),
            });
            let op_slug = if operation == Operation::Cut {
                "cut"
            } else {
                "engrave"
            };
            let recipe_id = format!("{}-{op_slug}", slugify(&row.model));
            if material.recipes.iter().any(|r| r.id == recipe_id) {
                skip(format!(
                    "duplicada após normalização (receita `{recipe_id}` já existe)"
                ));
                continue;
            }
            let power = f64::from(row.power);
            let mut notes = row.remarks.as_deref().and_then(clean_notes);
            if let Some(raw) = raw_thickness {
                let extra = format!("Espessura original: {raw}");
                notes = Some(notes.map_or(extra.clone(), |n| format!("{n}\n{extra}")));
            }
            material.recipes.push(Recipe {
                id: recipe_id,
                operation,
                laser,
                machine: None,
                speed_mm_min: row.speed,
                power_min_pct: if operation == Operation::Cut { power } else { 0.0 },
                power_max_pct: power,
                passes: row.cycles,
                air_assist: None,
                focus_offset_mm: None,
                line_interval_mm: None,
                dpi: None,
                dithering: None,
                confidence: Confidence::Community { reported_by: None },
                source: Some(Source {
                    name: SOURCE_NAME.into(),
                    url: Some(self.source_url.clone()),
                    license: "GPL-3.0-or-later".into(),
                    reference: Some(format!("{} ({})", row.id, row.model)),
                }),
                notes,
            });
            report.imported += 1;
        }

        let materials = groups
            .into_values()
            .map(|mut m| {
                m.recipes.sort_by(|a, b| a.id.cmp(&b.id));
                m
            })
            .collect();
        Ok(ImportOutcome { materials, report })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Hazard, LaserKind, MaterialCategory};

    const PSH: &str = r#"<?xml version="1.0" standalone="yes"?>
<MaterialDB xmlns="http://tempuri.org/MaterialDB.xsd">
  <Materials><id>1</id><Visible>true</Visible><Model>Acme (5.5W)</Model><Material>Kraft_Paper</Material><Thickness>-</Thickness><Action>Engrave</Action><Power>40</Power><Speed>3000</Speed><Cycles>1</Cycles><Remarks>  Low	power.  </Remarks></Materials>
  <Materials><id>2</id><Visible>true</Visible><Model>Acme (5.5W)</Model><Material>Basswood</Material><Thickness>3,0mm</Thickness><Action>Cut</Action><Power>100</Power><Speed>200</Speed><Cycles>3</Cycles></Materials>
  <Materials><id>3</id><Visible>true</Visible><Model>Acme (5.5W)</Model><Material>basswood</Material><Thickness>3mm</Thickness><Action>Cut</Action><Power>100</Power><Speed>250</Speed><Cycles>3</Cycles></Materials>
  <Materials><id>4</id><Visible>true</Visible><Model>Mystery</Model><Material>Basswood</Material><Thickness>3</Thickness><Action>Cut</Action><Power>100</Power><Speed>250</Speed><Cycles>3</Cycles></Materials>
  <Materials><id>5</id><Visible>true</Visible><Model>Acme (5.5W)</Model><Material>Poly_Vinyl_Chloride</Material><Thickness>-</Thickness><Action>Engrave</Action><Power>40</Power><Speed>3000</Speed><Cycles>1</Cycles></Materials>
  <Materials><id>6</id><Visible>true</Visible><Model>Acme (5.5W)</Model><Material>Paper</Material><Thickness>250 g/mq</Thickness><Action>Cut</Action><Power>80</Power><Speed>900</Speed><Cycles>1</Cycles><Remarks>Thin</Remarks></Materials>
  <Materials><id>7</id><Visible>true</Visible><Model>Acme (5.5W)</Model><Material>Paper</Material><Thickness>-</Thickness><Action>Fold</Action><Power>80</Power><Speed>900</Speed><Cycles>1</Cycles></Materials>
  <Materials><id>8</id><Visible>true</Visible><Model>Acme (5.5W)</Model><Material>Paper</Material><Thickness>-</Thickness><Action>Cut</Action><Power>0</Power><Speed>900</Speed><Cycles>1</Cycles></Materials>
  <Materials><id>9</id><Visible>false</Visible><Model>Acme (5.5W)</Model><Material>Hidden</Material><Thickness>-</Thickness><Action>Cut</Action><Power>10</Power><Speed>900</Speed><Cycles>1</Cycles></Materials>
</MaterialDB>"#;

    fn hazards() -> HazardList {
        HazardList {
            hazards: vec![Hazard {
                id: "pvc".into(),
                name: "PVC".into(),
                severity: Severity::Block,
                keywords: vec!["vinyl".into()],
                unless: vec![],
                reason: "HCl".into(),
                emissions: vec![],
            }],
        }
    }

    #[test]
    fn imports_rows() {
        let resolver = ModelResolver::default();
        let categorizer = KeywordCategorizer;
        let hazards = hazards();
        let importer = LaserGrblImporter::new(&resolver, &categorizer, &hazards, DEFAULT_SOURCE_URL);
        let out = importer.import(PSH).unwrap();
        assert_eq!(out.report.rows, 9);
        assert_eq!(out.report.imported, 3);
        let reasons: Vec<_> = out.report.skipped.iter().map(|s| s.reason.as_str()).collect();
        assert_eq!(reasons.len(), 5, "{reasons:?}");
        assert!(reasons[0].contains("duplicada"));
        assert!(reasons[1].contains("desconhecida"));
        assert!(reasons[2].contains("bloqueado"));
        assert!(reasons[3].contains("ação desconhecida"));
        assert!(reasons[4].contains("fora de faixa"));

        let ids: Vec<_> = out.materials.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "lasergrbl-basswood-3mm",
                "lasergrbl-kraft-paper",
                "lasergrbl-paper"
            ]
        );
        let wood = &out.materials[0];
        assert_eq!(wood.category, MaterialCategory::Wood);
        assert_eq!(wood.thickness_mm, Some(3.0));
        let cut = &wood.recipes[0];
        assert_eq!(cut.id, "acme-5-5w-cut");
        assert_eq!(cut.laser.kind, LaserKind::Diode);
        assert_eq!(cut.laser.power_w, 5.5);
        assert_eq!(
            (cut.power_min_pct, cut.power_max_pct, cut.passes),
            (100.0, 100.0, 3)
        );
        assert_eq!(cut.source.as_ref().unwrap().license, "GPL-3.0-or-later");

        let kraft = &out.materials[1];
        assert_eq!(kraft.name, "Kraft Paper");
        assert_eq!(kraft.category, MaterialCategory::Paper);
        let engrave = &kraft.recipes[0];
        assert_eq!(engrave.operation, Operation::EngraveRaster);
        assert_eq!((engrave.power_min_pct, engrave.power_max_pct), (0.0, 40.0));
        assert_eq!(engrave.notes.as_deref(), Some("Low power."));

        let paper = &out.materials[2];
        assert_eq!(paper.thickness_mm, None);
        assert_eq!(
            paper.recipes[0].notes.as_deref(),
            Some("Thin\nEspessura original: 250 g/mq")
        );
    }

    #[test]
    fn thickness_parsing() {
        assert_eq!(parse_thickness(" 2 mm"), Ok(Some(2.0)));
        assert_eq!(parse_thickness("0,5"), Ok(Some(0.5)));
        assert_eq!(parse_thickness(""), Ok(None));
        assert_eq!(parse_thickness("-"), Ok(None));
        assert_eq!(parse_thickness("0"), Err("0".into()));
        assert_eq!(thickness_suffix(0.5), "-0-5mm");
        assert_eq!(thickness_suffix(12.0), "-12mm");
    }

    #[test]
    fn rejects_bad_xml() {
        let resolver = ModelResolver::default();
        let hazards = hazards();
        let importer = LaserGrblImporter::new(&resolver, &KeywordCategorizer, &hazards, "x");
        assert!(importer.import("<nope").is_err());
    }
}
