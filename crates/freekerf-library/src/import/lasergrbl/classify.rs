//! Maps free-text material names to a [`MaterialCategory`].

use crate::model::MaterialCategory;
use crate::text::contains_phrase;

/// Assigns a category to a material name.
pub trait Categorizer {
    /// Category for `name`.
    fn categorize(&self, name: &str) -> MaterialCategory;
}

/// Ordered keyword rules (first match wins); names are matched as whole words,
/// accent-insensitive, in English, Spanish and Portuguese.
pub struct KeywordCategorizer;

const RULES: &[(MaterialCategory, &[&str])] = &[
    (
        MaterialCategory::AnodizedMetal,
        &["anodised", "anodized", "anodizado"],
    ),
    (
        MaterialCategory::CoatedMetal,
        &[
            "powder coated",
            "plated",
            "electroplated",
            "paint sprayed",
            "lacado",
            "banado",
            "coated metal",
            "coated stainless steel",
            "pintado",
        ],
    ),
    (
        MaterialCategory::Metal,
        &[
            "stainless",
            "steel",
            "inox",
            "iron",
            "aluminium",
            "aluminum",
            "aluminio",
            "metal",
            "galvanized",
            "cold rolled",
            "copper",
            "brass",
            "aco",
        ],
    ),
    (
        MaterialCategory::Acrylic,
        &["acrylic", "acrilyc", "acrilico", "pmma"],
    ),
    (MaterialCategory::Leather, &["leather", "cuero", "couro"]),
    (MaterialCategory::Cork, &["cork", "corcho", "cortica"]),
    (
        MaterialCategory::EngineeredWood,
        &["plywood", "mdf", "hdf", "contrachapada", "compensado"],
    ),
    (
        MaterialCategory::Wood,
        &[
            "wood",
            "basswood",
            "bamboo",
            "pine",
            "pinewood",
            "hardwood",
            "mahogany",
            "paulownia",
            "cherry",
            "madera",
            "madeira",
            "birch",
            "walnut",
            "oak",
            "plank",
        ],
    ),
    (
        MaterialCategory::Paper,
        &[
            "paper",
            "papel",
            "card",
            "cardboard",
            "carton",
            "kraft",
            "kraftpaper",
            "paperboard",
            "craft board",
            "duplex board",
            "papelao",
        ],
    ),
    (
        MaterialCategory::Textile,
        &[
            "cloth",
            "denim",
            "fabric",
            "fabrics",
            "tela",
            "telas",
            "cotton",
            "felt",
            "non woven",
            "tecido",
        ],
    ),
    (MaterialCategory::Foam, &["foam", "eva", "kt board", "tablero kt"]),
    (
        MaterialCategory::Rubber,
        &["rubber", "eraser", "neoprene", "neopreno", "silicone", "borracha"],
    ),
    (
        MaterialCategory::Glass,
        &["glass", "mirror", "mirrors", "vidrio", "vidro", "espelho"],
    ),
    (
        MaterialCategory::Ceramic,
        &[
            "ceramic",
            "ceramics",
            "ceramica",
            "tile",
            "clay",
            "barro",
            "alumina",
            "porcelain",
        ],
    ),
    (
        MaterialCategory::Stone,
        &[
            "stone",
            "rock",
            "slate",
            "cobblestone",
            "pebble",
            "piedra",
            "pedra",
            "agate",
            "granite",
            "marble",
            "canto rodado",
        ],
    ),
    (
        MaterialCategory::Plastic,
        &["abs", "plastic", "plastico", "resin", "two color plate"],
    ),
];

impl Categorizer for KeywordCategorizer {
    fn categorize(&self, name: &str) -> MaterialCategory {
        RULES
            .iter()
            .find(|(_, words)| words.iter().any(|w| contains_phrase(name, w)))
            .map_or(MaterialCategory::Other, |(category, _)| *category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use MaterialCategory::*;

    #[test]
    fn categories() {
        let c = |n| KeywordCategorizer.categorize(n);
        assert_eq!(c("Anodised Aluminium"), AnodizedMetal);
        assert_eq!(c("Electroplated aluminum"), CoatedMetal);
        assert_eq!(c("Mirror Stainless Steel"), Metal);
        assert_eq!(c("Acrílico"), Acrylic);
        assert_eq!(c("Cuero"), Leather);
        assert_eq!(c("Cork Wood"), Cork);
        assert_eq!(c("Madera contrachapada"), EngineeredWood);
        assert_eq!(c("Pine_Board"), Wood);
        assert_eq!(c("Cartón"), Paper);
        assert_eq!(c("Non-woven fabrics (dark color)"), Textile);
        assert_eq!(c("FOAM / EVA"), Foam);
        assert_eq!(c("Neoprene"), Rubber);
        assert_eq!(c("Mirrors (back)"), Glass);
        assert_eq!(c("Black Alumina"), Ceramic);
        assert_eq!(c("Piedra mate (Canto rodado)"), Stone);
        assert_eq!(c("Hoja de plástico (color oscuro)"), Plastic);
        assert_eq!(c("Artificial beef bone"), Other);
    }
}
