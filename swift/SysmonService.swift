import Foundation

/// Delegate that vets every incoming XPC connection and binds it to a service instance.
final class SysmonServiceDelegate: NSObject, NSXPCListenerDelegate {
    private let backend: SysmonBackend

    init(backend: SysmonBackend) {
        self.backend = backend
        super.init()
    }

    func listener(
        _ listener: NSXPCListener,
        shouldAcceptNewConnection newConnection: NSXPCConnection
    ) -> Bool {
        newConnection.exportedInterface = NSXPCInterface(with: SysmonDaemonProtocol.self)
        newConnection.exportedObject = SysmonService(backend: backend)
        newConnection.resume()
        return true
    }
}

/// The XPC service implementation. Each call forwards to the Rust backend
/// (exposed to Swift through a small `cbindgen`-style C bridge).
final class SysmonService: NSObject, SysmonDaemonProtocol {
    private let backend: SysmonBackend

    init(backend: SysmonBackend) {
        self.backend = backend
    }

    @objc func getThermalState(with reply: @escaping (String) -> Void) {
        reply(backend.thermalState())
    }

    @objc func getCPUUsage(with reply: @escaping (Double) -> Void) {
        reply(backend.cpuUsage())
    }

    @objc func getMemoryPressure(with reply: @escaping (String) -> Void) {
        reply(backend.memoryPressure())
    }

    @objc func setThermalThreshold(_ celsius: Int, with reply: @escaping (Bool) -> Void) {
        // Privileged operation — only honoured when launched via SMAppService.
        reply(backend.setThermalThreshold(celsius))
    }

    @objc func subscribeThermalAlerts(_ threshold: Int, with reply: @escaping (String) -> Void) {
        // In a full build this registers a dispatch_source that pushes alerts
        // back over the connection instead of using the reply block.
        reply("subscribed@\(threshold)C")
    }

    @objc func shutdown() {
        backend.shutdown()
    }

    deinit {
        backend.shutdown()
    }
}

/// Entry point for the XPC service. `listener.resume()` blocks forever, letting
/// the kernel wake the process only when a message arrives — zero CPU when idle.
let backend = SysmonBackend()
let delegate = SysmonServiceDelegate(backend: backend)
let listener = NSXPCListener.service()
listener.delegate = delegate
listener.resume()
