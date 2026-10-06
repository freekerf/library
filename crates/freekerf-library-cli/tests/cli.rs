//! End-to-end tests of the `freekerf-library` binary.

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn cli(root: &Path) -> Command {
    let mut cmd = Command::cargo_bin("freekerf-library").unwrap();
    cmd.arg("--root").arg(root).env_remove("SOURCE_DATE_EPOCH");
    cmd
}

fn write(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

const HAZARDS: &str = "[[hazards]]\nid = \"pvc\"\nname = \"PVC\"\nseverity = \"block\"\nkeywords = [\"pvc\", \"vinyl\"]\nreason = \"HCl\"\n";

const MACHINE_NO_LASER_MODE: &str = r#"id = "m"
manufacturer = "Generic"
model = "M"
origin = "front_left"
work_area = { x_mm = 300, y_mm = 300 }
firmware = { kind = "grbl" }
features = { air_assist = false, rotary = false, camera = false, z_axis = false }
laser = { kind = "diode", power_w = 5 }
"#;

#[test]
fn validates_the_repository() {
    cli(&repo_root())
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("0 error(s)"));
}

#[test]
fn reports_errors_and_warnings() {
    let dir = tempfile::tempdir().unwrap();
    cli(dir.path())
        .args(["validate", "--format", "json"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"rule\": \"missing-file\""));

    write(dir.path(), "data/safety/hazards.toml", HAZARDS);
    write(dir.path(), "data/machines/m.toml", MACHINE_NO_LASER_MODE);
    cli(dir.path())
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("warning[machine-settings]"));
    cli(dir.path())
        .args(["validate", "--deny-warnings"])
        .assert()
        .failure();
}

#[test]
fn schema_generation_and_check() {
    let dir = tempfile::tempdir().unwrap();
    cli(dir.path())
        .args(["schema", "--check"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("out of date"));
    cli(dir.path()).arg("schema").assert().success();
    cli(dir.path())
        .args(["schema", "--check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("up to date"));
    cli(&repo_root()).args(["schema", "--check"]).assert().success();
}

#[test]
fn packages_the_repository() {
    let out = tempfile::tempdir().unwrap();
    cli(&repo_root())
        .args(["package", "--version", "v0.1.0", "--out"])
        .arg(out.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("library-v0.1.0.tar.zst"));
    assert!(out.path().join("library-v0.1.0.index.json").is_file());
}

#[test]
fn package_rejects_bad_input() {
    let dir = tempfile::tempdir().unwrap();
    cli(dir.path())
        .args(["package", "--version", "one"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a semver version"));
    cli(dir.path())
        .args(["package", "--version", "1.0.0"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("refusing to package"));
}

#[test]
fn imports_lasergrbl() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "data/safety/hazards.toml", HAZARDS);
    write(
        dir.path(),
        "in.psh",
        r#"<MaterialDB xmlns="http://tempuri.org/MaterialDB.xsd">
  <Materials><id>1</id><Model>Acme (5W)</Model><Material>MDF</Material><Thickness>3mm</Thickness><Action>Cut</Action><Power>100</Power><Speed>200</Speed><Cycles>2</Cycles></Materials>
  <Materials><id>2</id><Model>Acme (5W)</Model><Material>Vinyl</Material><Thickness>-</Thickness><Action>Cut</Action><Power>100</Power><Speed>200</Speed><Cycles>2</Cycles></Materials>
  <Materials><id>3</id><Model>Custom</Model><Material>MDF</Material><Thickness>3mm</Thickness><Action>Cut</Action><Power>100</Power><Speed>200</Speed><Cycles>2</Cycles></Materials>
</MaterialDB>"#,
    );
    write(
        dir.path(),
        "tools/lasergrbl/models.toml",
        "[models.Custom]\nkind = \"diode\"\npower_w = 2\n",
    );
    write(
        dir.path(),
        "data/materials/imported/lasergrbl/stale.toml",
        "stale",
    );
    cli(dir.path())
        .args(["import", "lasergrbl", "--clean", "--input"])
        .arg(dir.path().join("in.psh"))
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "3 rows, 2 recipes imported into 1 materials (1 skipped)",
        ));
    let out = dir.path().join("data/materials/imported/lasergrbl");
    assert!(!out.join("stale.toml").exists());
    assert!(out.join("engineered_wood/lasergrbl-mdf-3mm.toml").is_file());
    assert!(
        fs::read_to_string(out.join("REPORT.md"))
            .unwrap()
            .contains("bloqueado")
    );
    cli(dir.path()).arg("validate").assert().success();
}

#[test]
fn import_reports_missing_files() {
    let dir = tempfile::tempdir().unwrap();
    cli(dir.path())
        .args(["import", "lasergrbl", "--input", "nope.psh"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("hazards.toml"));
    write(dir.path(), "data/safety/hazards.toml", HAZARDS);
    cli(dir.path())
        .args(["import", "lasergrbl", "--input"])
        .arg(dir.path().join("nope.psh"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("nope.psh"));
}
