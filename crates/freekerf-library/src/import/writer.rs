//! Writes materials as TOML files (`<dir>/<category>/<id>.toml`).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::model::Material;

/// Key order of a recipe block, matching hand-written files. Unknown keys go last.
const RECIPE_ORDER: &[&str] = &[
    "id",
    "operation",
    "laser",
    "machine",
    "speed_mm_min",
    "power_min_pct",
    "power_max_pct",
    "passes",
    "air_assist",
    "focus_offset_mm",
    "line_interval_mm",
    "dpi",
    "dithering",
    "confidence",
    "source",
    "notes",
];

/// Rebuilds a recipe table in [`RECIPE_ORDER`] with sub-tables inline.
fn canonical_recipe(recipe: &toml_edit::Table) -> toml_edit::Table {
    let mut keys: Vec<&str> = RECIPE_ORDER
        .iter()
        .copied()
        .filter(|k| recipe.contains_key(k))
        .collect();
    keys.extend(
        recipe
            .iter()
            .map(|(k, _)| k)
            .filter(|k| !RECIPE_ORDER.contains(k)),
    );
    let mut out = toml_edit::Table::new();
    for key in keys {
        let item = &recipe[key];
        let item = match item.as_table() {
            Some(table) => {
                let mut inline = table.clone().into_inline_table();
                inline.fmt();
                toml_edit::value(inline)
            }
            None => {
                let mut item = item.clone();
                if let Some(v) = item.as_value_mut() {
                    v.decor_mut().clear();
                }
                item
            }
        };
        out.insert(key, item);
    }
    out
}

/// Serializes a material in the canonical TOML layout (the same layout used by
/// hand-written files: one `[[recipes]]` block per recipe, small tables inline).
pub fn to_toml(material: &Material) -> String {
    let text = toml::to_string(material).expect("materials serialize to TOML");
    let mut doc: toml_edit::DocumentMut = text.parse().expect("serializer output parses");
    if let Some(recipes) = doc.get_mut("recipes").and_then(|r| r.as_array_of_tables_mut()) {
        for recipe in recipes.iter_mut() {
            *recipe = canonical_recipe(recipe);
        }
    }
    let mut text = doc.to_string();
    while text.ends_with("\n\n") {
        text.pop();
    }
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text
}

/// Writes each material to `<dir>/<category>/<id>.toml`, returning the paths.
pub fn write_materials(dir: &Path, materials: &[Material]) -> io::Result<Vec<PathBuf>> {
    materials
        .iter()
        .map(|m| {
            let path = dir.join(m.category.as_str()).join(format!("{}.toml", m.id));
            fs::create_dir_all(path.parent().expect("has parent"))?;
            fs::write(&path, to_toml(m))?;
            Ok(path)
        })
        .collect()
}

/// Deletes every `.toml` file under `dir` (used before regenerating an import).
pub fn remove_toml_files(dir: &Path) -> io::Result<usize> {
    let mut removed = 0;
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if entry.file_type().is_file() && path.extension().is_some_and(|e| e == "toml") {
            fs::remove_file(path)?;
            removed += 1;
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::fixtures;

    #[test]
    fn writes_and_cleans() {
        let dir = tempfile::tempdir().unwrap();
        let m = fixtures::material("mdf", vec![fixtures::recipe("r")]).doc;
        let paths = write_materials(dir.path(), std::slice::from_ref(&m)).unwrap();
        assert_eq!(paths, vec![dir.path().join("wood/mdf.toml")]);
        let text = fs::read_to_string(&paths[0]).unwrap();
        assert!(text.ends_with('\n') && !text.ends_with("\n\n"));
        let back: Material = toml::from_str(&text).unwrap();
        assert_eq!(back, m);
        fs::write(dir.path().join("README.md"), "keep").unwrap();
        assert_eq!(remove_toml_files(dir.path()).unwrap(), 1);
        assert!(dir.path().join("README.md").exists());
    }
}
