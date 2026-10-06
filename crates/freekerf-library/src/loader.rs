//! Loads a library checkout into typed documents.
//!
//! Every file goes through a pipeline of [`DocumentCheck`]s on its raw form
//! (text lint, JSON Schema) before being deserialized. A file that fails a
//! check with an error is reported and left out of the [`Library`].

use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde_json::Value;
use walkdir::WalkDir;

use crate::diagnostics::Diagnostics;
use crate::layout::Layout;
use crate::model::{HazardList, Machine, Material};
use crate::schema::{SchemaKind, SchemaSet};

/// A typed document and the file it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry<T> {
    /// Path relative to the library root.
    pub path: PathBuf,
    /// Parsed document.
    pub doc: T,
}

/// Every document of a library checkout.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Library {
    /// Materials, sorted by path.
    pub materials: Vec<Entry<Material>>,
    /// Machine profiles, sorted by path.
    pub machines: Vec<Entry<Machine>>,
    /// The hazard list, when present and valid.
    pub hazards: Option<Entry<HazardList>>,
}

impl Library {
    /// Finds a material by id.
    pub fn material(&self, id: &str) -> Option<&Material> {
        self.materials.iter().map(|e| &e.doc).find(|m| m.id == id)
    }

    /// Finds a machine by id.
    pub fn machine(&self, id: &str) -> Option<&Machine> {
        self.machines.iter().map(|e| &e.doc).find(|m| m.id == id)
    }
}

/// A file as read from disk, before deserialization.
pub struct RawDocument<'a> {
    /// Path relative to the library root.
    pub path: &'a Path,
    /// Document kind.
    pub kind: SchemaKind,
    /// File content.
    pub text: &'a str,
    /// Content converted to JSON.
    pub json: &'a Value,
}

/// A check on a raw document (Open/Closed: add checks without touching the loader).
pub trait DocumentCheck {
    /// Records findings for `doc` into `out`.
    fn check(&self, doc: &RawDocument<'_>, out: &mut Diagnostics);
}

/// Formatting lint: LF line endings, no tabs, single trailing newline, file name = id.
pub struct TextLint;

impl DocumentCheck for TextLint {
    fn check(&self, doc: &RawDocument<'_>, out: &mut Diagnostics) {
        const RULE: &str = "toml-lint";
        if doc.text.contains('\r') {
            out.error(RULE, "use LF line endings").file(doc.path);
        }
        if doc.text.contains('\t') {
            out.error(RULE, "use spaces, not tabs").file(doc.path);
        }
        if !doc.text.ends_with('\n') || doc.text.ends_with("\n\n") {
            out.error(RULE, "file must end with exactly one newline")
                .file(doc.path);
        }
        if doc.kind != SchemaKind::Hazards {
            let stem = doc.path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
            if let Some(id) = doc.json.get("id").and_then(Value::as_str) {
                if id != stem {
                    out.error(RULE, format!("file name must be `{id}.toml`"))
                        .file(doc.path);
                }
            }
        }
    }
}

/// Validates the document against the published JSON Schema.
pub struct SchemaCheck {
    schemas: SchemaSet,
}

impl SchemaCheck {
    /// Compiles the schemas.
    pub fn new() -> Self {
        Self {
            schemas: SchemaSet::compile(),
        }
    }
}

impl Default for SchemaCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentCheck for SchemaCheck {
    fn check(&self, doc: &RawDocument<'_>, out: &mut Diagnostics) {
        for message in self.schemas.validate(doc.kind, doc.json) {
            out.error("schema", message).file(doc.path);
        }
    }
}

/// Loads documents from a [`Layout`], applying the configured checks.
pub struct Loader {
    checks: Vec<Box<dyn DocumentCheck>>,
}

impl Default for Loader {
    /// Loader with the standard checks ([`TextLint`], [`SchemaCheck`]).
    fn default() -> Self {
        Self::new(vec![Box::new(TextLint), Box::new(SchemaCheck::new())])
    }
}

impl Loader {
    /// Loader with custom checks.
    pub fn new(checks: Vec<Box<dyn DocumentCheck>>) -> Self {
        Self { checks }
    }

    /// Loads every document under `layout`.
    pub fn load(&self, layout: &Layout) -> (Library, Diagnostics) {
        let mut out = Diagnostics::new();
        let mut library = Library {
            materials: self.load_dir(layout, &layout.materials_dir(), SchemaKind::Material, &mut out),
            machines: self.load_dir(layout, &layout.machines_dir(), SchemaKind::Machine, &mut out),
            hazards: None,
        };
        let hazards = layout.hazards_file();
        if hazards.is_file() {
            library.hazards = self.load_file(layout, &hazards, SchemaKind::Hazards, &mut out);
        } else {
            out.error("missing-file", "hazard list not found")
                .file(layout.relative(&hazards));
        }
        (library, out)
    }

    fn load_dir<T: DeserializeOwned>(
        &self,
        layout: &Layout,
        dir: &Path,
        kind: SchemaKind,
        out: &mut Diagnostics,
    ) -> Vec<Entry<T>> {
        let mut files: Vec<PathBuf> = WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        files.sort();
        files
            .iter()
            .filter_map(|path| self.load_file(layout, path, kind, out))
            .collect()
    }

