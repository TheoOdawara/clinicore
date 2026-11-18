import Testing

@testable import DesignSystem

struct PaletteTests {
    private func hex(_ rgb: RGB) -> String {
        let channels = [rgb.red, rgb.green, rgb.blue].map { Int(($0 * 255).rounded()) }
        return "#" + channels.map { String(format: "%02x", $0) }.joined()
    }

    @Test func `the light palette carries the tokens of docs product design`() {
        #expect(hex(Palette.background.light) == "#f7fafa")
        #expect(hex(Palette.foreground.light) == "#0f2a33")
        #expect(hex(Palette.card.light) == "#ffffff")
        #expect(hex(Palette.primary.light) == "#0e7490")
        #expect(hex(Palette.primaryForeground.light) == "#ffffff")
        #expect(hex(Palette.secondary.light) == "#e6f0f2")
        #expect(hex(Palette.muted.light) == "#eef4f5")
        #expect(hex(Palette.mutedForeground.light) == "#4b5f68")
        #expect(hex(Palette.destructive.light) == "#c81e1e")
        #expect(hex(Palette.border.light) == "#d6e2e5")
        #expect(hex(Palette.input.light) == "#778d95")
    }

    @Test func `the dark palette carries the tokens of docs product design`() {
        #expect(hex(Palette.background.dark) == "#0b1a1f")
        #expect(hex(Palette.foreground.dark) == "#e6f1f3")
        #expect(hex(Palette.card.dark) == "#11242a")
        #expect(hex(Palette.primary.dark) == "#22b8cf")
        #expect(hex(Palette.primaryForeground.dark) == "#04232a")
        #expect(hex(Palette.secondary.dark) == "#17303a")
        #expect(hex(Palette.muted.dark) == "#17303a")
        #expect(hex(Palette.mutedForeground.dark) == "#9db4bb")
        #expect(hex(Palette.destructive.dark) == "#f87171")
        #expect(hex(Palette.border.dark) == "#23404a")
        #expect(hex(Palette.input.dark) == "#4e7380")
    }
}
