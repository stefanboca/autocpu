use std::{io::Read, path::PathBuf};

const FLROOT: &str = "/sys/devices/system/cpu";

fn get(path: &str) -> Option<String> {
    let path = PathBuf::from(FLROOT).join(path);
    // dbg!(&path);
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
}

fn get_bool(path: &str) -> Option<bool> {
    let path = PathBuf::from(FLROOT).join(path);
    std::fs::read_to_string(path)
        .ok()
        .and_then(|n| n.trim().parse::<u8>().ok())
        .and_then(|n| match n {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        })
}

fn set(path: &str, value: &str) -> Result<(), std::io::Error> {
    let path = PathBuf::from(FLROOT).join(path);
    std::fs::write(path, value)
}

pub fn driver() -> Option<String> {
    get("cpu0/cpufreq/scaling_driver")
}

pub fn available_governors() -> Vec<String> {
    get("cpu0/cpufreq/scaling_available_governors")
        .map(|s| s.trim().split(" ").map(String::from).collect())
        .unwrap_or_default()
}

pub fn governor(core: u16) -> Option<String> {
    get(&format!("cpu{core}/cpufreq/scaling_governor"))
}

pub fn set_governor(core: u16, governor: &str) -> Result<(), std::io::Error> {
    set(&format!("cpu{core}/cpufreq/scaling_governor"), governor)
}

pub fn available_frequencies() -> Vec<String> {
    get("cpu0/cpufreq/scaling_avaliable_frequencies")
        .map(|s| s.trim().split(" ").map(String::from).collect())
        .unwrap_or_default()
}

pub fn frequency(core: u16) -> Option<String> {
    get(&format!("cpu{core}/cpufreq/scaling_cur_freq"))
}

pub fn set_frequency(core: u16, frequency: u32) -> Result<(), std::io::Error> {
    // not supported on intel_pstate driver
    set(
        &format!("cpu{core}/cpufreq/scaling_setfreq"),
        &frequency.to_string(),
    )
}

pub fn min_frequency(core: u16) -> Option<String> {
    get(&format!("cpu{core}/cpufreq/scaling_min_freq"))
}

pub fn set_min_frequency(core: u16, frequency: u32) -> Result<(), std::io::Error> {
    set(
        &format!("cpu{core}/cpufreq/scaling_min_freq"),
        &frequency.to_string(),
    )
}

pub fn max_frequency(core: u16) -> Option<String> {
    get(&format!("cpu{core}/cpufreq/scaling_max_freq"))
}

pub fn set_max_frequency(core: u16, frequency: u32) -> Result<(), std::io::Error> {
    set(
        &format!("cpu${core}/cpufreq/scaling_max_freq"),
        &frequency.to_string(),
    )
}

pub fn energy_performance_available_preferences() -> Vec<String> {
    get("cpu0/cpufreq/energy_performance_available_preferences")
        .map(|s| s.trim().split(" ").map(String::from).collect())
        .unwrap_or_default()
}

pub fn energy_performance_preference(core: u16) -> Option<String> {
    get(&format!("cpu{core}/cpufreq/energy_performance_preference"))
}

pub fn set_energy_performance_preference(
    core: u16,
    preference: &str,
) -> Result<(), std::io::Error> {
    set(
        &format!("cpu{core}/cpufreq/energy_performance_preference"),
        preference,
    )
}

#[derive(Debug)]
#[repr(u8)]
pub enum PerfBias {
    Performance = 0,
    BalancePerformance = 4,
    Normal = 6,
    BalancePower = 8,
    Power = 15,
}

impl TryFrom<u8> for PerfBias {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PerfBias::Performance),
            4 => Ok(PerfBias::BalancePerformance),
            6 => Ok(PerfBias::Normal),
            8 => Ok(PerfBias::BalancePower),
            15 => Ok(PerfBias::Power),
            _ => Err(()),
        }
    }
}

pub fn energy_perf_bias(core: u16) -> Option<PerfBias> {
    get(&format!("cpu{core}/power/energy_perf_bias"))
        .and_then(|b| b.parse::<u8>().ok())
        .and_then(|n| PerfBias::try_from(n).ok())
}

pub fn set_energy_perf_bias(core: u16, perf_bias: PerfBias) -> Result<(), std::io::Error> {
    // supported only on intel_pstate driver
    set(
        &format!("cpu{core}/power/energy_perf_bias"),
        &(perf_bias as u8).to_string(),
    )
}

pub fn platform_profile_choices() -> Vec<String> {
    get("acpi/platform_profile_choices")
        .map(|s| s.trim().split(" ").map(String::from).collect())
        .unwrap_or_default()
}

pub fn platform_profile() -> Option<String> {
    get("acpi/platform_profile")
}

pub fn set_platform_profile(platform_profile: &str) -> Result<(), std::io::Error> {
    set("acpi/platform_profile", platform_profile)
}

pub fn no_turbo() -> Option<bool> {
    get_bool("intel_pstate/no_turbo")
}

pub fn set_no_turbo(no_turbo: bool) -> Result<(), std::io::Error> {
    set("intel_pstate/no_turbo", if no_turbo { "1" } else { "0" })
}

pub fn boost() -> Option<bool> {
    get_bool("cpufreq/boost")
}

pub fn set_boost(boost: bool) -> Result<(), std::io::Error> {
    set("cpufreq/boost", if boost { "1" } else { "0" })
}

// TODO: get_frequency_{min,max}_limit

pub fn min_perf_pct() -> Option<u8> {
    get("intel_pstate/min_perf_pct").and_then(|n| n.parse().ok())
}

pub fn set_min_perf_pct(min_perf_pct: u8) -> Result<(), std::io::Error> {
    set("intel_pstate/min_perf_pct", &min_perf_pct.to_string())
}

pub fn max_perf_pct() -> Option<u8> {
    get("intel_pstate/max_perf_pct").and_then(|n| n.parse().ok())
}

pub fn set_max_perf_pct(max_perf_pct: u8) -> Result<(), std::io::Error> {
    set("intel_pstate/max_perf_pct", &max_perf_pct.to_string())
}

pub fn online(core: u16) -> Option<bool> {
    get_bool(&format!("cpu{core}/online"))
}

pub fn set_online(core: u16, online: bool) -> Result<(), std::io::Error> {
    set(&format!("cpu{core}/online"), if online { "1" } else { "0" })
}

pub fn throttle(core: u16) -> Option<u32> {
    get(&format!("cpu{core}/thermal_throttle/core_throttle_count")).and_then(|n| n.parse().ok())
}

// TODO: throttle events

pub fn irqbalance_pid() -> Option<u32> {
    for entry in std::fs::read_dir("/proc").ok()? {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();

        if path.is_dir()
            && let Some(pid) = path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.parse::<u32>().ok())
        {
            let comm_path = path.join("comm");
            let Ok(mut f) = std::fs::File::open(comm_path) else {
                continue;
            };

            let mut buf = [0u8; "irqbalance".len()];
            let Ok(_) = f.read_exact(&mut buf) else {
                continue;
            };

            if &buf == b"irqbalance" {
                return Some(pid);
            }
        }
    }
    None
}
