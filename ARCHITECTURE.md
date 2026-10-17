# Architecture

SysmonDaemon is a **three-tier systems daemon** that separates the UI, the IPC
boundary, and the system-call layer. The same protocol vocabulary flows through
all three tiers, so the design is portable between macOS (XPC/Mach) and Linux
(Unix sockets).

```
┌──────────────────────────────────────┐
│  Client (SwiftUI app or CLI)         │
│  • XPCClient.swift                   │
│  • NSXPCConnection(serviceName:)     │
└──────────────────┬───────────────────┘
                   │  XPC / Mach IPC  (or Unix socket on Linux)
                   ▼
┌──────────────────────────────────────┐
│  SysmonService (Swift, root via      │
│  SMAppService on macOS)              │
│  • NSXPCListener.service()           │
│  • validates code-signed connections │
│  • forwards to backend over FFI      │
└──────────────────┬───────────────────┘
                   │  FFI / C ABI boundary
                   ▼
┌──────────────────────────────────────┐
│  Rust backend (sysmon_core)          │
│  • thermal.rs  → /sys/class/thermal  │
│  • cpu.rs      → /proc/stat          │
│  • memory.rs   → /proc/meminfo       │
│  • event-driven, never polls         │
└──────────────────────────────────────┘
```

## Design decisions

- **Protocol-first, transport-agnostic.** `SysmonDaemonProtocol` defines the
  contract; the transport (Mach XPC vs. socket) is an implementation detail.
  This is why the Rust core is fully testable on Linux.
- **Event-driven, not polling.** The service blocks on `listener.resume()`;
  the kernel wakes it only on incoming messages, so idle cost is ~0% CPU.
- **Memory safety by construction.** The Rust side uses ownership + `Arc` for
  the shared `running` shutdown flag; the Swift side releases the XPC connection
  in `deinit`. No GC, no manual `free`.
- **Privilege boundary is explicit.** Privileged ops (thermal threshold changes)
  are only honoured when the service is launched as a privileged helper.

## Platform matrix

| Concern            | Linux                                  | macOS                              |
|--------------------|----------------------------------------|------------------------------------|
| Thermal            | `/sys/class/thermal/*/temp`            | IOKit `IOGetThermalPressure` (FFI) |
| CPU                | `/proc/stat` delta                     | `host_processor_info` (Mach)       |
| Memory pressure    | `MemAvailable / MemTotal`              | `DISPATCH_SOURCE_TYPE_MEMORYPRESSURE` |
| IPC transport      | Unix socket                            | XPC / Mach                         |
| Privilege model    | systemd service                        | SMAppService LaunchDaemon          |

See `README.md` for how this maps to the five systems-engineering requirements,
and `MACOS_DEPLOYMENT.md` for the macOS launch path.
