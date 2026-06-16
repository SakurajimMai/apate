use std::fs;
use std::path::PathBuf;

#[test]
fn android_project_is_restore_only_kotlin_rust_apk() {
    let root = workspace_root();
    let settings = fs::read_to_string(root.join("android/settings.gradle.kts")).unwrap();
    let app_gradle = fs::read_to_string(root.join("android/app/build.gradle.kts")).unwrap();
    let manifest =
        fs::read_to_string(root.join("android/app/src/main/AndroidManifest.xml")).unwrap();
    let strings =
        fs::read_to_string(root.join("android/app/src/main/res/values/strings.xml")).unwrap();
    let native_bridge = fs::read_to_string(
        root.join("android/app/src/main/java/moe/sakurajimamai/apate/NativeBridge.kt"),
    )
    .unwrap();
    let main_activity = fs::read_to_string(
        root.join("android/app/src/main/java/moe/sakurajimamai/apate/MainActivity.kt"),
    )
    .unwrap();
    let file_access = fs::read_to_string(
        root.join("android/app/src/main/java/moe/sakurajimamai/apate/FileAccess.kt"),
    )
    .unwrap();
    let skill = fs::read_to_string(root.join("skills/apate-cli/SKILL.md")).unwrap();

    assert!(settings.contains("rootProject.name = \"ApateAndroid\""));
    assert!(strings.contains("<string name=\"app_name\">apatex</string>"));
    assert!(app_gradle.contains("applicationId = \"moe.sakurajimamai.apate\""));
    assert!(app_gradle.contains("minSdk = 26"));
    assert!(app_gradle.contains("compose-bom:2026.05.01"));
    assert!(app_gradle.contains("sourceCompatibility = JavaVersion.VERSION_17"));
    assert!(app_gradle.contains("targetCompatibility = JavaVersion.VERSION_17"));
    assert!(app_gradle.contains("ANDROID_KEYSTORE_PASSWORD"));
    assert!(app_gradle.contains("ANDROID_KEY_ALIAS"));
    assert!(app_gradle.contains("ANDROID_KEY_PASSWORD"));
    assert!(manifest.contains("android:label=\"@string/app_name\""));
    assert!(manifest.contains("android:icon=\"@android:drawable/sym_def_app_icon\""));
    assert!(manifest.contains("<activity-alias"));
    assert!(manifest.contains("android:name=\".LauncherActivity\""));
    assert!(manifest.contains("android:targetActivity=\".MainActivity\""));
    assert!(manifest.contains("<intent-filter>"));
    assert!(manifest.contains("android.intent.action.MAIN"));
    assert!(native_bridge.contains("System.loadLibrary(\"apate_android\")"));
    assert!(native_bridge.contains("external fun inspectFd"));
    assert!(native_bridge.contains("external fun revealInPlaceFd"));
    assert!(native_bridge.contains("external fun restoreToFd"));
    assert!(main_activity.contains("ActivityResultContracts.OpenMultipleDocuments"));
    assert!(main_activity.contains("ActivityResultContracts.CreateDocument"));
    assert!(main_activity.contains("safeDrawingPadding"));
    assert!(main_activity.contains("BoxWithConstraints"));
    assert!(main_activity.contains("revealInPlace"));
    assert!(main_activity.contains("fileAccess.rename"));
    assert!(file_access.contains("DocumentsContract.renameDocument"));
    assert!(!main_activity.contains("disguise_file"));
    assert!(!main_activity.contains("Disguise"));
    assert!(skill.contains("不要把 `local`、`test`、`tests`"));
    assert!(skill.contains("用户明确给出的目标路径"));
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}
