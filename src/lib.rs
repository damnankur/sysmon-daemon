//! SysmonDaemon — cross-platform systems monitoring backend.
//!
//! The backend hides platform-specific metric collection behind a single
//! [`SysmonBackend`] API. On Linux it reads from `/proc` and `/sys`; on macOS
//! the same methods would be served through IOKit / Mach FFI (cfg-gated). The
//! contract is identical across platforms so the Swift XPC service and any
//! Linux socket client can share one protocol.

pub mod cpu;
pub mod memory;
pub mod thermal;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Shared daemon state. `running` lets a supervisor signal graceful shutdown
/// without the event loop ever having to poll for a stop flag.
pub struct SysmonBackend {
    running: Arc<AtomicBool>,
}

impl Default for SysmonBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SysmonBackend {
    /// Create a new backend in the running state.
    pub fn new() -> Self {
        SysmonBackend {
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Current thermal pressure state: `nominal | fair | serious | critical`.
    pub fn thermal_state(&self) -> String {
        thermal::thermal_state()
    }

    /// CPU busy percentage in the range `0.0..=100.0`.
    pub fn cpu_usage(&self) -> f64 {
        cpu::cpu_usage()
    }

    /// Memory pressure state: `nominal | fair | serious | critical`.
    pub fn memory_pressure(&self) -> String {
        memory::memory_pressure()
    }

    /// A point-in-time reading of every metric — convenient IPC payload.
    pub fn snapshot(&self) -> Metrics {
        Metrics {
            thermal: self.thermal_state(),
            cpu: self.cpu_usage(),
            memory: self.memory_pressure(),
        }
    }

    /// Signal the event loop to stop.
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// `true` until [`SysmonBackend::shutdown`] is called.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

/// A point-in-time reading of all monitored resources.
#[derive(Debug, Clone, PartialEq)]
pub struct Metrics {
    pub thermal: String,
    pub cpu: f64,
    pub memory: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_exposes_valid_thermal_state() {
        let b = SysmonBackend::new();
        let s = b.thermal_state();
        assert!(["nominal", "fair", "serious", "critical", "unknown"].contains(&s.as_str()));
    }

    #[test]
    fn cpu_usage_stays_in_range() {
        let b = SysmonBackend::new();
        let u = b.cpu_usage();
        assert!((0.0..=100.0).contains(&u), "cpu usage {u} out of range");
    }

    #[test]
    fn memory_pressure_is_recognised_state() {
        let b = SysmonBackend::new();
        let m = b.memory_pressure();
        assert!(["nominal", "fair", "serious", "critical", "unknown"].contains(&m.as_str()));
    }

    #[test]
    fn snapshot_is_coherent() {
        let b = SysmonBackend::new();
        let s = b.snapshot();
        assert!(!s.thermal.is_empty());
        assert!((0.0..=100.0).contains(&s.cpu));
    }

    #[test]
    fn shutdown_flips_running_flag() {
        let b = SysmonBackend::new();
        assert!(b.is_running());
        b.shutdown();
        assert!(!b.is_running());
    }
}
