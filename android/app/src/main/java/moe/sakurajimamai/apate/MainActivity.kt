package moe.sakurajimamai.apate

import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.CheckCircle
import androidx.compose.material.icons.outlined.ErrorOutline
import androidx.compose.material.icons.outlined.FolderOpen
import androidx.compose.material.icons.outlined.Restore
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            ApateTheme {
                RestoreScreen(FileAccess(contentResolver))
            }
        }
    }
}

@Composable
fun RestoreScreen(fileAccess: FileAccess) {
    val scope = rememberCoroutineScope()
    val files = remember { mutableStateListOf<FileItem>() }
    var busy by remember { mutableStateOf(false) }
    var status by remember { mutableStateOf("选择通过 Apate 伪装的文件，然后进行还原") }
    var pendingSaveAs by remember { mutableStateOf<PendingSaveAs?>(null) }

    fun updateFile(uri: Uri, transform: (FileItem) -> FileItem) {
        val index = files.indexOfFirst { it.uri == uri }
        if (index >= 0) files[index] = transform(files[index])
    }

    fun enqueueNextSaveAs() {
        val next = files.firstOrNull { it.state == FileState.NeedsSaveAs }
        pendingSaveAs = next?.let {
            PendingSaveAs(it.uri, restoreFileName(it.name, it.originalExtension))
        }
    }

    suspend fun inspectSelected(uris: List<Uri>) {
        busy = true
        files.clear()
        uris.forEach { uri ->
            val name = withContext(Dispatchers.IO) { fileAccess.displayName(uri) }
            val result = withContext(Dispatchers.IO) { fileAccess.inspect(uri) }
            files += if (result.ok && result.disguised == true) {
                FileItem(
                    uri = uri,
                    name = name,
                    state = FileState.Ready,
                    originalExtension = result.originalExtension,
                    payloadLength = result.payloadLength,
                    message = "可还原为 ${restoreFileName(name, result.originalExtension)}",
                )
            } else {
                FileItem(
                    uri = uri,
                    name = name,
                    state = FileState.Failed,
                    message = if (result.ok) "未识别为 Apate 文件" else result.message,
                )
            }
        }
        val ready = files.count { it.state == FileState.Ready }
        status = "已选择 ${files.size} 个文件，可还原 $ready 个"
        busy = false
    }

    suspend fun revealReadyFiles() {
        busy = true
        files.filter { it.state == FileState.Ready }.forEach { item ->
            val result = withContext(Dispatchers.IO) { fileAccess.revealInPlace(item.uri) }
            val restoredName = restoreFileName(
                item.name,
                result.originalExtension ?: item.originalExtension,
            )
            val renamedUri = if (result.ok) {
                withContext(Dispatchers.IO) { fileAccess.rename(item.uri, restoredName) }
            } else {
                null
            }
            updateFile(item.uri) {
                if (result.ok) {
                    it.copy(
                        uri = renamedUri ?: it.uri,
                        name = if (renamedUri != null) restoredName else it.name,
                        state = FileState.Restored,
                        message = if (renamedUri != null) {
                            "已原地还原为 $restoredName"
                        } else {
                            "已原地还原，文件名可手动改为 $restoredName"
                        },
                    )
                } else {
                    it.copy(
                        state = FileState.NeedsSaveAs,
                        message = "原地还原不可用，需要另存",
                    )
                }
            }
        }
        busy = false
        enqueueNextSaveAs()
        val restored = files.count { it.state == FileState.Restored }
        val needsSaveAs = files.count { it.state == FileState.NeedsSaveAs }
        status = if (needsSaveAs > 0) {
            "已原地还原 $restored 个，另有 $needsSaveAs 个需要另存"
        } else {
            "已完成，还原 $restored 个"
        }
    }

    val openDocuments = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.OpenMultipleDocuments(),
    ) { uris ->
        if (uris.isNotEmpty()) {
            scope.launch { inspectSelected(uris) }
        }
    }

    val createDocument = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.CreateDocument("application/octet-stream"),
    ) { outputUri ->
        val pending = pendingSaveAs
        if (outputUri != null && pending != null) {
            scope.launch {
                busy = true
                val result = withContext(Dispatchers.IO) {
                    fileAccess.restoreTo(pending.uri, outputUri)
                }
                updateFile(pending.uri) {
                    if (result.ok) {
                        it.copy(state = FileState.Restored, message = "已另存还原")
                    } else {
                        it.copy(state = FileState.Failed, message = result.message)
                    }
                }
                pendingSaveAs = null
                busy = false
                enqueueNextSaveAs()
                status = "另存处理完成"
            }
        } else {
            pendingSaveAs = null
            status = "已取消另存"
        }
    }

    LaunchedEffect(pendingSaveAs) {
        pendingSaveAs?.let { pending ->
            createDocument.launch(pending.suggestedName)
        }
    }

    Surface(
        modifier = Modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background,
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(horizontal = 20.dp, vertical = 24.dp),
        ) {
            Header(status = status, busy = busy)
            Spacer(Modifier.height(20.dp))
            ActionBar(
                canRestore = files.any { it.state == FileState.Ready } && !busy,
                onPick = { openDocuments.launch(arrayOf("*/*")) },
                onRestore = { scope.launch { revealReadyFiles() } },
            )
            Spacer(Modifier.height(16.dp))
            FileList(files = files)
        }
    }
}

