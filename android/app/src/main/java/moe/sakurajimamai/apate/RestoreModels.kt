package moe.sakurajimamai.apate

import android.net.Uri

data class FileItem(
    val uri: Uri,
    val name: String,
    val state: FileState = FileState.Selected,
    val originalExtension: String? = null,
    val payloadLength: Long? = null,
    val message: String = "等待检查",
)

enum class FileState {
    Selected,
    Ready,
    Restored,
    NeedsSaveAs,
    Failed,
}

data class PendingSaveAs(
    val uri: Uri,
    val suggestedName: String,
)
