package moe.sakurajimamai.apate

import android.content.ContentResolver
import android.database.Cursor
import android.net.Uri
import android.provider.DocumentsContract
import android.provider.OpenableColumns

class FileAccess(private val resolver: ContentResolver) {
    fun displayName(uri: Uri): String {
        val queried = resolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)
            ?.use { cursor -> cursor.displayNameOrNull() }
        return queried ?: uri.lastPathSegment?.substringAfterLast('/') ?: "apate-file"
    }

    fun inspect(uri: Uri): NativeResult {
        return runCatching {
            val fd = resolver.openFileDescriptor(uri, "r")?.use { it.detachFd() }
                ?: return ioError("无法打开文件")
            NativeResult.parse(NativeBridge.inspectFd(fd))
        }.getOrElse { error -> ioError(error.readableMessage("无法打开文件")) }
    }

    fun revealInPlace(uri: Uri): NativeResult {
        return runCatching {
            val fd = resolver.openFileDescriptor(uri, "rw")?.use { it.detachFd() }
                ?: return ioError("无法打开文件进行原地还原")
            NativeResult.parse(NativeBridge.revealInPlaceFd(fd))
        }.getOrElse { error -> ioError(error.readableMessage("无法打开文件进行原地还原")) }
    }

    fun restoreTo(input: Uri, output: Uri): NativeResult {
        return runCatching {
            resolver.openFileDescriptor(input, "r").use { inputPfd ->
                resolver.openFileDescriptor(output, "w").use { outputPfd ->
                    if (inputPfd == null) return ioError("无法打开源文件")
                    if (outputPfd == null) return ioError("无法打开保存位置")
                    NativeResult.parse(
                        NativeBridge.restoreToFd(inputPfd.detachFd(), outputPfd.detachFd()),
                    )
                }
            }
        }.getOrElse { error -> ioError(error.readableMessage("无法另存还原文件")) }
    }

    fun rename(uri: Uri, newName: String): Uri? =
        runCatching { DocumentsContract.renameDocument(resolver, uri, newName) }.getOrNull()

    private fun ioError(message: String): NativeResult =
        NativeResult(false, "io_error", message, null, null, null, null)
}

private fun Throwable.readableMessage(fallback: String): String =
    localizedMessage?.takeIf { it.isNotBlank() } ?: message?.takeIf { it.isNotBlank() } ?: fallback

private fun Cursor.displayNameOrNull(): String? {
    if (!moveToFirst()) return null
    val index = getColumnIndex(OpenableColumns.DISPLAY_NAME)
    if (index < 0) return null
    return getString(index)
}
