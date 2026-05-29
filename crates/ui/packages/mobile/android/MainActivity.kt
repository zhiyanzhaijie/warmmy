package dev.dioxus.main

import android.app.Activity
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.net.Uri
import android.provider.OpenableColumns
import android.util.Base64
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.result.ActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import org.json.JSONArray
import org.json.JSONObject
import java.io.ByteArrayOutputStream

typealias BuildConfig = com.zhiyanzhaijie.warmmy.BuildConfig

class MainActivity : WryActivity() {
    private val maxImageEdge = 1536
    private val jpegQuality = 82
    private var webView: WebView? = null

    private val imagePicker = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result: ActivityResult ->
        handleImagePickerResult(result)
    }

    override fun onWebViewCreate(webView: WebView) {
        this.webView = webView
        webView.addJavascriptInterface(WarmmyBridge(), "WarmmyAndroid")
    }

    inner class WarmmyBridge {
        @JavascriptInterface
        fun pickImages() {
            runOnUiThread {
                val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
                    addCategory(Intent.CATEGORY_OPENABLE)
                    type = "image/*"
                    putExtra(Intent.EXTRA_ALLOW_MULTIPLE, true)
                }
                imagePicker.launch(intent)
            }
        }
    }

    private fun handleImagePickerResult(result: ActivityResult) {
        if (result.resultCode != Activity.RESULT_OK) {
            dispatchPickedImages(JSONArray(), null)
            return
        }

        try {
            val data = result.data
            val output = JSONArray()

            data?.clipData?.let { clip ->
                for (index in 0 until clip.itemCount) {
                    output.put(readImage(clip.getItemAt(index).uri))
                }
            } ?: data?.data?.let { uri ->
                output.put(readImage(uri))
            }

            dispatchPickedImages(output, null)
        } catch (error: Throwable) {
            dispatchPickedImages(JSONArray(), error.message ?: error.toString())
        }
    }

    private fun readImage(uri: Uri): JSONObject {
        val bytes = readCompressedJpeg(uri)

        val name = queryDisplayName(uri) ?: "image"
        val encoded = Base64.encodeToString(bytes, Base64.NO_WRAP)

        return JSONObject().apply {
            put("name", name)
            put("mime_type", "image/jpeg")
            put("size_bytes", bytes.size)
            put("data_url", "data:image/jpeg;base64,$encoded")
        }
    }

    private fun readCompressedJpeg(uri: Uri): ByteArray {
        val bounds = BitmapFactory.Options().apply {
            inJustDecodeBounds = true
        }
        contentResolver.openInputStream(uri)?.use { stream ->
            BitmapFactory.decodeStream(stream, null, bounds)
        }

        val sampleSize = calculateSampleSize(bounds.outWidth, bounds.outHeight)
        val options = BitmapFactory.Options().apply {
            inSampleSize = sampleSize
        }
        val bitmap = contentResolver.openInputStream(uri)?.use { stream ->
            BitmapFactory.decodeStream(stream, null, options)
        } ?: return ByteArray(0)

        val scaled = scaleBitmap(bitmap)
        if (scaled !== bitmap) {
            bitmap.recycle()
        }

        val output = ByteArrayOutputStream()
        scaled.compress(Bitmap.CompressFormat.JPEG, jpegQuality, output)
        scaled.recycle()
        return output.toByteArray()
    }

    private fun calculateSampleSize(width: Int, height: Int): Int {
        var sampleSize = 1
        var currentWidth = width
        var currentHeight = height
        while (currentWidth / 2 >= maxImageEdge || currentHeight / 2 >= maxImageEdge) {
            currentWidth /= 2
            currentHeight /= 2
            sampleSize *= 2
        }
        return sampleSize
    }

    private fun scaleBitmap(bitmap: Bitmap): Bitmap {
        val width = bitmap.width
        val height = bitmap.height
        val longest = maxOf(width, height)
        if (longest <= maxImageEdge) {
            return bitmap
        }

        val scale = maxImageEdge.toFloat() / longest.toFloat()
        val targetWidth = (width * scale).toInt().coerceAtLeast(1)
        val targetHeight = (height * scale).toInt().coerceAtLeast(1)
        return Bitmap.createScaledBitmap(bitmap, targetWidth, targetHeight, true)
    }

    private fun queryDisplayName(uri: Uri): String? {
        return contentResolver.query(uri, null, null, null, null)?.use { cursor ->
            val index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            if (index >= 0 && cursor.moveToFirst()) cursor.getString(index) else null
        }
    }

    private fun dispatchPickedImages(files: JSONArray, error: String?) {
        val payload = JSONObject().apply {
            put("files", files)
            put("error", error)
        }
        val script = """
            window.dispatchEvent(new CustomEvent("warmmy-images-picked", {
                detail: $payload
            }));
        """.trimIndent()

        webView?.post {
            webView?.evaluateJavascript(script, null)
        }
    }
}
