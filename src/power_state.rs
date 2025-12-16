use upower_dbus::BatteryState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PowerState {
    OnBattery,
    OnWallpower,
}
impl From<BatteryState> for PowerState {
    fn from(value: BatteryState) -> Self {
        match value {
            BatteryState::PendingDischarge
            | BatteryState::Discharging
            | BatteryState::Empty
            | BatteryState::Unknown => PowerState::OnBattery,
            BatteryState::PendingCharge | BatteryState::Charging | BatteryState::FullyCharged => {
                PowerState::OnWallpower
            }
        }
    }
}
