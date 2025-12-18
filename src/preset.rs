// TODO: validate these values.
#[derive(Debug, serde::Deserialize)]
pub struct Preset {
    pub epp: Option<String>, // TODO: make enum
    pub hwp_dynamic_boost: Option<bool>,
    pub no_turbo: Option<bool>,
    pub scaling_governor: Option<String>, // TODO: make enum
}

impl Preset {
    pub fn apply(&self) -> Result<(), std::io::Error> {
        if let Some(hwp_dynamic_boost) = self.hwp_dynamic_boost {
            std::fs::write(
                "/sys/devices/system/cpu/intel_pstate/hwp_dynamic_boost",
                if hwp_dynamic_boost { "1" } else { "0" },
            )?;
        }

        if let Some(no_turbo) = self.no_turbo {
            std::fs::write(
                "/sys/devices/system/cpu/intel_pstate/no_turbo",
                if no_turbo { "1" } else { "0" },
            )?;
        }

        for dir in std::fs::read_dir("/sys/devices/system/cpu/cpufreq/")? {
            let Ok(dir) = dir else {
                continue;
            };
            let path = dir.path();

            // The ordering of these two settings is important. If the governer is set to
            // performance, then epp can only be set to performance. So if governor=performance and
            // we're applying governor=powersave, epp=power, then applying epp first will fail.
            if let Some(scaling_governor) = self.scaling_governor.as_deref() {
                std::fs::write(path.join("scaling_governor"), scaling_governor)?;
            }
            if let Some(epp) = self.epp.as_deref() {
                std::fs::write(path.join("energy_performance_preference"), epp)?;
            }
        }

        Ok(())
    }
}
