//! Data model (serde + JSON Schema). The Rust types are the single source of
//! truth for `schema/v1/*.schema.json`.

mod common;
mod hazard;
mod machine;
mod material;

pub use common::{ID_PATTERN, LaserKind, LaserSource};
pub use hazard::{Hazard, HazardList, Severity};
pub use machine::{Features, Firmware, FirmwareKind, FirmwareSetting, Machine, Origin, WorkArea};
pub use material::{Confidence, Dithering, Material, MaterialCategory, Operation, Recipe, Source};

#[cfg(test)]
mod tests {
    use super::*;

    fn serialized<T: serde::Serialize>(value: T) -> String {
        serde_json::to_value(value).unwrap().as_str().unwrap().to_string()
    }

    #[test]
    fn as_str_matches_serde() {
        for kind in LaserKind::ALL {
            assert_eq!(kind.as_str(), serialized(kind));
        }
        for op in [Operation::Cut, Operation::EngraveVector, Operation::EngraveRaster] {
            assert_eq!(op.as_str(), serialized(op));
        }
        use MaterialCategory::*;
        for c in [
            Wood,
            EngineeredWood,
            Acrylic,
            Leather,
            Paper,
            Textile,
            AnodizedMetal,
            CoatedMetal,
            Metal,
            Stone,
            Glass,
            Ceramic,
            Plastic,
            Foam,
            Rubber,
            Cork,
            Other,
        ] {
            assert_eq!(c.as_str(), serialized(c));
        }
    }

    #[test]
    fn confidence_levels() {
        let parse = |s: &str| toml::from_str::<Confidence>(s);
        let tested = parse("level = \"tested\"\nauthor = \"A\"\nevidence = [\"x.jpg\"]\n").unwrap();
        assert_eq!(tested.level(), "tested");
        assert_eq!(parse("level = \"community\"\n").unwrap().level(), "community");
        assert_eq!(
            parse("level = \"estimated\"\nmethod = \"m\"\n").unwrap().level(),
            "estimated"
        );
        assert!(parse("level = \"community\"\nextra = 1\n").is_err());
        assert!(parse("level = \"guaranteed\"\n").is_err());
    }

    #[test]
    fn firmware_families() {
        assert!(FirmwareKind::Grbl.uses_dollar_settings());
        assert!(FirmwareKind::GrblHal.uses_dollar_settings());
        assert!(!FirmwareKind::Smoothieware.uses_dollar_settings());
        assert!(!FirmwareKind::Marlin.uses_dollar_settings());
    }
}
