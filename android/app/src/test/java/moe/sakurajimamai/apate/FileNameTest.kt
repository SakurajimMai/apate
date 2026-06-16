package moe.sakurajimamai.apate

import kotlin.test.Test
import kotlin.test.assertEquals

class FileNameTest {
    @Test
    fun restoresRecordedExtensionByReplacingDisguiseExtension() {
        assertEquals("secret.zip", restoreFileName("secret.jpg", "zip"))
    }

    @Test
    fun fallsBackToRemovingLastExtensionWhenMetadataIsMissing() {
        assertEquals("secret", restoreFileName("secret.mp4", null))
    }

    @Test
    fun handlesExtensionWithLeadingDot() {
        assertEquals("secret.7z", restoreFileName("secret.jpg", ".7z"))
    }
}
