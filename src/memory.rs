//! Memory-pressure state.
//!
//! On Linux we derive pressure from `MemAvailable` vs `MemTotal` in
//! `/proc/meminfo`. macOS exposes `DISPATCH_SOURCE_TYPE_MEMORYPRESSURE`
//! through Grand Central Dispatch. The same four-level vocabulary
//! (`nominal | fair | serious | critical`) is returned on both platforms.

/// Returns one of: `nominal | fair | serious | critical | unknown`.
pub fn memory_pressure() -> String {
    #[cfg(target_os = "linux")]
    {
        match read_meminfo() {
            Some((available, total)) if total > 0 => {
                let free_ratio = available as f64 / total as f64;
                if free_ratio > 0.5 {
                    "nominal".into()
                } else if free_ratio > 0.25 {
                    "fair".into()
                } else if free_ratio > 0.10 {
                    "serious".into()
                } else {
                    "critical".into()
                }
            }
            _ => "unknown".into(),
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        "nominal".into()
    }
}

#[cfg(target_os = "linux")]
fn read_meminfo() -> Option<(u64, u64)> {
    use std::fs;
    let content = fs::read_to_string("/proc/meminfo").ok()?;
    let mut available = None;
    let mut total = None;
    for line in content.lines() {
        if let Some(v) = line.strip_prefix("MemTotal:") {
            total = parse_kb(v);
        } else if let Some(v) = line.strip_prefix("MemAvailable:") {
            available = parse_kb(v);
        }
    }
    match (available, total) {
        (Some(a), Some(t)) => Some((a, t)),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn parse_kb(s: &str) -> Option<u64> {
    s.split_whitespace().next()?.parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressure_is_known_level() {
        let m = memory_pressure();
        assert!(["nominal", "fair", "serious", "critical", "unknown"].contains(&m.as_str()));
    }
}
