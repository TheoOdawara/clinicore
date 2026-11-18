import SwiftUI
import UIKit

public enum ClinicoreColor {
    public static let background = Color(Palette.background)
    public static let primary = Color(Palette.primary)
}

extension Color {
    init(_ token: ColorToken) {
        self.init(
            uiColor: UIColor { traits in
                let rgb = traits.userInterfaceStyle == .dark ? token.dark : token.light
                return UIColor(red: rgb.red, green: rgb.green, blue: rgb.blue, alpha: 1)
            }
        )
    }
}
