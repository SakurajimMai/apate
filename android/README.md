# Apate Android

Android 工程用于构建手机端 restore-only APK。它不实现伪装，只负责检查和还原通过 Apate 生成的文件。

## 构建逻辑

1. `cargo ndk` 构建 `crates/apate-android-jni`，输出到 `app/src/main/jniLibs`。
2. Gradle 构建 `:app:assembleRelease`。
3. 如果存在固定 release keystore，则使用 release 签名；否则使用 debug 签名生成可手动侧载的 APK。

## APK 签名

Android 系统要求 APK 必须签名，但不要求一定使用你自己的 release keystore。当前 CI 会始终构建 APK 并上传到 GitHub Releases：

- 未配置 secrets：使用 Gradle debug 签名，用户可以手动下载安装；后续版本可能需要先卸载已安装版本再安装新版。
- 已配置 secrets：使用固定 release 签名，用户后续可以直接覆盖安装升级。

固定 release 签名需要以下 GitHub Secrets：

- `ANDROID_KEYSTORE_BASE64`
- `ANDROID_KEYSTORE_PASSWORD`
- `ANDROID_KEY_ALIAS`
- `ANDROID_KEY_PASSWORD`
