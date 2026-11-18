struct RGB: Equatable, Sendable {
    let red: Double
    let green: Double
    let blue: Double

    init(_ hex: UInt32) {
        red = Double((hex >> 16) & 0xFF) / 255
        green = Double((hex >> 8) & 0xFF) / 255
        blue = Double(hex & 0xFF) / 255
    }
}

struct ColorToken: Equatable, Sendable {
    let light: RGB
    let dark: RGB
}

enum Palette {
    static let background = ColorToken(light: RGB(0xF7FAFA), dark: RGB(0x0B1A1F))
    static let foreground = ColorToken(light: RGB(0x0F2A33), dark: RGB(0xE6F1F3))
    static let card = ColorToken(light: RGB(0xFFFFFF), dark: RGB(0x11242A))
    static let primary = ColorToken(light: RGB(0x0E7490), dark: RGB(0x22B8CF))
    static let primaryForeground = ColorToken(light: RGB(0xFFFFFF), dark: RGB(0x04232A))
    static let secondary = ColorToken(light: RGB(0xE6F0F2), dark: RGB(0x17303A))
    static let muted = ColorToken(light: RGB(0xEEF4F5), dark: RGB(0x17303A))
    static let mutedForeground = ColorToken(light: RGB(0x4B5F68), dark: RGB(0x9DB4BB))
    static let destructive = ColorToken(light: RGB(0xC81E1E), dark: RGB(0xF87171))
    static let border = ColorToken(light: RGB(0xD6E2E5), dark: RGB(0x23404A))
    static let input = ColorToken(light: RGB(0x778D95), dark: RGB(0x4E7380))
}
