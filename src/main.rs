//! SysmonDaemon binary — prints a one-shot system snapshot.
//!
//! In production this would `resume()` an XPC/Mach listener (macOS) or a Unix
//! socket acceptor (Linux) and answer IPC requests from a client. The local
//! build prints metrics so the architecture is verifiable without Mac hardware.

use sysmon_core::SysmonBackend;

fn main() {
    println!("SysmonDaemon — cross-platform IPC monitoring daemon");
    println!("=====================================================");

    let backend = SysmonBackend::new();
    let snap = backend.snapshot();

    println!("\nSystem snapshot:");
    println!("  thermal : {}", snap.thermal);
    println!("  cpu     : {:.1}%", snap.cpu);
    println!("  memory  : {}", snap.memory);

    println!("\nArchitecture: a Rust backend answers IPC requests from a");
    println!("Swift XPC service (macOS) or a socket client (Linux).");
    println!("See README.md and MACOS_DEPLOYMENT.md for the full design.");
}
