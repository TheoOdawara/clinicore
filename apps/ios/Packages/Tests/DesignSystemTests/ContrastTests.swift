import Foundation
import Testing

@testable import DesignSystem

struct ContrastTests {
    enum Scheme: CaseIterable {
        case light, dark
    }

    struct Pair {
        let name: String
        let foreground: ColorToken
        let background: ColorToken

        init(_ name: String, _ foreground: ColorToken, _ background: ColorToken) {
            self.name = name
            self.foreground = foreground
            self.background = background
        }
    }

    private static let textPairs: [Pair] = [
        Pair("foreground on background", Palette.foreground, Palette.background),
        Pair("foreground on card", Palette.foreground, Palette.card),
        Pair("foreground on muted", Palette.foreground, Palette.muted),
        Pair("mutedForeground on background", Palette.mutedForeground, Palette.background),
        Pair("mutedForeground on card", Palette.mutedForeground, Palette.card),
        Pair("mutedForeground on muted", Palette.mutedForeground, Palette.muted),
        Pair("primary on background", Palette.primary, Palette.background),
        Pair("primary on card", Palette.primary, Palette.card),
        Pair("primaryForeground on primary", Palette.primaryForeground, Palette.primary),
        Pair("foreground on secondary", Palette.foreground, Palette.secondary),
        Pair("destructive on background", Palette.destructive, Palette.background),
        Pair("destructive on card", Palette.destructive, Palette.card),
    ]

    private static let borderPairs: [Pair] = [
        Pair("input on background", Palette.input, Palette.background),
        Pair("input on card", Palette.input, Palette.card),
    ]

    private func luminance(_ rgb: RGB) -> Double {
        func linear(_ channel: Double) -> Double {
            channel <= 0.04045 ? channel / 12.92 : pow((channel + 0.055) / 1.055, 2.4)
        }
        return 0.2126 * linear(rgb.red) + 0.7152 * linear(rgb.green) + 0.0722 * linear(rgb.blue)
    }

    private func contrast(_ foreground: RGB, _ background: RGB) -> Double {
        let lighter = max(luminance(foreground), luminance(background))
        let darker = min(luminance(foreground), luminance(background))
        return (lighter + 0.05) / (darker + 0.05)
    }

    private func pick(_ token: ColorToken, _ scheme: Scheme) -> RGB {
        scheme == .light ? token.light : token.dark
    }

    @Test(arguments: Scheme.allCases)
    func `every text pair meets WCAG AA`(scheme: Scheme) {
        for pair in Self.textPairs {
            let ratio = contrast(pick(pair.foreground, scheme), pick(pair.background, scheme))
            #expect(ratio >= 4.5, "\(pair.name) is \(ratio)")
        }
    }

    @Test(arguments: Scheme.allCases)
    func `every border pair meets WCAG AA for non-text`(scheme: Scheme) {
        for pair in Self.borderPairs {
            let ratio = contrast(pick(pair.foreground, scheme), pick(pair.background, scheme))
            #expect(ratio >= 3.0, "\(pair.name) is \(ratio)")
        }
    }
}