@Composable
private fun Header(status: String, busy: Boolean) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Box(
                modifier = Modifier
                    .size(42.dp)
                    .background(MaterialTheme.colorScheme.primaryContainer, RoundedCornerShape(12.dp)),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = Icons.Outlined.Restore,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onPrimaryContainer,
                )
            }
            Column {
                Text(
                    text = "Apate 还原",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.SemiBold,
                )
                Text(
                    text = "手机端恢复伪装文件",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }
            if (busy) {
                Spacer(Modifier.weight(1f))
                CircularProgressIndicator(modifier = Modifier.size(24.dp), strokeWidth = 3.dp)
            }
        }
        Text(
            text = status,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.bodyMedium,
        )
    }
}

@Composable
private fun ActionBar(
    canRestore: Boolean,
    onPick: () -> Unit,
    onRestore: () -> Unit,
) {
    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Button(
            onClick = onPick,
            contentPadding = PaddingValues(horizontal = 18.dp, vertical = 14.dp),
        ) {
            Icon(Icons.Outlined.FolderOpen, contentDescription = null)
            Spacer(Modifier.width(8.dp))
            Text("选择文件")
        }
        OutlinedButton(
            enabled = canRestore,
            onClick = onRestore,
            contentPadding = PaddingValues(horizontal = 18.dp, vertical = 14.dp),
        ) {
            Icon(Icons.Outlined.Restore, contentDescription = null)
            Spacer(Modifier.width(8.dp))
            Text("还原")
        }
    }
}

@Composable
private fun FileList(files: List<FileItem>) {
    if (files.isEmpty()) {
        EmptyState()
        return
    }

    LazyColumn(
        verticalArrangement = Arrangement.spacedBy(10.dp),
        contentPadding = PaddingValues(bottom = 24.dp),
    ) {
        items(files, key = { it.uri.toString() }) { item ->
            FileRow(item)
        }
    }
}

@Composable
private fun EmptyState() {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(8.dp),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant),
    ) {
        Column(
            modifier = Modifier.padding(20.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text("还没有选择文件", fontWeight = FontWeight.Medium)
            Text(
                "支持从文件管理器、网盘目录或下载目录选择 Apate 文件。",
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}

@Composable
private fun FileRow(item: FileItem) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(8.dp),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
        elevation = CardDefaults.cardElevation(defaultElevation = 1.dp),
    ) {
        Row(
            modifier = Modifier.padding(14.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Icon(
                imageVector = if (item.state == FileState.Failed) {
                    Icons.Outlined.ErrorOutline
                } else {
                    Icons.Outlined.CheckCircle
                },
                contentDescription = null,
                tint = item.stateColor(),
            )
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = item.name,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    fontWeight = FontWeight.Medium,
                )
                Text(
                    text = item.message,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    style = MaterialTheme.typography.bodySmall,
                )
            }
        }
    }
}

@Composable
private fun FileItem.stateColor(): Color = when (state) {
    FileState.Selected -> MaterialTheme.colorScheme.onSurfaceVariant
    FileState.Ready -> MaterialTheme.colorScheme.primary
    FileState.Restored -> MaterialTheme.colorScheme.tertiary
    FileState.NeedsSaveAs -> MaterialTheme.colorScheme.secondary
    FileState.Failed -> MaterialTheme.colorScheme.error
}
