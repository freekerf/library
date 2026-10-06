//! Identifier uniqueness.

use std::collections::{HashMap, HashSet};

use super::{Context, Rule};
use crate::diagnostics::Diagnostics;

/// Material, machine, hazard and per-material recipe ids are unique; warns
/// about two recipes for the same operation/laser/machine/source record in one material.
pub struct UniqueIds;

const RULE: &str = "unique-ids";

impl Rule for UniqueIds {
    fn name(&self) -> &'static str {
        RULE
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Diagnostics) {
        let lib = ctx.library;
        let mut seen = HashMap::new();
        for e in &lib.materials {
            if let Some(first) = seen.insert(e.doc.id.as_str(), &e.path) {
                out.error(
                    RULE,
                    format!("material id `{}` already used by {}", e.doc.id, first.display()),
                )
                .file(&e.path);
            }
        }
        let mut seen = HashMap::new();
        for e in &lib.machines {
            if let Some(first) = seen.insert(e.doc.id.as_str(), &e.path) {
                out.error(
                    RULE,
                    format!("machine id `{}` already used by {}", e.doc.id, first.display()),
                )
                .file(&e.path);
            }
        }
        if let Some(h) = &lib.hazards {
            let mut seen = HashSet::new();
            for hazard in &h.doc.hazards {
                if !seen.insert(hazard.id.as_str()) {
                    out.error(RULE, format!("duplicate hazard id `{}`", hazard.id))
                        .file(&h.path);
                }
            }
        }
        for e in &lib.materials {
            let mut ids = HashSet::new();
            let mut keys = HashSet::new();
            for r in &e.doc.recipes {
                if !ids.insert(r.id.as_str()) {
                    out.error(RULE, format!("duplicate recipe id `{}`", r.id))
                        .file(&e.path);
                }
                let key = (
                    r.operation,
                    r.laser.kind,
                    r.laser.power_w.to_bits(),
                    r.machine.as_deref(),
                    r.source.as_ref().and_then(|s| s.reference.as_deref()),
                );
                if !keys.insert(key) {
                    out.warning(
                        RULE,
                        "another recipe has the same operation, laser and machine; merge them or explain the difference in `notes`",
                    )
                    .file(&e.path)
                    .context(format!("recipe {}", r.id));
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
    fn duplicates() {
        let mut lib = library(
            vec![
                material("a", vec![recipe("r"), recipe("r")]),
                material("a", vec![recipe("x")]),
            ],
            vec![machine("m"), machine("m")],
        );
        let mut h = lib.hazards.take().unwrap();
        h.doc.hazards.push(h.doc.hazards[0].clone());
        lib.hazards = Some(h);
        let (errors, warnings) = run(UniqueIds, &lib);
        assert_eq!(errors.len(), 4, "{errors:?}");
        assert!(errors.iter().any(|e| e.starts_with("material id `a`")));
        assert!(errors.iter().any(|e| e.starts_with("machine id `m`")));
        assert!(errors.iter().any(|e| e.starts_with("duplicate hazard id")));
        assert!(errors.iter().any(|e| e.starts_with("duplicate recipe id")));
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn clean() {
        let mut other = recipe("b");
        other.laser.power_w = 20.0;
        let lib = library(vec![material("a", vec![recipe("a"), other])], vec![]);
        let (errors, warnings) = run(UniqueIds, &lib);
        assert!(errors.is_empty() && warnings.is_empty());
    }
}
