//! Matching material names against the hazard list.

use crate::model::{Hazard, HazardList, Severity};
use crate::text::contains_phrase;

/// Matches free-text material names against a [`HazardList`].
pub struct HazardMatcher<'a> {
    list: &'a HazardList,
}

impl<'a> HazardMatcher<'a> {
    /// Matcher over `list`.
    pub fn new(list: &'a HazardList) -> Self {
        Self { list }
    }

    /// Every hazard whose keywords appear in `text` (and none of its `unless` phrases).
    pub fn matches(&self, text: &str) -> Vec<&'a Hazard> {
        self.list
            .hazards
            .iter()
            .filter(|h| h.keywords.iter().any(|k| contains_phrase(text, k)))
            .filter(|h| !h.unless.iter().any(|u| contains_phrase(text, u)))
            .collect()
    }

    /// The most severe matching hazard, if any.
    pub fn worst(&self, text: &str) -> Option<&'a Hazard> {
        self.matches(text).into_iter().max_by_key(|h| h.severity)
    }

    /// Whether `text` matches a [`Severity::Block`] hazard.
    pub fn is_blocked(&self, text: &str) -> bool {
        self.worst(text).is_some_and(|h| h.severity == Severity::Block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hazard(id: &str, severity: Severity, keywords: &[&str], unless: &[&str]) -> Hazard {
        Hazard {
            id: id.into(),
            name: id.into(),
            severity,
            keywords: keywords.iter().map(|s| s.to_string()).collect(),
            unless: unless.iter().map(|s| s.to_string()).collect(),
            reason: "r".into(),
            emissions: vec![],
        }
    }

    #[test]
    fn matching() {
        let list = HazardList {
            hazards: vec![
                hazard(
                    "leather",
                    Severity::Warn,
                    &["leather", "couro"],
                    &["vegetable tanned"],
                ),
                hazard("chrome-leather", Severity::Block, &["chrome tanned"], &[]),
                hazard("pvc", Severity::Block, &["pvc", "vinyl"], &[]),
            ],
        };
        let m = HazardMatcher::new(&list);
        assert_eq!(m.worst("Leather 2mm").unwrap().id, "leather");
        assert!(!m.is_blocked("Leather 2mm"));
        assert!(m.matches("Vegetable-tanned leather").is_empty());
        assert_eq!(m.worst("Chrome tanned leather").unwrap().id, "chrome-leather");
        assert!(m.is_blocked("Poly_Vinyl_Chloride"));
        assert!(m.worst("Basswood").is_none());
        assert!(!m.is_blocked("Basswood"));
    }
}
