package moe.sakurajimamai.apate

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

private val LightColors = lightColorScheme(
    primary = Color(0xFF286A55),
    onPrimary = Color.White,
    primaryContainer = Color(0xFFD6F0E4),
    onPrimaryContainer = Color(0xFF08382B),
    secondary = Color(0xFF705D00),
    onSecondary = Color.White,
    secondaryContainer = Color(0xFFFFE88C),
    onSecondaryContainer = Color(0xFF342B00),
    tertiary = Color(0xFF476810),
    onTertiary = Color.White,
    background = Color(0xFFF7F8F5),
    onBackground = Color(0xFF1A1C1A),
    surface = Color(0xFFFFFFFF),
    onSurface = Color(0xFF1A1C1A),
    surfaceVariant = Color(0xFFE3E8E1),
    onSurfaceVariant = Color(0xFF434941),
    error = Color(0xFFBA1A1A),
)

private val DarkColors = darkColorScheme(
    primary = Color(0xFF8CD7BB),
    onPrimary = Color(0xFF00382A),
    primaryContainer = Color(0xFF0F503F),
    onPrimaryContainer = Color(0xFFD6F0E4),
    secondary = Color(0xFFE8CA45),
    onSecondary = Color(0xFF3B2F00),
    secondaryContainer = Color(0xFF554600),
    onSecondaryContainer = Color(0xFFFFE88C),
    tertiary = Color(0xFFB6D881),
    onTertiary = Color(0xFF213600),
    background = Color(0xFF101412),
    onBackground = Color(0xFFE0E4DE),
    surface = Color(0xFF181D1A),
    onSurface = Color(0xFFE0E4DE),
    surfaceVariant = Color(0xFF3F4944),
    onSurfaceVariant = Color(0xFFC0C9C2),
    error = Color(0xFFFFB4AB),
)

@Composable
fun ApateTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit,
) {
    MaterialTheme(
        colorScheme = if (darkTheme) DarkColors else LightColors,
        content = content,
    )
}
