//! Licensing of third-party data.

use super::{Context, Rule, recipes};
use crate::diagnostics::Diagnostics;

/// SPDX identifiers whose data may be redistributed in a GPL-3.0-or-later library.
pub const COMPATIBLE_LICENSES: &[&str] = &[
    "GPL-3.0-or-later",
    "GPL-3.0-only",
    "GPL-2.0-or-later",
    "LGPL-3.0-or-later",
    "LGPL-2.1-or-later",
    "AGPL-3.0-or-later",
    "CC0-1.0",
    "CC-BY-4.0",
    "CC-BY-SA-4.0",
    "MIT",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "Apache-2.0",
    "ISC",
    "Zlib",
    "Unlicense",
];

/// `recipe.source.license` must be a known GPL-3.0-compatible SPDX id.
pub struct SourceLicense;

impl Rule for SourceLicense {
    fn name(&self) -> &'static str {
        "source-license"
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Diagnostics) {
        for (path, _, r) in recipes(ctx.library) {
            let Some(source) = &r.source else { continue };
            if !COMPATIBLE_LICENSES.contains(&source.license.as_str()) {
                out.error(
                    self.name(),
                    format!(
                        "license `{}` of source `{}` is not in the GPL-3.0-compatible allow-list",
                        source.license, source.name
                    ),
                )
                .file(path)
                .context(format!("recipe {}", r.id));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Source;
    use crate::validate::fixtures::*;

    #[test]
    fn licenses() {
        let source = |license: &str| Source {
            name: "x".into(),
            url: None,
            license: license.into(),
            reference: None,
        };
        let mut ok = recipe("ok");
        ok.source = Some(source("GPL-3.0-or-later"));
        let mut bad = recipe("bad");
        bad.source = Some(source("LicenseRef-Proprietary"));
        let lib = library(vec![material("a", vec![ok, bad, recipe("none")])], vec![]);
        let (errors, _) = run(SourceLicense, &lib);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("LicenseRef-Proprietary"));
    }
}