    fn load_file<T: DeserializeOwned>(
        &self,
        layout: &Layout,
        path: &Path,
        kind: SchemaKind,
        out: &mut Diagnostics,
    ) -> Option<Entry<T>> {
        let rel = layout.relative(path).to_path_buf();
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) => {
                out.error("io", e.to_string()).file(&rel);
                return None;
            }
        };
        let table: toml::Table = match toml::from_str(&text) {
            Ok(table) => table,
            Err(e) => {
                out.error("toml", e.message().to_string()).file(&rel);
                return None;
            }
        };
        let json = serde_json::to_value(&table).expect("TOML maps to JSON");
        let raw = RawDocument {
            path: &rel,
            kind,
            text: &text,
            json: &json,
        };
        let mut local = Diagnostics::new();
        for check in &self.checks {
            check.check(&raw, &mut local);
        }
        let failed = local.has_errors();
        out.extend(local);
        if failed {
            return None;
        }
        match T::deserialize(toml::Value::Table(table)) {
            Ok(doc) => Some(Entry { path: rel, doc }),
            Err(e) => {
                out.error("model", e.to_string()).file(&rel);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Level;

    const HAZARDS: &str = "[[hazards]]\nid = \"pvc\"\nname = \"PVC\"\nseverity = \"block\"\nkeywords = [\"pvc\"]\nreason = \"HCl\"\n";
    const MATERIAL: &str = r#"id = "mdf-3mm"
name = "MDF"
category = "engineered_wood"
thickness_mm = 3

[[recipes]]
id = "cut"
operation = "cut"
laser = { kind = "diode", power_w = 10 }
speed_mm_min = 300
power_min_pct = 100
power_max_pct = 100
passes = 2
confidence = { level = "community" }
"#;

    fn write(root: &Path, rel: &str, text: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    #[test]
    fn loads_valid_library() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "data/safety/hazards.toml", HAZARDS);
        write(dir.path(), "data/materials/wood/mdf-3mm.toml", MATERIAL);
        write(dir.path(), "data/materials/README.md", "ignored");
        let (lib, diags) = Loader::default().load(&Layout::new(dir.path()));
        assert!(diags.is_empty(), "{diags:?}");
        assert_eq!(lib.materials.len(), 1);
        assert_eq!(
            lib.materials[0].path,
            Path::new("data/materials/wood/mdf-3mm.toml")
        );
        assert_eq!(lib.material("mdf-3mm").unwrap().recipes[0].laser.power_w, 10.0);
        assert!(lib.material("nope").is_none());
        assert!(lib.machine("nope").is_none());
        assert!(lib.hazards.is_some());
    }

    #[test]
    fn reports_bad_files() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "data/materials/broken.toml", "id = ");
        write(dir.path(), "data/materials/wrong-name.toml", MATERIAL);
        write(
            dir.path(),
            "data/materials/schema-fail.toml",
            &MATERIAL
                .replace("id = \"mdf-3mm\"", "id = \"schema-fail\"")
                .replace("passes = 2", "passes = 0"),
        );
        write(dir.path(), "data/materials/tabs.toml", "id\t= \"tabs\"\r\n\n");
        let (lib, diags) = Loader::default().load(&Layout::new(dir.path()));
        assert!(lib.materials.is_empty());
        assert!(lib.hazards.is_none());
        let rules: Vec<_> = diags.iter().map(|d| d.rule).collect();
        assert!(rules.contains(&"toml"));
        assert!(rules.contains(&"toml-lint"));
        assert!(rules.contains(&"schema"));
        assert!(rules.contains(&"missing-file"));
        let text: Vec<String> = diags.iter().map(ToString::to_string).collect();
        assert!(
            text.iter()
                .any(|t| t.contains("file name must be `mdf-3mm.toml`")),
            "{text:?}"
        );
        assert!(text.iter().any(|t| t.contains("LF line endings")));
        assert!(text.iter().any(|t| t.contains("not tabs")));
        assert!(text.iter().any(|t| t.contains("exactly one newline")));
        assert!(diags.iter().all(|d| d.level == Level::Error));
    }

    #[test]
    fn model_errors_surface_without_schema() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "data/safety/hazards.toml", HAZARDS);
        write(dir.path(), "data/machines/m.toml", "id = \"m\"\n");
        let (lib, diags) = Loader::new(vec![]).load(&Layout::new(dir.path()));
        assert!(lib.machines.is_empty());
        assert_eq!(diags.len(), 1);
        assert_eq!(diags.iter().next().unwrap().rule, "model");
    }

    #[test]
    fn unreadable_file_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "data/safety/hazards.toml", HAZARDS);
        // Invalid UTF-8 cannot be read as a string.
        let path = dir.path().join("data/machines/bin.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();
        let (_, diags) = Loader::default().load(&Layout::new(dir.path()));
        assert_eq!(diags.iter().next().unwrap().rule, "io");
    }
}
