import DesignSystem
import SwiftUI

@main
struct ClinicoreApp: App {
    init() {
        ClinicoreFont.register()
    }

    var body: some Scene {
        WindowGroup {
            RootView()
                .tint(ClinicoreColor.primary)
        }
    }
}
