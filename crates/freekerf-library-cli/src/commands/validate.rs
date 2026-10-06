use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;
use freekerf_library::diagnostics::Level;
use freekerf_library::layout::Layout;

pub fn run(root: &Path, deny_warnings: bool, json: bool) -> Result<ExitCode> {
    let (library, diagnostics) = freekerf_library::check(&Layout::new(root));
    let errors = diagnostics.count(Level::Error);
    let warnings = diagnostics.count(Level::Warning);
    if json {
        println!("{}", serde_json::to_string_pretty(&diagnostics)?);
    } else {
        for d in &diagnostics {
            println!("{d}");
        }
        println!(
            "{} materials ({} recipes), {} machines: {errors} error(s), {warnings} warning(s)",
            library.materials.len(),
            library
                .materials
                .iter()
                .map(|m| m.doc.recipes.len())
                .sum::<usize>(),
            library.machines.len(),
        );
    }
    let failed = errors > 0 || (deny_warnings && warnings > 0);
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}
