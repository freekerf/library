use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;
use freekerf_library::layout::Layout;
use freekerf_library::schema;

pub fn run(root: &Path, check: bool) -> Result<ExitCode> {
    let dir = Layout::new(root).schema_dir();
    if check {
        let stale = schema::stale(&dir);
        for path in &stale {
            println!("out of date: {}", path.display());
        }
        if !stale.is_empty() {
            println!("run `freekerf-library schema` and commit the result");
            return Ok(ExitCode::FAILURE);
        }
        println!("schemas up to date");
    } else {
        for path in schema::write_all(&dir)? {
            println!("wrote {}", path.display());
        }
    }
    Ok(ExitCode::SUCCESS)
}
