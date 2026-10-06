//! Small text helpers: slugs and accent-insensitive word matching.

/// Folds common Latin accents to ASCII and lowercases.
fn fold(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
        'ç' | 'Ç' => 'c',
        'ñ' | 'Ñ' => 'n',
        other => other.to_ascii_lowercase(),
    }
}

/// Lowercase ASCII words separated by single spaces (`"Poly_Vinyl-Chloride"` → `"poly vinyl chloride"`).
pub fn normalize_words(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut pending_space = false;
    for c in input.chars().map(fold) {
        if c.is_ascii_alphanumeric() {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            out.push(c);
        } else {
            pending_space = true;
        }
    }
    out
}

/// Kebab-case identifier (`"Ortur LU2 (5.5W)"` → `"ortur-lu2-5-5w"`).
pub fn slugify(input: &str) -> String {
    normalize_words(input).replace(' ', "-")
}

/// Whether `phrase` appears in `text` as whole words (both normalized here).
pub fn contains_phrase(text: &str, phrase: &str) -> bool {
    let text = format!(" {} ", normalize_words(text));
    let phrase = normalize_words(phrase);
    !phrase.is_empty() && text.contains(&format!(" {phrase} "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes() {
        assert_eq!(normalize_words("  Poly_Vinyl-Chloride "), "poly vinyl chloride");
        assert_eq!(normalize_words("Acrílico Ação ñ"), "acrilico acao n");
        assert_eq!(normalize_words("---"), "");
    }

    #[test]
    fn slugs() {
        assert_eq!(slugify("Ortur LU2 (5.5W)"), "ortur-lu2-5-5w");
        assert_eq!(slugify("Cartón"), "carton");
    }

    #[test]
    fn phrases() {
        assert!(contains_phrase("Black PVC sheet", "pvc"));
        assert!(contains_phrase("Poly_Vinyl_Chloride", "vinyl"));
        assert!(contains_phrase("Chrome-tanned leather", "chrome tanned"));
        assert!(!contains_phrase("Vinylidene", "vinyl"));
        assert!(!contains_phrase("anything", "  "));
    }
}
