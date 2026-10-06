//! Confidence-level requirements.

use std::path::{Component, Path};

use super::{Context, Rule, recipes};
use crate::diagnostics::Diagnostics;
use crate::model::Confidence;

/// `tested` needs an author and evidence that exists (https URL or a file in the
/// repository); `estimated.derived_from` must point to an existing recipe.
pub struct ConfidenceEvidence;

impl Rule for ConfidenceEvidence {
    fn name(&self) -> &'static str {
        "confidence-evidence"
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Diagnostics) {
        for (path, _, r) in recipes(ctx.library) {
            let context = format!("recipe {}", r.id);
            match &r.confidence {
                Confidence::Tested { author, evidence, .. } => {
                    if author.trim().is_empty() {
                        out.error(self.name(), "tested recipes need an author")
                            .file(path)
                            .context(context.clone());
                    }
                    for item in evidence {
                        if let Some(problem) = evidence_problem(ctx.layout.root(), item) {
                            out.error(self.name(), format!("evidence `{item}`: {problem}"))
                                .file(path)
                                .context(context.clone());
                        }
                    }
                }
                Confidence::Estimated {
                    derived_from: Some(reference),
                    ..
                } => {
                    let found = reference.split_once('/').is_some_and(|(material, recipe)| {
                        ctx.library
                            .material(material)
                            .is_some_and(|m| m.recipes.iter().any(|x| x.id == recipe))
                    });
                    if !found {
                        out.error(
                            self.name(),
                            format!("derived_from `{reference}` does not name an existing <material-id>/<recipe-id>"),
                        )
                        .file(path)
                        .context(context);
                    }
                }
                _ => {}
            }
        }
    }
}

fn evidence_problem(root: &Path, item: &str) -> Option<&'static str> {
    if item.starts_with("https://") {
        return None;
    }
    if item.contains("://") {
        return Some("only https URLs are accepted");
    }
    let rel = Path::new(item);
    if !rel.components().all(|c| matches!(c, Component::Normal(_))) {
        return Some("must be a relative path inside the repository");
    }
    if !root.join(rel).is_file() {
        return Some("file not found");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::fixtures::*;

    fn tested(author: &str, evidence: &[&str]) -> Confidence {
        Confidence::Tested {
            author: author.into(),
            evidence: evidence.iter().map(|s| s.to_string()).collect(),
            date: None,
        }
    }

    #[test]
    fn tested_requires_evidence() {
        let mut ok = recipe("ok");
        ok.confidence = tested("Ana", &["https://example.org/grid.jpg", "Cargo.toml"]);
        let mut bad = recipe("bad");
        bad.confidence = tested(" ", &["http://x/y.jpg", "../secret", "missing.jpg"]);
        let lib = library(vec![material("a", vec![ok, bad])], vec![]);
        let (errors, _) = run(ConfidenceEvidence, &lib);
        assert_eq!(errors.len(), 4, "{errors:?}");
        assert!(errors[1].contains("only https"));
        assert!(errors[2].contains("relative path"));
        assert!(errors[3].contains("not found"));
    }

    #[test]
    fn estimated_reference() {
        let estimated = |from: &str| Confidence::Estimated {
            method: "power scaling".into(),
            derived_from: Some(from.into()),
        };
        let mut ok = recipe("ok");
        ok.confidence = estimated("a/base");
        let mut bad = recipe("bad");
        bad.confidence = estimated("a/nope");
        let mut malformed = recipe("malformed");
        malformed.confidence = estimated("nothing");
        let lib = library(
            vec![material("a", vec![recipe("base"), ok, bad, malformed])],
            vec![],
        );
        let (errors, _) = run(ConfidenceEvidence, &lib);
        assert_eq!(errors.len(), 2, "{errors:?}");
    }
}
