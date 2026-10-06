//! Machine profile consistency.

use std::collections::HashSet;

use super::process::plausible_power_w;
use super::{Context, Rule};
use crate::diagnostics::Diagnostics;

/// `$$` settings only for Grbl-family firmware, unique keys, laser mode (`$32=1`)
/// recommended, and a plausible laser power.
pub struct MachineSettings;

impl Rule for MachineSettings {
    fn name(&self) -> &'static str {
        "machine-settings"
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Diagnostics) {
        for e in &ctx.library.machines {
            let m = &e.doc;
            let dollar = m.firmware.kind.uses_dollar_settings();
            if !dollar && !m.settings.is_empty() {
                out.error(
                    self.name(),
                    "`settings` ($$) are only supported for grbl and grbl_hal firmware",
                )
                .file(&e.path);
            }
            let mut keys = HashSet::new();
            for s in &m.settings {
                if !keys.insert(s.key.as_str()) {
                    out.error(self.name(), format!("setting {} declared twice", s.key))
                        .file(&e.path);
                }
            }
            if dollar && !m.settings.iter().any(|s| s.key == "$32" && s.value == "1") {
                out.warning(
                    self.name(),
                    "Grbl laser mode `$32 = 1` is not recommended in settings",
                )
                .file(&e.path);
            }
            let power = plausible_power_w(m.laser.kind);
            if !power.contains(&m.laser.power_w) {
                out.error(
                    self.name(),
                    format!(
                        "{} W is not a plausible optical power for a {} laser",
                        m.laser.power_w,
                        m.laser.kind.as_str()
                    ),
                )
                .file(&e.path);
            }
            if let Some(electrical) = m.electrical_power_w {
                if electrical < m.laser.power_w {
                    out.error(self.name(), "electrical_power_w is lower than the optical power")
                        .file(&e.path);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FirmwareKind, FirmwareSetting};
    use crate::validate::fixtures::*;

    #[test]
    fn settings() {
        let mut marlin = machine("marlin");
        marlin.doc.firmware.kind = FirmwareKind::Marlin;
        let mut dup = machine("dup");
        dup.doc.settings.push(dup.doc.settings[0].clone());
        let mut no_laser_mode = machine("no-laser-mode");
        no_laser_mode.doc.settings = vec![FirmwareSetting {
            key: "$30".into(),
            value: "1000".into(),
            description: "max S".into(),
        }];
        let mut weird = machine("weird");
        weird.doc.laser.power_w = 200.0;
        weird.doc.electrical_power_w = Some(100.0);
        let lib = library(vec![], vec![marlin, dup, no_laser_mode, weird, machine("ok")]);
        let (errors, warnings) = run(MachineSettings, &lib);
        assert_eq!(errors.len(), 4, "{errors:?}");
        assert_eq!(warnings.len(), 1, "{warnings:?}");
    }
}
