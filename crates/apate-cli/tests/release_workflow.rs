use std::fs;

#[test]
fn release_workflow_builds_on_main_push_and_tags() {
    let workflow = fs::read_to_string("../../.github/workflows/release.yml").unwrap();

    assert!(workflow.contains("branches: [main]"));
    assert!(workflow.contains("tags: [\"v*\"]"));
    assert!(workflow.contains("target: x86_64-pc-windows-msvc"));
    assert!(workflow.contains("target: x86_64-unknown-linux-gnu"));
    assert!(workflow.contains("name: build android apk"));
    assert!(workflow.contains("name: check android signing"));
    assert!(workflow.contains("configured: ${{ steps.check.outputs.configured }}"));
    assert!(workflow.contains("if: needs.android-signing.outputs.configured == 'true'"));
    assert!(workflow.contains("actions/setup-java@v4"));
    assert!(workflow.contains("android-actions/setup-android@v3"));
    assert!(workflow.contains("gradle/actions/setup-gradle@v4"));
    assert!(workflow.contains(
        "sdkmanager \"platforms;android-36\" \"build-tools;36.0.0\" \"ndk;27.0.12077973\""
    ));
    assert!(workflow.contains("ANDROID_NDK_HOME=$ANDROID_HOME/ndk/27.0.12077973"));
    assert!(workflow.contains("ANDROID_NDK_ROOT=$ANDROID_HOME/ndk/27.0.12077973"));
    assert!(workflow.contains("aarch64-linux-android"));
    assert!(workflow.contains("armv7-linux-androideabi"));
    assert!(workflow.contains("i686-linux-android"));
    assert!(workflow.contains("x86_64-linux-android"));
    assert!(workflow.contains("-t arm64-v8a"));
    assert!(workflow.contains("-t armeabi-v7a"));
    assert!(workflow.contains("-t x86"));
    assert!(workflow.contains("-t x86_64"));
    assert!(workflow.contains("cargo ndk"));
    assert!(
        workflow.contains(
            "build --manifest-path crates/apate-android-jni/Cargo.toml --release --locked"
        )
    );
    assert!(workflow.contains("android/app/src/main/jniLibs"));
    assert!(workflow.contains("working-directory: android"));
    assert!(workflow.contains("gradle :app:assembleRelease"));
    assert!(workflow.contains("ANDROID_KEYSTORE_BASE64"));
    assert!(workflow.contains("ANDROID_KEYSTORE_PASSWORD"));
    assert!(workflow.contains("ANDROID_KEY_ALIAS"));
    assert!(workflow.contains("ANDROID_KEY_PASSWORD"));
    assert!(workflow.contains("needs: [build, android-signing, android]"));
    assert!(
        workflow.contains("needs.android.result == 'success' || needs.android.result == 'skipped'")
    );
    assert!(workflow.contains("name: Upload tagged Android APK"));
    assert!(workflow.contains("name: Upload latest Android APK"));
    assert!(workflow.contains("gh release upload \"${{ github.ref_name }}\""));
    assert!(workflow.contains("gh release upload latest"));
    assert!(workflow.contains("Copy-Item \"target\\${{ matrix.target }}\\release\\apate.exe\""));
    assert!(workflow.contains("cp target/${{ matrix.target }}/release/apate dist/apate"));
    assert!(workflow.contains("name: Publish tagged release"));
    assert!(workflow.contains("if: startsWith(github.ref, 'refs/tags/')"));
    assert!(workflow.contains("name: Publish latest prerelease"));
    assert!(workflow.contains("if: github.ref == 'refs/heads/main'"));
    assert!(workflow.contains("tag_name: latest"));
    assert!(workflow.contains("prerelease: true"));
    assert!(workflow.contains("overwrite_files: true"));
    assert!(workflow.contains("tag_name: ${{ github.ref_name }}"));
    assert!(workflow.contains("dist/**/*.zip"));
    assert!(workflow.contains("dist/**/*.tar.gz"));
    assert!(!workflow.contains("Copy-Item \"CHANGELOG.md\""));
    assert!(!workflow.contains("cp CHANGELOG.md dist/CHANGELOG.md"));
}
