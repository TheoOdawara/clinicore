package br.com.clinicore.app

import androidx.compose.ui.test.junit4.accessibility.enableAccessibilityChecks
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.tryPerformAccessibilityChecks
import androidx.test.ext.junit.runners.AndroidJUnit4
import br.com.clinicore.core.designsystem.ClinicoreTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class ClinicoreAppAccessibilityTest {
    @get:Rule val compose = createComposeRule()

    private fun passesTheAccessibilityChecks(darkTheme: Boolean) {
        compose.enableAccessibilityChecks()
        compose.setContent { ClinicoreTheme(darkTheme = darkTheme) { ClinicoreApp() } }

        compose.onRoot().tryPerformAccessibilityChecks()
    }

    @Test
    fun shellPassesTheAccessibilityChecksInTheLightTheme() =
        passesTheAccessibilityChecks(darkTheme = false)

    @Test
    fun shellPassesTheAccessibilityChecksInTheDarkTheme() =
        passesTheAccessibilityChecks(darkTheme = true)
}
