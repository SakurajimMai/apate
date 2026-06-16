package moe.sakurajimamai.apate

import org.json.JSONObject

data class NativeResult(
    val ok: Boolean,
    val code: String,
    val message: String,
    val disguised: Boolean?,
    val maskLength: Int?,
    val payloadLength: Long?,
    val originalExtension: String?,
) {
    companion object {
        fun parse(json: String): NativeResult {
            val value = JSONObject(json)
            return NativeResult(
                ok = value.optBoolean("ok"),
                code = value.optString("code"),
                message = value.optString("message"),
                disguised = value.optionalBoolean("disguised"),
                maskLength = value.optionalInt("maskLength"),
                payloadLength = value.optionalLong("payloadLength"),
                originalExtension = value.optionalString("originalExtension"),
            )
        }
    }
}

private fun JSONObject.optionalBoolean(name: String): Boolean? =
    if (isNull(name) || !has(name)) null else optBoolean(name)

private fun JSONObject.optionalInt(name: String): Int? =
    if (isNull(name) || !has(name)) null else optInt(name)

private fun JSONObject.optionalLong(name: String): Long? =
    if (isNull(name) || !has(name)) null else optLong(name)

private fun JSONObject.optionalString(name: String): String? =
    if (isNull(name) || !has(name)) null else optString(name).takeIf { it.isNotBlank() }
