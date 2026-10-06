//! Diagnostics collected while loading and validating the library.

use std::fmt;
use std::path::{Path, PathBuf};

use serde::Serialize;

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// Must be fixed; fails validation.
    Error,
    /// Should be reviewed; fails only with `--deny-warnings`.
    Warning,
}

/// A single finding.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagnostic {
    /// Severity.
    pub level: Level,
    /// Name of the check or rule that produced it.
    pub rule: &'static str,
    /// File the finding refers to, relative to the library root.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
    /// Location inside the file (e.g. a recipe id).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Human readable message.
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = match self.level {
            Level::Error => "error",
            Level::Warning => "warning",
        };
        write!(f, "{level}[{}]", self.rule)?;
        if let Some(file) = &self.file {
            write!(f, " {}", file.display())?;
        }
        if let Some(context) = &self.context {
            write!(f, " ({context})")?;
        }
        write!(f, ": {}", self.message)
    }
}

/// An ordered collection of diagnostics.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
}

impl Diagnostics {
    /// Empty collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a diagnostic.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.items.push(diagnostic);
    }

    /// Starts an error builder.
    pub fn error(&mut self, rule: &'static str, message: impl Into<String>) -> Builder<'_> {
        self.builder(Level::Error, rule, message.into())
    }

    /// Starts a warning builder.
    pub fn warning(&mut self, rule: &'static str, message: impl Into<String>) -> Builder<'_> {
        self.builder(Level::Warning, rule, message.into())
    }

    fn builder(&mut self, level: Level, rule: &'static str, message: String) -> Builder<'_> {
        self.items.push(Diagnostic {
            level,
            rule,
            file: None,
            context: None,
            message,
        });
        Builder {
            diagnostic: self.items.last_mut().expect("just pushed"),
        }
    }

    /// Appends all diagnostics from `other`.
    pub fn extend(&mut self, other: Diagnostics) {
        self.items.extend(other.items);
    }

    /// All diagnostics in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.items.iter()
    }

    /// Number of diagnostics at `level`.
    pub fn count(&self, level: Level) -> usize {
        self.items.iter().filter(|d| d.level == level).count()
    }

    /// Whether any error was recorded.
    pub fn has_errors(&self) -> bool {
        self.count(Level::Error) > 0
    }

    /// Whether the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Number of diagnostics.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Sorts by file, then level, then message for stable output.
    pub fn sort(&mut self) {
        self.items.sort_by(|a, b| {
            (&a.file, a.level, &a.context, &a.message).cmp(&(&b.file, b.level, &b.context, &b.message))
        });
    }
}

impl<'a> IntoIterator for &'a Diagnostics {
    type Item = &'a Diagnostic;
    type IntoIter = std::slice::Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

/// Fluent builder returned by [`Diagnostics::error`] / [`Diagnostics::warning`].
pub struct Builder<'a> {
    diagnostic: &'a mut Diagnostic,
}

impl Builder<'_> {
    /// Sets the file.
    pub fn file(self, path: &Path) -> Self {
        self.diagnostic.file = Some(path.to_path_buf());
        self
    }

    /// Sets the location inside the file.
    pub fn context(self, context: impl Into<String>) -> Self {
        self.diagnostic.context = Some(context.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_and_counts() {
        let mut d = Diagnostics::new();
        assert!(d.is_empty());
        d.warning("w", "careful").file(Path::new("b.toml"));
        d.error("e", "broken")
            .file(Path::new("a.toml"))
            .context("recipe x");
        assert_eq!(d.len(), 2);
        assert_eq!(d.count(Level::Error), 1);
        assert_eq!(d.count(Level::Warning), 1);
        assert!(d.has_errors());
        d.sort();
        let first = d.iter().next().unwrap();
        assert_eq!(first.to_string(), "error[e] a.toml (recipe x): broken");
        let second = (&d).into_iter().nth(1).unwrap();
        assert_eq!(second.to_string(), "warning[w] b.toml: careful");
    }

    #[test]
    fn extend_and_json() {
        let mut a = Diagnostics::new();
        let mut b = Diagnostics::new();
        b.push(Diagnostic {
            level: Level::Warning,
            rule: "r",
            file: None,
            context: None,
            message: "m".into(),
        });
        a.extend(b);
        assert!(!a.has_errors());
        let json = serde_json::to_string(&a).unwrap();
        assert_eq!(json, r#"[{"level":"warning","rule":"r","message":"m"}]"#);
    }
}
