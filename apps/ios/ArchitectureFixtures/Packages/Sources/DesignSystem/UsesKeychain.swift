import LocalAuthentication
import Security

enum UsesKeychain {
    static let context = LAContext()

    static func clear() {
        SecItemDelete([:] as CFDictionary)
    }
}
