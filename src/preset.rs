// TODO: validate these values.
// TODO: Arc<String> for all of these
#[derive(Debug, serde::Deserialize)]
pub struct Preset {
    pub epp: Option<String>, // TODO: make enum
    pub hwp_dynamic_boost: Option<bool>,
    pub no_turbo: Option<bool>,
    pub scaling_governor: Option<String>, // TODO: make enum
}

impl Preset {
    pub fn apply(&self) {
        if let Some(hwp_dynamic_boost) = self.hwp_dynamic_boost
            && let Err(err) = std::fs::write(
                "/sys/devices/system/cpu/intel_pstate/hwp_dynamic_boost",
                if hwp_dynamic_boost { "1" } else { "0" },
            )
        {
            log::error!("Failed to set hwp_dynamic_boost: {err}");
        }

        if let Some(no_turbo) = self.no_turbo
            && let Err(err) = std::fs::write(
                "/sys/devices/system/cpu/intel_pstate/no_turbo",
                if no_turbo { "1" } else { "0" },
            )
        {
            log::error!("Failed to set no_turbo: {err}");
        }

        match std::fs::read_dir("/sys/devices/system/cpu/cpufreq/") {
            Ok(dir) => {
                for entry in dir {
                    let Ok(dir) = entry else {
                        continue;
                    };
                    let path = dir.path();

                    // The ordering of these two settings is important. If the governer is set to
                    // performance, then epp can only be set to performance. So if governor=performance and
                    // we're applying governor=powersave, epp=power, then applying epp first will fail.

                    if let Some(scaling_governor) = self.scaling_governor.as_deref()
                        && let Err(err) =
                            std::fs::write(path.join("scaling_governor"), scaling_governor)
                    {
                        log::error!("Failed to set scaling_governor for `{path:?}`: {err}");
                    }

                    if let Some(epp) = self.epp.as_deref()
                        && let Err(err) =
                            std::fs::write(path.join("energy_performance_preference"), epp)
                    {
                        log::error!("Failed to set epp for `{path:?}`: {err}");
                    }
                }
            }
            Err(err) => {
                log::error!("Failed to enumerate cpufreq policies: {err}");
            }
        }
    }
}
