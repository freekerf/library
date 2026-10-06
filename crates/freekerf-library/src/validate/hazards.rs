//! Safety screening against the hazard list.

use super::{Context, Rule};
use crate::diagnostics::Diagnostics;
use crate::model::Severity;
use crate::safety::HazardMatcher;
use crate::text::normalize_words;

/// Rejects materials matching a `block` hazard and malformed hazard keywords.
/// (`warn` matches are expected: they are exported in the package index so the
/// app can alert the user.)
pub struct HazardScreen;

impl Rule for HazardScreen {
    fn name(&self) -> &'static str {
        "hazards"
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Diagnostics) {
        // A missing hazard list is reported by the loader.
        let Some(list) = &ctx.library.hazards else { return };
        for hazard in &list.doc.hazards {
            for keyword in hazard.keywords.iter().chain(&hazard.unless) {
                if normalize_words(keyword) != *keyword {
                    out.error(
                        self.name(),
                        format!(
                            "keyword `{keyword}` must be normalized as `{}`",
                            normalize_words(keyword)
                        ),
                    )
                    .file(&list.path)
                    .context(format!("hazard {}", hazard.id));
                }
            }
        }
        let matcher = HazardMatcher::new(&list.doc);
        for e in &ctx.library.materials {
            for hazard in matcher.matches(&e.doc.name) {
                if hazard.severity == Severity::Block {
                    out.error(
                        self.name(),
                        format!(
                            "material matches blocked hazard `{}` ({}): {}",
                            hazard.id, hazard.name, hazard.reason
                        ),
                    )
                    .file(&e.path);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::fixtures::*;

    #[test]
    fn screening() {
        let mut pvc = material("pvc-sheet", vec![recipe("r")]);
        pvc.doc.name = "Black PVC sheet".into();
        let mut abs = material("abs", vec![recipe("r")]);
        abs.doc.name = "ABS".into();
        let mut lib = library(vec![pvc, abs], vec![]);
        lib.hazards.as_mut().unwrap().doc.hazards[0]
            .keywords
            .push("Bad Keyword".into());
        let (errors, warnings) = run(HazardScreen, &lib);
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(errors[0].contains("must be normalized"));
        assert!(errors[1].contains("blocked hazard `pvc`"));
        assert!(warnings.is_empty());
        lib.hazards = None;
        assert_eq!(run(HazardScreen, &lib), (vec![], vec![]));
    }
}
