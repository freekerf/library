//! Resolves LaserGRBL model names to a laser source.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::model::{LaserKind, LaserSource};

/// Maps a machine model name to its laser source.
pub trait LaserResolver {
    /// The laser source of `model`, if known.
    fn resolve(&self, model: &str) -> Option<LaserSource>;
}

/// Manual model → laser table (`tools/lasergrbl/models.toml`).
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelOverrides {
    /// Model name (exactly as in the `.psh`) → laser source.
    #[serde(default)]
    pub models: BTreeMap<String, LaserSource>,
}

/// Resolver: explicit overrides first, then the optical power written in the
/// model name (`Ortur LU7W (1.5W)` → 1.5 W; parentheses win). LaserGRBL targets
/// diode lasers, so parsed names default to [`LaserKind::Diode`].
#[derive(Debug, Clone, Default)]
pub struct ModelResolver {
    overrides: ModelOverrides,
}

impl ModelResolver {
    /// Resolver with `overrides`.
    pub fn new(overrides: ModelOverrides) -> Self {
        Self { overrides }
    }
}

/// First `<number>W` token in `text` (`5W`, `5.5 w`, `2,5W`).
fn watts_in(text: &str) -> Option<f64> {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == ',') {
            i += 1;
        }
        let number: String = chars[start..i]
            .iter()
            .map(|&c| if c == ',' { '.' } else { c })
            .collect();
        let mut j = i;
        while j < chars.len() && chars[j] == ' ' {
            j += 1;
        }
        let is_watt = j < chars.len()
            && chars[j].eq_ignore_ascii_case(&'w')
            && chars.get(j + 1).is_none_or(|c| !c.is_ascii_alphanumeric());
        if is_watt {
            if let Ok(value) = number.trim_end_matches('.').parse::<f64>() {
                return Some(value);
            }
        }
    }
    None
}

/// Contents of every `(...)` group in `text`.
fn parenthesized(text: &str) -> impl Iterator<Item = &str> {
    text.split('(')
        .skip(1)
        .filter_map(|part| part.split_once(')').map(|(inside, _)| inside))
}

impl LaserResolver for ModelResolver {
    fn resolve(&self, model: &str) -> Option<LaserSource> {
        if let Some(source) = self.overrides.models.get(model) {
            return Some(source.clone());
        }
        let power_w = parenthesized(model)
            .find_map(watts_in)
            .or_else(|| watts_in(model))?;
        (power_w > 0.0).then_some(LaserSource {
            kind: LaserKind::Diode,
            power_w,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn power(model: &str) -> Option<f64> {
        ModelResolver::default().resolve(model).map(|s| s.power_w)
    }

    #[test]
    fn parses_names() {
        assert_eq!(power("Ortur LU7W (1.5W)"), Some(1.5));
        assert_eq!(power("Atomstack A5 20W (5W)"), Some(5.0));
        assert_eq!(power("YoraHome 3.5 (2.2W optical)"), Some(2.2));
        assert_eq!(power("Atomstack M50 10W (A10/S10/X7)"), Some(10.0));
        assert_eq!(power("Longer RAY5 5W"), Some(5.0));
        assert_eq!(power("LEON3D L60 (10w)"), Some(10.0));
        assert_eq!(power("Comgo Z1 2,5 W"), Some(2.5));
        assert_eq!(power("Two Trees TS2-10"), None);
        assert_eq!(power("Watts 5Wx"), None);
        assert_eq!(power("Zero 0W"), None);
    }

    #[test]
    fn overrides_win() {
        let overrides: ModelOverrides =
            toml::from_str("[models.\"SculpFun S9\"]\nkind = \"diode\"\npower_w = 5.5\n").unwrap();
        let r = ModelResolver::new(overrides);
        assert_eq!(r.resolve("SculpFun S9").unwrap().power_w, 5.5);
        assert!(r.resolve("Unknown").is_none());
    }
}
