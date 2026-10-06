//! Integration tests against the real repository content: the committed data
//! must validate, the committed schemas and the LaserGRBL import must be
//! reproducible, and the release package must round-trip.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use freekerf_library::diagnostics::Level;
use freekerf_library::import::Importer;
use freekerf_library::import::lasergrbl::{
    DEFAULT_SOURCE_URL, KeywordCategorizer, LaserGrblImporter, ModelOverrides, ModelResolver,
};
use freekerf_library::layout::Layout;
use freekerf_library::model::{Confidence, HazardList};
use freekerf_library::package::{self, BuildOptions};
use freekerf_library::{check, schema};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

#[test]
fn committed_data_has_no_errors() {
    let (library, diagnostics) = check(&Layout::new(repo_root()));
    let errors: Vec<String> = diagnostics
        .iter()
        .filter(|d| d.level == Level::Error)
        .map(ToString::to_string)
        .collect();
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    assert!(library.materials.len() >= 10);
    assert!(library.machines.len() >= 3);
}

#[test]
fn milestone_one_examples_exist() {
    let (library, _) = check(&Layout::new(repo_root()));
    let curated: Vec<_> = library
        .materials
        .iter()
        .filter(|m| !m.path.starts_with("data/materials/imported"))
        .collect();
    assert!(curated.len() >= 10, "milestone 1 requires 10 example materials");
    // Curated starting values are not verified: they must not claim more than `estimated`.
    for m in curated {
        for r in &m.doc.recipes {
            assert!(
                !matches!(r.confidence, Confidence::Tested { .. }),
                "{} / {} claims tested without a test grid",
                m.doc.id,
                r.id
            );
        }
    }
}

#[test]
fn committed_schemas_are_up_to_date() {
    let stale = schema::stale(&Layout::new(repo_root()).schema_dir());
    assert!(stale.is_empty(), "run `cargo run -- schema`: {stale:?}");
}

fn read_tree(dir: &Path) -> BTreeMap<PathBuf, String> {
    walkdir(dir)
        .into_iter()
        .map(|p| {
            (
                p.strip_prefix(dir).unwrap().to_path_buf(),
                fs::read_to_string(&p).unwrap(),
            )
        })
        .collect()
}

fn walkdir(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(walkdir(&path));
        } else {
            out.push(path);
        }
    }
    out
}

#[test]
fn lasergrbl_import_is_reproducible() {
    let root = repo_root();
    let layout = Layout::new(&root);
    let hazards: HazardList = toml::from_str(&fs::read_to_string(layout.hazards_file()).unwrap()).unwrap();
    let overrides: ModelOverrides =
        toml::from_str(&fs::read_to_string(root.join("tools/lasergrbl/models.toml")).unwrap()).unwrap();
    let resolver = ModelResolver::new(overrides);
    let importer = LaserGrblImporter::new(&resolver, &KeywordCategorizer, &hazards, DEFAULT_SOURCE_URL);
    let input = fs::read_to_string(root.join("tools/lasergrbl/upstream/StandardMaterials.psh")).unwrap();
    let outcome = importer.import(&input).unwrap();

    let tmp = tempfile::tempdir().unwrap();
    freekerf_library::import::write_materials(tmp.path(), &outcome.materials).unwrap();
    fs::write(
        tmp.path().join("REPORT.md"),
        outcome
            .report
            .to_markdown("Importação LaserGRBL", outcome.materials.len()),
    )
    .unwrap();

    let committed = read_tree(&root.join("data/materials/imported/lasergrbl"));
    let generated = read_tree(tmp.path());
    assert_eq!(
        committed.keys().collect::<Vec<_>>(),
        generated.keys().collect::<Vec<_>>(),
        "re-run the LaserGRBL import (see doc/features/import-lasergrbl.md)"
    );
    for (path, text) in &generated {
        assert_eq!(
            &committed[path],
            text,
            "{} differs from a fresh import",
            path.display()
        );
    }
}

#[test]
fn package_round_trip() {
    let root = repo_root();
    let layout = Layout::new(&root);
    let (library, _) = check(&layout);
    let tmp = tempfile::tempdir().unwrap();
    let out = package::build(
        &library,
        &layout,
        &BuildOptions {
            version: semver::Version::new(0, 1, 0),
            out_dir: tmp.path().to_path_buf(),
            mtime: 0,
        },
    )
    .unwrap();
    let package = package::read_archive(&out.archive).unwrap();
    assert_eq!(package.index.materials.len(), library.materials.len());
    assert_eq!(package.materials().unwrap().len(), library.materials.len());
    assert!(package.files.contains_key("LICENSE"));
    assert!(package.files.contains_key("NOTICE"));
    // Hazard warnings are exported for the app.
    assert!(
        package
            .index
            .materials
            .iter()
            .any(|m| m.hazards.iter().any(|h| h.id == "leather"))
    );
}
