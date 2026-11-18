import CoreText
import Testing
import UIKit

@testable import DesignSystem

struct ClinicoreFontTests {
    private func weight(of font: UIFont) -> Double? {
        let variation = CTFontCopyVariation(font) as? [NSNumber: NSNumber]
        let weightAxisTag = NSNumber(value: UInt32(0x7767_6874))
        return variation?[weightAxisTag]?.doubleValue
    }

    @Test func `the bundled family resolves once registered`() throws {
        ClinicoreFont.register()
        ClinicoreFont.register()

        let font = try #require(UIFont(name: "AtkinsonHyperlegibleNext-Regular", size: 17))

        #expect(font.familyName == ClinicoreFont.family)
    }

    @Test func `the semibold weight comes from the variable axis`() throws {
        ClinicoreFont.register()
        let descriptor = UIFontDescriptor(fontAttributes: [.family: ClinicoreFont.family])
            .addingAttributes([.traits: [UIFontDescriptor.TraitKey.weight: UIFont.Weight.semibold]])

        let font = UIFont(descriptor: descriptor, size: 17)

        #expect(font.familyName == ClinicoreFont.family)
        #expect(try #require(weight(of: font)) == 600)
    }
}
