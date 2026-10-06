//! Parser for LaserGRBL `.psh` files (DataSet XML).

use roxmltree::{Document, Node};

use crate::import::ImportError;

/// One row of the `Materials` table. Missing fields take the XSD defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PshRow {
    /// Row GUID.
    pub id: String,
    /// Whether the row is visible in LaserGRBL.
    pub visible: bool,
    /// Machine model (free text, often with the laser power).
    pub model: String,
    /// Material name (free text).
    pub material: String,
    /// Thickness (free text: `3mm`, `0,5`, `-`…).
    pub thickness: String,
    /// `Cut` or `Engrave`.
    pub action: String,
    /// Power, percent.
    pub power: u32,
    /// Speed, mm/min.
    pub speed: u32,
    /// Passes.
    pub cycles: u32,
    /// Remarks.
    pub remarks: Option<String>,
}

fn child<'a>(row: Node<'a, '_>, name: &str) -> Option<&'a str> {
    row.children()
        .find(|c| c.is_element() && c.tag_name().name() == name)
        .map(|c| c.text().unwrap_or(""))
}

fn number(row: Node<'_, '_>, name: &str, default: u32) -> Result<u32, ImportError> {
    match child(row, name) {
        None => Ok(default),
        Some(text) => text
            .trim()
            .parse()
            .map_err(|_| ImportError::Parse(format!("invalid {name} `{text}`"))),
    }
}

/// Parses every `Materials` row of a `.psh` document.
pub fn parse_psh(input: &str) -> Result<Vec<PshRow>, ImportError> {
    let input = input.trim_start_matches('\u{feff}');
    let doc = Document::parse(input).map_err(|e| ImportError::Parse(e.to_string()))?;
    let root = doc.root_element();
    if root.tag_name().name() != "MaterialDB" {
        return Err(ImportError::Parse(format!(
            "expected <MaterialDB>, found <{}>",
            root.tag_name().name()
        )));
    }
    root.children()
        .filter(|n| n.is_element() && n.tag_name().name() == "Materials")
        .map(|row| {
            Ok(PshRow {
                id: child(row, "id").unwrap_or("").trim().to_string(),
                visible: child(row, "Visible").is_none_or(|v| v.trim() != "false"),
                model: child(row, "Model").unwrap_or("Generic Laser").trim().to_string(),
                material: child(row, "Material")
                    .unwrap_or("Generic Material")
                    .trim()
                    .to_string(),
                thickness: child(row, "Thickness").unwrap_or("").to_string(),
                action: child(row, "Action").unwrap_or("").trim().to_string(),
                power: number(row, "Power", 100)?,
                speed: number(row, "Speed", 1000)?,
                cycles: number(row, "Cycles", 1)?,
                remarks: child(row, "Remarks").map(str::to_string),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_bom() {
        let rows =
            parse_psh("\u{feff}<MaterialDB><Materials><id>x</id></Materials><Other/></MaterialDB>").unwrap();
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert!(r.visible);
        assert_eq!(r.model, "Generic Laser");
        assert_eq!(r.material, "Generic Material");
        assert_eq!((r.power, r.speed, r.cycles), (100, 1000, 1));
        assert_eq!(r.remarks, None);
    }

    #[test]
    fn errors() {
        assert!(
            parse_psh("<Other/>")
                .unwrap_err()
                .to_string()
                .contains("expected <MaterialDB>")
        );
        let err =
            parse_psh("<MaterialDB><Materials><Power>lots</Power></Materials></MaterialDB>").unwrap_err();
        assert!(err.to_string().contains("invalid Power"));
    }
}
