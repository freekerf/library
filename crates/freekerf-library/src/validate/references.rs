//! Cross-document references from recipes to machine profiles.

use super::{Context, Rule, recipes};
use crate::diagnostics::Diagnostics;

/// `recipe.machine` must exist, use the same laser technology and (roughly) the same power.
pub struct MachineReferences;

impl Rule for MachineReferences {
    fn name(&self) -> &'static str {
        "machine-references"
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Diagnostics) {
        for (path, _, r) in recipes(ctx.library) {
            let Some(id) = &r.machine else { continue };
            let context = format!("recipe {}", r.id);
            let Some(machine) = ctx.library.machine(id) else {
                out.error(self.name(), format!("unknown machine `{id}`"))
                    .file(path)
                    .context(context);
                continue;
            };
            if machine.laser.kind != r.laser.kind {
                out.error(
                    self.name(),
                    format!(
                        "recipe laser is {} but machine `{id}` has a {} laser",
                        r.laser.kind.as_str(),
                        machine.laser.kind.as_str()
                    ),
                )
                .file(path)
                .context(context);
            } else if (machine.laser.power_w - r.laser.power_w).abs() > machine.laser.power_w * 0.1 {
                out.warning(
                    self.name(),
                    format!(
                        "recipe laser power {} W differs from machine `{id}` ({} W)",
                        r.laser.power_w, machine.laser.power_w
                    ),
                )
                .file(path)
                .context(context);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::LaserKind;
    use crate::validate::fixtures::*;

    #[test]
    fn references() {
        let mut ok = recipe("ok");
        ok.machine = Some("m".into());
        let mut unknown = recipe("unknown");
        unknown.machine = Some("nope".into());
        let mut kind = recipe("kind");
        kind.machine = Some("m".into());
        kind.laser.kind = LaserKind::Co2;
        kind.laser.power_w = 40.0;
        let mut power = recipe("power");
        power.machine = Some("m".into());
        power.laser.power_w = 20.0;
        let lib = library(
            vec![material("a", vec![ok, unknown, kind, power])],
            vec![machine("m")],
        );
        let (errors, warnings) = run(MachineReferences, &lib);
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(errors[0].contains("unknown machine"));
        assert!(errors[1].contains("co2"));
        assert_eq!(warnings.len(), 1);
    }
}
