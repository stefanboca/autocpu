use autocpu::cpuinfo::{
    available_frequencies, available_governors, boost, driver, energy_perf_bias,
    energy_performance_available_preferences, energy_performance_preference, frequency, governor,
    irqbalance_pid, max_frequency, max_perf_pct, min_frequency, min_perf_pct, no_turbo, online,
    platform_profile, platform_profile_choices, throttle,
};

fn main() {
    dbg!(driver());
    dbg!(available_governors());
    dbg!(governor(0));
    dbg!(available_frequencies());
    dbg!(frequency(0));
    dbg!(min_frequency(0));
    dbg!(max_frequency(0));
    dbg!(energy_performance_available_preferences());
    dbg!(energy_performance_preference(0));
    dbg!(energy_perf_bias(0));
    dbg!(platform_profile_choices());
    dbg!(platform_profile());
    dbg!(no_turbo());
    dbg!(boost());
    dbg!(min_perf_pct());
    dbg!(max_perf_pct());
    dbg!(online(1));
    dbg!(throttle(0));
    dbg!(irqbalance_pid());
}
