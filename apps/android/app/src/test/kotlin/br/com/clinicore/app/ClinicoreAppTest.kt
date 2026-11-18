package br.com.clinicore.app

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isHeading
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import br.com.clinicore.core.designsystem.ClinicoreTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config

@RunWith(AndroidJUnit4::class)
@Config(qualifiers = "w360dp-h640dp")
class ClinicoreAppTest {
    @get:Rule val compose = createComposeRule()

    private fun showsTheProductNameAsAHeading(darkTheme: Boolean) {
        compose.setContent { ClinicoreTheme(darkTheme = darkTheme) { ClinicoreApp() } }

        compose.onNode(hasText("Clinicore") and isHeading()).assertIsDisplayed()
    }

    @Test
    fun `light theme shows the product name as a heading`() =
        showsTheProductNameAsAHeading(darkTheme = false)

    @Test
    fun `dark theme shows the product name as a heading`() =
        showsTheProductNameAsAHeading(darkTheme = true)
}
