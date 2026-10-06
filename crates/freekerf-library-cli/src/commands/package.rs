use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use freekerf_library::layout::Layout;
use freekerf_library::package::{self, BuildOptions};
use semver::Version;

pub fn run(root: &Path, version: &str, out: &Path, mtime: u64) -> Result<ExitCode> {
    let version = Version::parse(version.trim_start_matches('v'))
        .with_context(|| format!("`{version}` is not a semver version"))?;
    let layout = Layout::new(root);
    let (library, diagnostics) = freekerf_library::check(&layout);
    if diagnostics.has_errors() {
        for d in &diagnostics {
            eprintln!("{d}");
        }
        bail!("the library has validation errors; refusing to package");
    }
    let output = package::build(
        &library,
        &layout,
        &BuildOptions {
            version,
            out_dir: out.to_path_buf(),
            mtime,
        },
    )?;
    println!("wrote {}", output.archive.display());
    println!("wrote {}", output.index_file.display());
    println!(
        "{} materials, {} machines",
        output.index.materials.len(),
        output.index.machines.len()
    );
    Ok(ExitCode::SUCCESS)
}
