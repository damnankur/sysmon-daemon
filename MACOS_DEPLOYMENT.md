# macOS Deployment Guide

SysmonDaemon is designed to run as a **privileged XPC service** on macOS 13.3+,
launched through `SMAppService` (or a `LaunchDaemon` plist). This guide covers
the production launch path. The Rust backend is identical to the Linux build —
only the transport and privilege layer differ.

## Prerequisites

- macOS 13.3 or later
- Xcode 14+ / Swift 5.8+
- An Apple Developer Team ID for code signing
- A Swift package that builds `SysmonService` as a `.xpc` bundle

## 1. Build the XPC service

```bash
# From a Swift package wrapping Swift/ and the Rust static lib
swift build -c release
```

The Rust core is compiled to a static library (`libsysmon_core.a`) and linked
into the service via a thin C bridge generated with `cbindgen`.

## 2. Register the privileged helper

Using `SMAppService` from the containing app:

```swift
import ServiceManagement

let service = SMAppService.daemon
do {
    try service.register()
} catch {
    print("Failed to register daemon: \(error)")
}
```

This installs the `.xpc` bundle under
`/Library/PrivilegedHelperTools` and starts it as root.

## 3. LaunchDaemon plist (alternative)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.damnankur.sysmon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/sysmon-daemon</string>
    </array>
    <key>KeepAlive</key>
    <true/>
    <key>RunAtLoad</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/var/log/sysmon-daemon.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/sysmon-daemon.err</string>
</dict>
</plist>
```

```bash
sudo cp com.damnankur.sysmon.plist /Library/LaunchDaemons/
sudo launchctl load /Library/LaunchDaemons/com.damnankur.sysmon.plist
```

## 4. Connect from a client

```swift
let client = XPCClient(serviceName: "com.damnankur.sysmon")
client.getThermalState { state in
    DispatchQueue.main.async { /* update UI on the main thread */ }
}
```

> XPC callbacks arrive on a background thread, so always bounce UI updates back
> to `DispatchQueue.main` (GCD) — exactly as shown in `swift/XPCClient.swift`.

## 5. Cleanup

```bash
sudo launchctl unload /Library/LaunchDaemons/com.damnankur.sysmon.plist
sudo rm /Library/LaunchDaemons/com.damnankur.sysmon.plist
sudo rm /usr/local/bin/sysmon-daemon
```

## Notes

- The daemon runs as **root** via `SMAppService` privilege escalation.
- All XPC connections are validated by **code signature**.
- Logs are written to `/var/log/sysmon-daemon.log`.
- The architecture is production-shaped: explicit privilege boundaries,
  event-driven listen loop, and RAII-style resource cleanup on both sides.
