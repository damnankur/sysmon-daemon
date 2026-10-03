import Foundation

/// XPC protocol for SysmonDaemon.
///
/// Marked `@objc` so the Objective-C runtime can serialise method calls across
/// the process boundary. The Rust backend implements the same vocabulary
/// (`nominal | fair | serious | critical`), so the protocol is transport-agnostic:
/// it works over Mach XPC on macOS and over a Unix socket on Linux.
@objc protocol SysmonDaemonProtocol {
    /// Current thermal state: `nominal | fair | serious | critical`.
    func getThermalState(with reply: @escaping (String) -> Void)

    /// CPU usage as a percentage (0–100).
    func getCPUUsage(with reply: @escaping (Double) -> Void)

    /// Memory pressure state: `nominal | fair | serious | critical`.
    func getMemoryPressure(with reply: @escaping (String) -> Void)

    /// Request a privileged change to the thermal warning threshold (°C).
    func setThermalThreshold(_ celsius: Int, with reply: @escaping (Bool) -> Void)

    /// Subscribe to asynchronous thermal alerts when the system crosses `threshold` (°C).
    func subscribeThermalAlerts(_ threshold: Int, with reply: @escaping (String) -> Void)

    /// Shut the daemon down gracefully.
    func shutdown()
}

/*
 USAGE ON macOS:

 let connection = NSXPCConnection(serviceName: "com.damnankur.sysmon")
 connection.remoteObjectInterface = NSXPCInterface(with: SysmonDaemonProtocol.self)
 connection.resume()

 if let daemon = connection.remoteObjectProxy as? SysmonDaemonProtocol {
     daemon.getThermalState { state in
         print("Thermal: \(state)")
     }
 }
*/
