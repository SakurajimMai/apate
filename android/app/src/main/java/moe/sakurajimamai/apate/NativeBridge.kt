package moe.sakurajimamai.apate

object NativeBridge {
    init {
        System.loadLibrary("apate_android")
    }

    @JvmStatic
    external fun inspectFd(fd: Int): String

    @JvmStatic
    external fun revealInPlaceFd(fd: Int): String

    @JvmStatic
    external fun restoreToFd(inputFd: Int, outputFd: Int): String
}
