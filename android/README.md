# Apate Android

Android 工程用于构建手机端 restore-only APK。它不实现伪装，只负责检查和还原通过 Apate 生成的文件。

## 构建逻辑

1. `cargo ndk` 构建 `crates/apate-android-jni`，输出到 `app/src/main/jniLibs`。
2. Gradle 构建 `:app:assembleRelease`。
3. Release 构建必须使用正式 keystore 签名。

## Release 签名

GitHub Actions 需要以下 Secrets：

- `ANDROID_KEYSTORE_BASE64`
- `ANDROID_KEYSTORE_PASSWORD`
- `ANDROID_KEY_ALIAS`
- `ANDROID_KEY_PASSWORD`

本地调试可以使用 debug 签名；GitHub Release 发布会强制检查以上 secrets。
