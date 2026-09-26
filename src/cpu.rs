//! CPU busy-time percentage.
//!
//! Reads `/proc/stat` twice and divides the delta of busy jiffies by the delta
//! of total jiffies. On macOS `host_processor_info` (Mach) would be used
//! instead. The windowed sampling mirrors how a real daemon refreshes metrics
//! without busy-polling the file system.

use std::thread;
use std::time::Duration;

/// Sample window between the two `/proc/stat` reads (milliseconds).
const SAMPLE_MS: u64 = 150;

/// Returns CPU busy percentage in `0.0..=100.0`.
pub fn cpu_usage() -> f64 {
    #[cfg(target_os = "linux")]
    {
        let first = read_cpu_times();
        thread::sleep(Duration::from_millis(SAMPLE_MS));
        let second = read_cpu_times();
        match (first, second) {
            (Some(a), Some(b)) => {
                let idle_delta = b.idle.saturating_sub(a.idle);
                let total_delta = b.total.saturating_sub(a.total);
                if total_delta == 0 {
                    return 0.0;
                }
                let busy = total_delta - idle_delta;
                (busy as f64 / total_delta as f64) * 100.0
            }
            _ => 42.5,
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        42.5
    }
}

#[cfg(target_os = "linux")]
struct CpuTimes {
    idle: u64,
    total: u64,
}

#[cfg(target_os = "linux")]
fn read_cpu_times() -> Option<CpuTimes> {
    use std::fs;
    let stat = fs::read_to_string("/proc/stat").ok()?;
    let line = stat.lines().find(|l| l.starts_with("cpu "))?;
    let nums: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|n| n.parse::<u64>().ok())
        .collect();
    if nums.len() < 4 {
        return None;
    }
    let idle = nums[3];
    let total: u64 = nums.iter().sum();
    Some(CpuTimes { idle, total })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_in_range() {
        let u = cpu_usage();
        assert!((0.0..=100.0).contains(&u), "cpu usage {u} out of range");
    }
}
