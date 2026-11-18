package br.com.clinicore.core.designsystem

import androidx.compose.material3.ColorScheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.ui.graphics.Color

internal val LightColors: ColorScheme =
    lightColorScheme(
        primary = Color(0xFF0E7490),
        onPrimary = Color(0xFFFFFFFF),
        background = Color(0xFFF7FAFA),
        onBackground = Color(0xFF0F2A33),
        surface = Color(0xFFF7FAFA),
        onSurface = Color(0xFF0F2A33),
        onSurfaceVariant = Color(0xFF4B5F68),
        surfaceContainerLowest = Color(0xFFFFFFFF),
        surfaceContainerLow = Color(0xFFFFFFFF),
        surfaceContainer = Color(0xFFFFFFFF),
        surfaceContainerHigh = Color(0xFFFFFFFF),
        surfaceContainerHighest = Color(0xFFEEF4F5),
        secondaryContainer = Color(0xFFE6F0F2),
        onSecondaryContainer = Color(0xFF0F2A33),
        outline = Color(0xFF778D95),
        outlineVariant = Color(0xFFD6E2E5),
        error = Color(0xFFC81E1E),
    )

internal val DarkColors: ColorScheme =
    darkColorScheme(
        primary = Color(0xFF22B8CF),
        onPrimary = Color(0xFF04232A),
        background = Color(0xFF0B1A1F),
        onBackground = Color(0xFFE6F1F3),
        surface = Color(0xFF0B1A1F),
        onSurface = Color(0xFFE6F1F3),
        onSurfaceVariant = Color(0xFF9DB4BB),
        surfaceContainerLowest = Color(0xFF11242A),
        surfaceContainerLow = Color(0xFF11242A),
        surfaceContainer = Color(0xFF11242A),
        surfaceContainerHigh = Color(0xFF11242A),
        surfaceContainerHighest = Color(0xFF17303A),
        secondaryContainer = Color(0xFF17303A),
        onSecondaryContainer = Color(0xFFE6F1F3),
        outline = Color(0xFF4E7380),
        outlineVariant = Color(0xFF23404A),
        error = Color(0xFFF87171),
    )
