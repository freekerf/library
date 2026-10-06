use std::fs;
use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context, Result};
use freekerf_library::import::lasergrbl::{
    KeywordCategorizer, LaserGrblImporter, ModelOverrides, ModelResolver,
};
use freekerf_library::import::{Importer, remove_toml_files, write_materials};
use freekerf_library::layout::Layout;
use freekerf_library::model::HazardList;

pub struct Args<'a> {
    pub root: &'a Path,
    pub input: &'a Path,
    pub out: &'a Path,
    pub models: &'a Path,
    pub source_url: &'a str,
    pub clean: bool,
}

pub fn lasergrbl(args: &Args<'_>) -> Result<ExitCode> {
    let layout = Layout::new(args.root);
    let hazards_path = layout.hazards_file();
    let hazards: HazardList = toml::from_str(
        &fs::read_to_string(&hazards_path).with_context(|| format!("reading {}", hazards_path.display()))?,
    )
    .with_context(|| format!("parsing {}", hazards_path.display()))?;

    let models_path = args.root.join(args.models);
    let overrides = if models_path.is_file() {
        toml::from_str::<ModelOverrides>(&fs::read_to_string(&models_path)?)
            .with_context(|| format!("parsing {}", models_path.display()))?
    } else {
        ModelOverrides::default()
    };
    let resolver = ModelResolver::new(overrides);
    let importer = LaserGrblImporter::new(&resolver, &KeywordCategorizer, &hazards, args.source_url);

    let input =
        fs::read_to_string(args.input).with_context(|| format!("reading {}", args.input.display()))?;
    let outcome = importer.import(&input)?;

    let out_dir = args.root.join(args.out);
    if args.clean && out_dir.is_dir() {
        remove_toml_files(&out_dir)?;
    }
    let written = write_materials(&out_dir, &outcome.materials)?;
    let report = outcome
        .report
        .to_markdown("Importação LaserGRBL", outcome.materials.len());
    fs::write(out_dir.join("REPORT.md"), report)?;

    println!(
        "{} rows, {} recipes imported into {} materials ({} skipped); see {}",
        outcome.report.rows,
        outcome.report.imported,
        written.len(),
        outcome.report.skipped.len(),
        out_dir.join("REPORT.md").display()
    );
    Ok(ExitCode::SUCCESS)
}
