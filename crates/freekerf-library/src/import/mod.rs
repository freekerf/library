//! Importers that convert third-party libraries into FreeKerf materials.

pub mod lasergrbl;
mod writer;

pub use writer::{remove_toml_files, write_materials};

use thiserror::Error;

use crate::model::Material;

/// Import failure.
#[derive(Debug, Error)]
pub enum ImportError {
    /// The input could not be parsed.
    #[error("cannot parse input: {0}")]
    Parse(String),
}

/// A row that was not imported, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    /// Row reference in the origin (id, model, material…).
    pub reference: String,
    /// Reason.
    pub reason: String,
}

/// Statistics of an import run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportReport {
    /// Rows read from the input.
    pub rows: usize,
    /// Rows converted into recipes.
    pub imported: usize,
    /// Rows left out.
    pub skipped: Vec<Skipped>,
}

impl ImportReport {
    /// Markdown summary (committed next to imported data for review).
    pub fn to_markdown(&self, title: &str, materials: usize) -> String {
        let mut out = format!(
            "# {title}\n\nGerado pelo importador; não edite à mão.\n\n- Linhas lidas: {}\n- Receitas importadas: {}\n- Materiais gerados: {materials}\n- Linhas ignoradas: {}\n",
            self.rows,
            self.imported,
            self.skipped.len()
        );
        if !self.skipped.is_empty() {
            out.push_str("\n## Linhas ignoradas\n\n| Referência | Motivo |\n| --- | --- |\n");
            for s in &self.skipped {
                out.push_str(&format!(
                    "| {} | {} |\n",
                    s.reference.replace('|', "\\|"),
                    s.reason.replace('|', "\\|")
                ));
            }
        }
        out
    }
}

/// Result of an import.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportOutcome {
    /// Generated materials, sorted by id.
    pub materials: Vec<Material>,
    /// What happened.
    pub report: ImportReport,
}

/// Converts a third-party library (as text) into materials.
pub trait Importer {
    /// Runs the import.
    fn import(&self, input: &str) -> Result<ImportOutcome, ImportError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_report() {
        let report = ImportReport {
            rows: 3,
            imported: 2,
            skipped: vec![Skipped {
                reference: "a|b".into(),
                reason: "why".into(),
            }],
        };
        let md = report.to_markdown("Import", 1);
        assert!(md.starts_with("# Import\n"));
        assert!(md.contains("- Linhas lidas: 3"));
        assert!(md.contains("| a\\|b | why |"));
        let empty = ImportReport::default().to_markdown("X", 0);
        assert!(!empty.contains("Linhas ignoradas\n\n|"));
    }
}
