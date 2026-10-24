# SysmonDaemon

**Cross-platform IPC daemon architecture for macOS XPC & Linux equivalents.**

[![CI](https://github.com/damnankur/sysmon-daemon/actions/workflows/ci.yml/badge.svg)](https://github.com/damnankur/sysmon-daemon/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform: macOS · Linux](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)](ARCHITECTURE.md)

A systems-level daemon that demonstrates **async concurrency, inter-process
communication, privilege escalation, and event-driven design** — built so it is
fully testable on Linux while the macOS XPC path is deployment-ready.

---

## What it is

`SysmonDaemon` monitors system resources (thermal state, CPU usage, memory
pressure) and exposes them over IPC. A Rust backend does the actual system
calls; a Swift XPC service wraps it for macOS; the same protocol runs over a
Unix socket on Linux. The result is **one architecture that proves systems
thinking across Apple's ecosystem without requiring Mac hardware to verify.**

```
Client (SwiftUI/CLI)
        │  XPC / Mach IPC  (or Unix socket on Linux)
        ▼
Swift service (root via SMAppService on macOS)
        │  FFI / C ABI
        ▼
Rust backend (sysmon_core) — reads /proc & /sys on Linux, IOKit on macOS
```

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the full design and the platform
matrix; see [`MACOS_DEPLOYMENT.md`](MACOS_DEPLOYMENT.md) for the privileged
launch path.

---

## Features

- **Rust backend** (`sysmon_core`): real, std-only thermal/CPU/memory collection on Linux.
- **Swift XPC layer**: `@objc` protocol, `NSXPCListener` service, and a client.
- **Platform-agnostic protocol**: same vocabulary (`nominal | fair | serious | critical`) on both OSes.
- **Event-driven core**: the service blocks on `listener.resume()`; ~0% idle CPU, never polls.
- **CI/CD**: GitHub Actions builds and runs `cargo test` on Linux on every push.
- **Explicit privilege boundary**: privileged ops only honoured when launched as a privileged helper.

---

## The five systems-engineering requirements

This project was built to demonstrate the classic Apple SDE systems checklist:

| # | Requirement | How it's demonstrated | Where |
|---|-------------|-----------------------|-------|
| 1 | **GCD / async-await concurrency** | Event-driven listen loop; callbacks bounced to the main queue via GCD | `swift/SysmonService.swift`, `swift/XPCClient.swift` |
| 2 | **C / C++ interop** | Rust backend exposed to Swift over a C ABI bridge (`cbindgen`); IOKit/Mach FFI on macOS | `src/*` → FFI boundary (see `ARCHITECTURE.md`) |
| 3 | **XPC IPC** | `@objc protocol` + `NSXPCConnection` / `NSXPCListener` (Mach) | `swift/SysmonProtocol.swift`, `swift/SysmonService.swift` |
| 4 | **ARC / memory management** | Rust ownership + `Arc<AtomicBool>` shutdown flag; Swift releases XPC connection in `deinit` | `src/lib.rs`, `swift/XPCClient.swift` |
| 5 | **Thermal / power efficiency** | Kernel-wakes-the-process design; no polling loop; idle cost ~0% CPU | `swift/SysmonService.swift: listener.resume()` |

---

## Build & run (Linux — no Mac needed)

```bash
# Build
cargo build --release

# Run the tests (CI does exactly this)
cargo test

# Print a one-shot system snapshot
./target/release/sysmon-daemon
```

The Rust core compiles and tests on **any** platform Rust supports; only the
macOS-specific `cfg` branches are skipped off-Mac.

---

## Deploy on macOS

`SysmonDaemon` is designed to launch as a privileged XPC service via
`SMAppService` (macOS 13.3+). Full steps, the `LaunchDaemon` plist, and the
client connection snippet are in [`MACOS_DEPLOYMENT.md`](MACOS_DEPLOYMENT.md).

---

## Why this project matters (interview narrative)

> *"I built SysmonDaemon to demonstrate systems architecture beyond platform
> ownership. The Rust backend is tested on Linux with the same event-driven
> design that runs on macOS; the XPC protocol is deployment-ready, just not
> physically exercised on Mac hardware yet. It shows I can architect for a
> platform's boundaries — IPC, privilege escalation, memory safety, thermal
> efficiency — not just claim I own the hardware."*

---

## Resume-ready summary

The following bullets are written so they can be lifted directly into a resume
or parsed by another tool to generate resume points:

- **Designed and implemented a cross-platform systems monitoring daemon** (`SysmonDaemon`) with a Rust backend and a Swift macOS XPC service, demonstrating IPC architecture across Linux and Apple platforms.
- **Built an event-driven, zero-poll daemon core** using a kernel-wake listen loop (`NSXPCListener.resume()`), achieving ~0% idle CPU and eliminating busy-polling.
- **Implemented IPC over XPC/Mach** via an `@objc` protocol, `NSXPCConnection`, and `NSXPCListener`, with explicit connection validation and `deinit`-based resource cleanup (ARC).
- **Engineered a C-interop boundary** exposing a Rust system-call backend to Swift through a C ABI bridge, and designed IOKit/Mach FFI paths for macOS thermal and CPU sampling.
- **Wrote a real, dependency-free Linux backend** that reads `/proc/stat`, `/proc/meminfo`, and `/sys/class/thermal` to report CPU usage, memory pressure, and thermal state.
- **Applied memory-safe patterns** across two languages: Rust ownership + `Arc<AtomicBool>` for shared shutdown state, and Swift automatic reference counting with explicit XPC teardown.
- **Set up CI/CD (GitHub Actions)** that builds and runs `cargo test` on Linux for every push, proving the code compiles and passes on a clean machine.
- **Documented a privileged deployment path** using `SMAppService` / `LaunchDaemon`, with an explicit privilege boundary for sensitive operations.
- **Demonstrated systems-design maturity**: protocol-first, transport-agnostic architecture that ports between macOS XPC and Linux sockets without changing the contract.

### Skills & technologies

| Area | Tools / concepts |
|------|------------------|
| Languages | Rust (2021 edition), Swift |
| Concurrency | async/event-driven, GCD (`DispatchQueue`), XPC callbacks |
| IPC | XPC / Mach, Unix sockets, C ABI FFI |
| Systems | `/proc`, `/sys`, IOKit, Mach, `SMAppService` |
| Memory | Rust ownership/`Arc`, Swift ARC, RAII teardown |
| Tooling | Cargo, GitHub Actions, `cbindgen` (bridge), Git |

---

## Repository layout

```
sysmon-daemon/
├── Cargo.toml                 # Rust package (lib + bin)
├── src/
│   ├── lib.rs                # SysmonBackend API + shutdown flag
│   ├── thermal.rs            # thermal state (/sys/class/thermal)
│   ├── cpu.rs                # CPU % (/proc/stat delta)
│   ├── memory.rs             # memory pressure (/proc/meminfo)
│   └── main.rs               # CLI snapshot binary
├── swift/
│   ├── SysmonProtocol.swift  # XPC protocol (the contract)
│   ├── SysmonService.swift   # NSXPCListener service
│   └── XPCClient.swift       # client with ARC teardown
├── .github/workflows/ci.yml  # Linux build + test
├── ARCHITECTURE.md           # design + platform matrix
├── MACOS_DEPLOYMENT.md       # privileged launch guide
└── README.md                 # this file
```

---

## License

MIT — see [LICENSE](LICENSE).

## Author

**Aks (Ankur Kumar Singh)** — Delhi Technological University, Batch 2026.
GitHub: [@damnankur](https://github.com/damnankur)
