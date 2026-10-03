import Foundation

/// SwiftUI / CLI client that talks to the SysmonDaemon XPC service.
///
/// Mirrors the two-tier pattern from tony-go/XPCDemo: create a connection by
/// service name, attach the protocol interface, resume, then call methods on the
/// remote proxy. All XPC resources are released in `deinit`.
final class XPCClient {
    private let connection: NSXPCConnection
    private let service: SysmonDaemonProtocol

    init(serviceName: String = "com.damnankur.sysmon") {
        connection = NSXPCConnection(serviceName: serviceName)
        connection.remoteObjectInterface = NSXPCInterface(with: SysmonDaemonProtocol.self)
        connection.resume()

        service = connection.remoteObjectProxyWithErrorHandler { error in
            NSLog("SysmonDaemon XPC error: \(error.localizedDescription)")
        } as! SysmonDaemonProtocol
    }

    deinit {
        connection.invalidate()
    }

    func getThermalState(completion: @escaping (String) -> Void) {
        service.getThermalState(with: completion)
    }

    func getCPUUsage(completion: @escaping (Double) -> Void) {
        service.getCPUUsage(with: completion)
    }

    func getMemoryPressure(completion: @escaping (String) -> Void) {
        service.getMemoryPressure(with: completion)
    }
}
