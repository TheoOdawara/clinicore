// swift-tools-version: 6.4
import PackageDescription

let strict: [SwiftSetting] = [.treatAllWarnings(as: .error)]

let package = Package(
    name: "Packages",
    defaultLocalization: "pt-BR",
    platforms: [.iOS(.v26)],
    products: [
        .library(name: "DesignSystem", targets: ["DesignSystem"])
    ],
    dependencies: [
        .package(url: "https://github.com/SimplyDanny/SwiftLintPlugins", exact: "0.65.1")
    ],
    targets: [
        .target(
            name: "DesignSystem",
            resources: [.copy("Resources/Fonts")],
            swiftSettings: strict
        ),
        .testTarget(
            name: "DesignSystemTests",
            dependencies: ["DesignSystem"],
            swiftSettings: strict
        ),
    ],
    swiftLanguageModes: [.v6]
)
