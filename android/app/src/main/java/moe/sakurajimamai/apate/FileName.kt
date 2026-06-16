package moe.sakurajimamai.apate

fun restoreFileName(displayName: String, originalExtension: String?): String {
    val cleanExtension = originalExtension
        ?.trim()
        ?.trimStart('.')
        ?.takeIf { it.isNotBlank() }

    return if (cleanExtension == null) {
        displayName.substringBeforeLast('.', displayName)
    } else {
        val baseName = displayName.substringBeforeLast('.', displayName)
        "$baseName.$cleanExtension"
    }
}
