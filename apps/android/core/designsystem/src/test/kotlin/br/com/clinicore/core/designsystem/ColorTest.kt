package br.com.clinicore.core.designsystem

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import com.google.common.truth.Truth.assertThat
import org.junit.Test

class ColorTest {
    private fun Color.hex() = "#%06x".format(toArgb() and 0xFFFFFF)

    @Test
    fun `the light scheme carries the tokens of docs product design`() {
        assertThat(LightColors.primary.hex()).isEqualTo("#0e7490")
        assertThat(LightColors.onPrimary.hex()).isEqualTo("#ffffff")
        assertThat(LightColors.background.hex()).isEqualTo("#f7fafa")
        assertThat(LightColors.onBackground.hex()).isEqualTo("#0f2a33")
        assertThat(LightColors.surfaceContainer.hex()).isEqualTo("#ffffff")
        assertThat(LightColors.onSurfaceVariant.hex()).isEqualTo("#4b5f68")
        assertThat(LightColors.outline.hex()).isEqualTo("#778d95")
        assertThat(LightColors.outlineVariant.hex()).isEqualTo("#d6e2e5")
        assertThat(LightColors.error.hex()).isEqualTo("#c81e1e")
    }

    @Test
    fun `the dark scheme carries the tokens of docs product design`() {
        assertThat(DarkColors.primary.hex()).isEqualTo("#22b8cf")
        assertThat(DarkColors.onPrimary.hex()).isEqualTo("#04232a")
        assertThat(DarkColors.background.hex()).isEqualTo("#0b1a1f")
        assertThat(DarkColors.onBackground.hex()).isEqualTo("#e6f1f3")
        assertThat(DarkColors.surfaceContainer.hex()).isEqualTo("#11242a")
        assertThat(DarkColors.onSurfaceVariant.hex()).isEqualTo("#9db4bb")
        assertThat(DarkColors.outline.hex()).isEqualTo("#4e7380")
        assertThat(DarkColors.outlineVariant.hex()).isEqualTo("#23404a")
        assertThat(DarkColors.error.hex()).isEqualTo("#f87171")
    }
}
