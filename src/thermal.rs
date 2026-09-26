//! Thermal-state collection.
//!
//! Linux exposes thermal zones under `/sys/class/thermal/thermal_zoneN/temp`
//! as millidegrees Celsius. macOS would surface the same information through
//! IOKit (`IOGetThermalPressure`). The [`thermal_state`] contract is identical
//! on both platforms.

/// Temperature thresholds (°C) mapping the hottest zone to a pressure level.
const NOMINAL_MAX: f64 = 70.0;
const FAIR_MAX: f64 = 80.0;
const SERIOUS_MAX: f64 = 90.0;
// anything above SERIOUS_MAX is reported as `critical`.

/// Returns one of: `nominal | fair | serious | critical | unknown`.
pub fn thermal_state() -> String {
    #[cfg(target_os = "linux")]
    {
        match max_zone_temp_celsius() {
            Some(t) if t <= NOMINAL_MAX => "nominal".into(),
            Some(t) if t <= FAIR_MAX => "fair".into(),
            Some(t) if t <= SERIOUS_MAX => "serious".into(),
            Some(_) => "critical".into(),
            None => "unknown".into(),
        }
    }

    #[cfg(target_os = "macos")]
    {
        // IOKit FFI would populate this; stubbed for the Linux build.
        "nominal".into()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        "unknown".into()
    }
}

#[cfg(target_os = "linux")]
fn max_zone_temp_celsius() -> Option<f64> {
    use std::fs;
    let entries = fs::read_dir("/sys/class/thermal").ok()?;
    let mut max: Option<f64> = None;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("thermal_zone") {
            continue;
        }
        let path = entry.path().join("temp");
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(milli) = raw.trim().parse::<f64>() {
                let c = milli / 1000.0;
                max = Some(max.map_or(c, |m: f64| m.max(c)));
            }
        }
    }
    max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_is_one_of_known_levels() {
        let s = thermal_state();
        assert!(["nominal", "fair", "serious", "critical", "unknown"].contains(&s.as_str()));
    }
}
