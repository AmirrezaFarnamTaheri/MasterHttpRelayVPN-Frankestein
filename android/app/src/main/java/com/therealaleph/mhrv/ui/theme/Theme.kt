package com.therealaleph.mhrv.ui.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/**
 * Visual theme aligned with the desktop `mhrv-f-ui` egui app: same dark
 * palette family and corner radii. Desktop accent values live in
 * `src/bin/ui.rs`; Android uses the same hues in `Color(0xAARRGGBB)`.
 *
 * Deliberate choices:
 *   - ALWAYS dark. The desktop UI is always dark (`egui::Visuals::dark()`),
 *     so Android follows. Neither light mode nor Android 12+ dynamic color
 *     is respected — matching the desktop trumps blending with the user's
 *     wallpaper here.
 *   - Card corners 6.dp, button corners 4.dp, matching the eframe
 *     `.rounding(6.0)` / `.rounding(4.0)` pairs in the desktop code.
 */

// Palette aligned with src/bin/ui.rs (desktop egui accents).
val AccentBlue = Color(0xFF5294D4)
val AccentHover = Color(0xFF6CA8E4)
// OK_GREEN / ERR_RED (see src/bin/ui.rs)
val OkGreen = Color(0xFF48BC6C)
val ErrRed = Color(0xFFE67676)

// Card fill and stroke used by section containers in the desktop UI.
val CardFill = Color(0xFF1C1E22)
val CardStroke = Color(0xFF32363C)

// Backdrop slightly darker than cards so containers pop off the page —
// egui's default dark background sits right around this value.
val BgDark = Color(0xFF111317)

// Text shades — `egui::Color32::from_gray(200)` etc.
val TextPrimary = Color(0xFFC8C8C8)
val TextSecondary = Color(0xFF8C8C8C)
val TextLabel = Color(0xFFB4B4B4)

private val MhrvDark = darkColorScheme(
    primary = AccentBlue,
    onPrimary = Color.White,
    primaryContainer = AccentHover,
    onPrimaryContainer = Color.White,

    secondary = OkGreen,
    onSecondary = Color.Black,

    tertiary = OkGreen,
    onTertiary = Color.Black,

    error = ErrRed,
    onError = Color.White,

    background = BgDark,
    onBackground = TextPrimary,

    surface = CardFill,
    onSurface = TextPrimary,

    surfaceVariant = Color(0xFF232933),
    onSurfaceVariant = TextSecondary,

    outline = CardStroke,
    outlineVariant = CardStroke,
)

/**
 * Material3 consumes Shapes through component defaults (Button uses
 * `shapes.full`, Card uses `shapes.medium`, etc.). Mapping every size to
 * tight rounded-rectangles keeps the whole app visually consistent with
 * the desktop's squared-off controls instead of Material's default pills.
 */
private val MhrvShapes = Shapes(
    extraSmall = RoundedCornerShape(4.dp),
    small = RoundedCornerShape(4.dp),
    medium = RoundedCornerShape(8.dp),
    large = RoundedCornerShape(10.dp),
    extraLarge = RoundedCornerShape(12.dp),
)

private val BaseTypography = Typography()

/** Slightly more open line heights and clearer hierarchy than Material defaults. */
private val MhrvTypography = BaseTypography.copy(
    titleLarge = BaseTypography.titleLarge.copy(
        fontWeight = FontWeight.SemiBold,
        lineHeight = 30.sp,
    ),
    titleMedium = BaseTypography.titleMedium.copy(
        fontWeight = FontWeight.SemiBold,
        lineHeight = 26.sp,
    ),
    titleSmall = BaseTypography.titleSmall.copy(
        fontWeight = FontWeight.SemiBold,
        lineHeight = 22.sp,
    ),
    bodyLarge = BaseTypography.bodyLarge.copy(lineHeight = 24.sp),
    bodyMedium = BaseTypography.bodyMedium.copy(lineHeight = 22.sp),
    bodySmall = BaseTypography.bodySmall.copy(lineHeight = 18.sp),
    labelMedium = BaseTypography.labelMedium.copy(
        fontWeight = FontWeight.Medium,
        letterSpacing = 0.2.sp,
    ),
)

@Composable
fun MhrvTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = MhrvDark,
        shapes = MhrvShapes,
        typography = MhrvTypography,
        content = content,
    )
}
