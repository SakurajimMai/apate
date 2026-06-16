package moe.sakurajimamai.apate

import kotlin.test.Test
import kotlin.test.assertEquals

class NativeResultTest {
    @Test
    fun parsesInspectionResponse() {
        val result = NativeResult.parse(
            """
            {
              "ok": true,
              "code": "ok",
              "message": "处理成功",
              "disguised": true,
              "maskLength": 4,
              "payloadLength": 16,
              "originalExtension": "zip"
            }
            """.trimIndent(),
        )

        assertEquals(true, result.ok)
        assertEquals(true, result.disguised)
        assertEquals(4, result.maskLength)
        assertEquals(16, result.payloadLength)
        assertEquals("zip", result.originalExtension)
    }
}
