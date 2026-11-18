import XCTest

@MainActor
final class ShellTests: XCTestCase {
    override func setUp() async throws {
        continueAfterFailure = false
    }

    override func tearDown() async throws {
        XCUIDevice.shared.appearance = .light
    }

    private func launchShell(arguments: [String] = []) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchArguments = arguments
        app.launch()
        XCTAssertTrue(app.staticTexts["Clinicore"].waitForExistence(timeout: 10))
        return app
    }

    func testShellPassesTheAccessibilityAuditInTheLightAppearance() throws {
        XCUIDevice.shared.appearance = .light

        let app = launchShell()

        try app.performAccessibilityAudit()
    }

    func testShellPassesTheAccessibilityAuditInTheDarkAppearance() throws {
        XCUIDevice.shared.appearance = .dark

        let app = launchShell()

        try app.performAccessibilityAudit()
    }

    func testShellPassesTheAccessibilityAuditAtTheLargestTextSize() throws {
        let app = launchShell(arguments: [
            "-UIPreferredContentSizeCategoryName", "UICTContentSizeCategoryAccessibilityXXXL",
        ])

        try app.performAccessibilityAudit()
    }
}
