package br.com.clinicore.core.designsystem

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

internal val AtkinsonHyperlegibleNext =
    FontFamily(
        Font(R.font.atkinson_hyperlegible_next_regular, FontWeight.Normal),
        Font(R.font.atkinson_hyperlegible_next_medium, FontWeight.Medium),
        Font(R.font.atkinson_hyperlegible_next_semibold, FontWeight.SemiBold),
        Font(R.font.atkinson_hyperlegible_next_bold, FontWeight.Bold),
    )

private val Default = Typography()

private fun TextStyle.inFamily() = copy(fontFamily = AtkinsonHyperlegibleNext)

internal val ClinicoreTypography =
    Typography(
        displayLarge = Default.displayLarge.inFamily(),
        displayMedium = Default.displayMedium.inFamily(),
        displaySmall = Default.displaySmall.inFamily(),
        headlineLarge = Default.headlineLarge.inFamily(),
        headlineMedium = Default.headlineMedium.inFamily(),
        headlineSmall =
            Default.headlineSmall
                .inFamily()
                .copy(fontSize = 24.sp, fontWeight = FontWeight.SemiBold),
        titleLarge = Default.titleLarge.inFamily(),
        titleMedium = Default.titleMedium.inFamily(),
        titleSmall = Default.titleSmall.inFamily(),
        bodyLarge = Default.bodyLarge.inFamily().copy(fontSize = 16.sp),
        bodyMedium = Default.bodyMedium.inFamily(),
        bodySmall = Default.bodySmall.inFamily(),
        labelLarge = Default.labelLarge.inFamily(),
        labelMedium =
            Default.labelMedium.inFamily().copy(fontSize = 14.sp, fontWeight = FontWeight.SemiBold),
        labelSmall = Default.labelSmall.inFamily(),
    )
