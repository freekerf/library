//! Physical plausibility of process parameters.

use std::ops::RangeInclusive;

use super::{Context, Rule, recipes};
use crate::diagnostics::Diagnostics;
use crate::model::{LaserKind, Operation};

/// Plausible real optical power for each laser technology, in watts.
pub fn plausible_power_w(kind: LaserKind) -> RangeInclusive<f64> {
    match kind {
        LaserKind::Diode => 0.5..=60.0,
        LaserKind::Co2 => 10.0..=300.0,
        LaserKind::Fiber => 10.0..=500.0,
    }
}

/// Plausible feed rate for each laser technology, in mm/min (galvo fibers are fast).
pub fn plausible_speed_mm_min(kind: LaserKind) -> RangeInclusive<u32> {
    match kind {
        LaserKind::Diode => 1..=60_000,
        LaserKind::Co2 => 1..=120_000,
        LaserKind::Fiber => 1..=1_200_000,
    }
}

/// `power_min_pct <= power_max_pct` and `power_max_pct > 0`.
pub struct PowerRange;

impl Rule for PowerRange {
    fn name(&self) -> &'static str {
        "power-range"
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Diagnostics) {
        for (path, _, r) in recipes(ctx.library) {
            if r.power_min_pct > r.power_max_pct {
                out.error(
                    self.name(),
                    format!(
                        "power_min_pct ({}) is greater than power_max_pct ({})",
                        r.power_min_pct, r.power_max_pct
                    ),
                )
                .file(path)
                .context(format!("recipe {}", r.id));
            }
            if r.power_max_pct <= 0.0 {
                out.error(self.name(), "power_max_pct must be greater than 0")
                    .file(path)
                    .context(format!("recipe {}", r.id));
            }
        }
    }
}

/// Power and speed within plausible bands for the laser technology; many passes or
/// cutting without a thickness are flagged for review.
pub struct PlausibleProcess;

impl Rule for PlausibleProcess {
    fn name(&self) -> &'static str {
        "plausible-process"
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Diagnostics) {
        for (path, material, r) in recipes(ctx.library) {
            let context = format!("recipe {}", r.id);
            let kind = r.laser.kind;
            let power = plausible_power_w(kind);
            if !power.contains(&r.laser.power_w) {
                out.error(
                    self.name(),
                    format!(
                        "{} W is not a plausible optical power for a {} laser ({}–{} W)",
                        r.laser.power_w,
                        kind.as_str(),
                        power.start(),
                        power.end()
                    ),
                )
                .file(path)
                .context(context.clone());
            }
            let speed = plausible_speed_mm_min(kind);
            if !speed.contains(&r.speed_mm_min) {
                out.error(
                    self.name(),
                    format!(
                        "{} mm/min is not a plausible speed for a {} laser (max {} mm/min)",
                        r.speed_mm_min,
                        kind.as_str(),
                        speed.end()
                    ),
                )
                .file(path)
                .context(context.clone());
            }
            if r.passes > 20 {
                out.warning(
                    self.name(),
                    format!("{} passes is unusual; double-check", r.passes),
                )
                .file(path)
                .context(context.clone());
            }
            if r.operation == Operation::Cut && material.thickness_mm.is_none() {
                out.warning(self.name(), "cut recipe on a material without thickness_mm")
                    .file(path)
                    .context(context);
            }
        }
    }
}

/// Raster-only fields (`dpi`, `line_interval_mm`, `dithering`) are only used on
/// raster recipes, and `dpi` agrees with `line_interval_mm` when both are set.
pub struct RasterParams;

impl Rule for RasterParams {
    fn name(&self) -> &'static str {
        "raster-params"
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Diagnostics) {
        for (path, _, r) in recipes(ctx.library) {
            let context = format!("recipe {}", r.id);
            let raster_fields = r.dpi.is_some() || r.line_interval_mm.is_some() || r.dithering.is_some();
            if r.operation != Operation::EngraveRaster && raster_fields {
                out.error(
                    self.name(),
                    format!(
                        "dpi, line_interval_mm and dithering only apply to engrave_raster (operation is {})",
                        r.operation.as_str()
                    ),
                )
                .file(path)
                .context(context.clone());
            }
            if let (Some(dpi), Some(interval)) = (r.dpi, r.line_interval_mm) {
                let expected = 25.4 / f64::from(dpi);
                if ((interval - expected) / expected).abs() > 0.1 {
                    out.error(
                        self.name(),
                        format!(
                            "dpi {dpi} implies line_interval_mm ≈ {expected:.3}, got {interval}; set only one"
                        ),
                    )
                    .file(path)
                    .context(context);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Dithering;
    use crate::validate::fixtures::*;

    #[test]
    fn power_range() {
        let mut a = recipe("a");
        a.power_min_pct = 80.0;
        a.power_max_pct = 50.0;
        let mut b = recipe("b");
        b.power_min_pct = 0.0;
        b.power_max_pct = 0.0;
        let lib = library(vec![material("m", vec![a, b, recipe("ok")])], vec![]);
        let (errors, _) = run(PowerRange, &lib);
        assert_eq!(errors.len(), 2, "{errors:?}");
    }

    #[test]
    fn plausibility() {
        let mut a = recipe("a");
        a.laser.power_w = 150.0; // diode
        a.speed_mm_min = 90_000;
        a.passes = 30;
        let mut fiber = recipe("fiber");
        fiber.laser.kind = LaserKind::Fiber;
        fiber.laser.power_w = 30.0;
        fiber.speed_mm_min = 300_000;
        let mut m = material("m", vec![a, fiber]);
        m.doc.thickness_mm = None;
        let lib = library(vec![m], vec![]);
        let (errors, warnings) = run(PlausibleProcess, &lib);
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(errors[0].contains("optical power"));
        assert!(errors[1].contains("plausible speed"));
        assert_eq!(warnings.len(), 3, "{warnings:?}"); // passes + 2× cut without thickness
    }

    #[test]
    fn ranges_cover_all_kinds() {
        for kind in LaserKind::ALL {
            assert!(plausible_power_w(kind).start() > &0.0);
            assert!(plausible_speed_mm_min(kind).end() >= &60_000);
        }
    }

    #[test]
    fn raster_fields() {
        let mut cut = recipe("cut");
        cut.dpi = Some(254);
        let mut raster = recipe("raster");
        raster.operation = Operation::EngraveRaster;
        raster.dpi = Some(254);
        raster.line_interval_mm = Some(0.2);
        let mut good = recipe("good");
        good.operation = Operation::EngraveRaster;
        good.dpi = Some(254);
        good.line_interval_mm = Some(0.1);
        good.dithering = Some(Dithering::Jarvis);
        let lib = library(vec![material("m", vec![cut, raster, good])], vec![]);
        let (errors, _) = run(RasterParams, &lib);
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(errors[0].contains("only apply to engrave_raster"));
        assert!(errors[1].contains("implies line_interval_mm"));
    }
}
