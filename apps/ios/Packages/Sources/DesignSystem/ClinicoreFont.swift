import CoreText
import OSLog
import SwiftUI

public enum ClinicoreFont {
    static let family = "Atkinson Hyperlegible Next"

    private static let logger = Logger(subsystem: "br.com.clinicore", category: "DesignSystem")

    public static func register() {
        guard
            let url = Bundle.module.url(
                forResource: "AtkinsonHyperlegibleNext",
                withExtension: "ttf",
                subdirectory: "Fonts"
            )
        else {
            logger.error("The bundled font file is missing")
            return
        }
        var error: Unmanaged<CFError>?
        if !CTFontManagerRegisterFontsForURL(url as CFURL, .process, &error),
            let failure = error?.takeRetainedValue(),
            CFErrorGetCode(failure) != CTFontManagerError.alreadyRegistered.rawValue
        {
            logger.error("The font failed to register: \(failure.localizedDescription)")
        }
    }

    public static func custom(size: CGFloat, relativeTo style: Font.TextStyle) -> Font {
        .custom(family, size: size, relativeTo: style)
    }
}
