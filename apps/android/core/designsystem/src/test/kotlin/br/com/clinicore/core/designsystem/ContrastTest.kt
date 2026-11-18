package br.com.clinicore.core.designsystem

import androidx.compose.material3.ColorScheme
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import com.google.common.truth.Truth.assertWithMessage
import org.junit.Test

class ContrastTest {
    private val textMinimum = 4.5
    private val nonTextMinimum = 3.0

    private fun contrast(foreground: Color, background: Color): Double {
        val lighter = maxOf(foreground.luminance(), background.luminance())
        val darker = minOf(foreground.luminance(), background.luminance())
        return (lighter + 0.05) / (darker + 0.05)
    }

    private fun ColorScheme.textPairs() =
        mapOf(
            "onSurface on surface" to (onSurface to surface),
            "onSurface on surfaceContainer" to (onSurface to surfaceContainer),
            "onSurface on surfaceContainerHighest" to (onSurface to surfaceContainerHighest),
            "onSurfaceVariant on surface" to (onSurfaceVariant to surface),
            "onSurfaceVariant on surfaceContainer" to (onSurfaceVariant to surfaceContainer),
            "onSurfaceVariant on surfaceContainerHighest" to
                (onSurfaceVariant to surfaceContainerHighest),
            "primary on surface" to (primary to surface),
            "primary on surfaceContainer" to (primary to surfaceContainer),
            "onPrimary on primary" to (onPrimary to primary),
            "onSecondaryContainer on secondaryContainer" to
                (onSecondaryContainer to secondaryContainer),
            "error on surface" to (error to surface),
            "error on surfaceContainer" to (error to surfaceContainer),
        )

    private fun ColorScheme.nonTextPairs() =
        mapOf(
            "outline on surface" to (outline to surface),
            "outline on surfaceContainer" to (outline to surfaceContainer),
        )

    private fun assertReadable(scheme: ColorScheme) {
        scheme.textPairs().forEach { (pair, colors) ->
            assertWithMessage(pair)
                .that(contrast(colors.first, colors.second))
                .isAtLeast(textMinimum)
        }
        scheme.nonTextPairs().forEach { (pair, colors) ->
            assertWithMessage(pair)
                .that(contrast(colors.first, colors.second))
                .isAtLeast(nonTextMinimum)
        }
    }

    @Test
    fun `every text and border pair of the light scheme meets WCAG AA`() =
        assertReadable(LightColors)

    @Test
    fun `every text and border pair of the dark scheme meets WCAG AA`() = assertReadable(DarkColors)
}
