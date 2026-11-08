package local.aqelith.host

import android.app.Activity
import android.graphics.Typeface
import android.os.Bundle
import android.view.ViewGroup
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import org.json.JSONObject

// Draws one page of the JSON that `aqelith view` exports. A link switches to the page with that route.
// The language does not know about Android; this host is platform code (docs/platforms/MOBILE_ANDROID.md).
// The asset is about 1 KB and is read once at start-up, so reading it on the main thread is accepted.
// Malformed or missing data is shown as a message, never as a crash.
class MainActivity : Activity() {
    private var pages: List<JSONObject> = emptyList()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        pages = try {
            val text = assets.open("landing.json").bufferedReader().use { it.readText() }
            val array = JSONObject(text).optJSONArray("pages")
            if (array == null) emptyList() else (0 until array.length()).mapNotNull { array.optJSONObject(it) }
        } catch (e: Exception) {
            emptyList()
        }
        val home = pages.firstOrNull { it.optString("route") == "/" }
        if (home == null) showMessage("No home page was found in the exported pages.") else show(home)
    }

    private fun showMessage(message: String) {
        title = "AQELITH host"
        val column = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(48, 48, 48, 48)
            addView(TextView(this@MainActivity).apply { text = message; textSize = 16f })
        }
        setContentView(column)
    }

    private fun show(page: JSONObject) {
        title = page.optString("title", "AQELITH host")
        val column = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(48, 48, 48, 48)
        }
        val items = page.optJSONArray("items")
        for (i in 0 until (items?.length() ?: 0)) {
            val item = items?.optJSONObject(i) ?: continue
            when (item.optString("kind")) {
                "heading" -> column.addView(TextView(this).apply {
                    val label = item.optString("text")
                    text = label
                    textSize = if (item.optInt("level", 1) == 1) 28f else 20f
                    setTypeface(typeface, Typeface.BOLD)
                    contentDescription = label
                })
                "para" -> column.addView(TextView(this).apply {
                    val label = item.optString("text")
                    text = label
                    textSize = 16f
                    contentDescription = label
                })
                // The exporter names a link's visible text "label" (headings and paragraphs use "text").
                "link" -> column.addView(Button(this).apply {
                    val label = item.optString("label")
                    val href = item.optString("href")
                    text = label
                    contentDescription = label
                    setOnClickListener {
                        // A link whose route is not in the bundled pages does nothing; it is never followed to the outside.
                        pages.firstOrNull { it.optString("route") == href }?.let { show(it) }
                    }
                })
                // Unknown kinds are skipped on purpose: the exporter may add kinds that this host does not draw yet.
            }
        }
        setContentView(ScrollView(this).apply {
            layoutParams = ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT)
            addView(column)
        })
    }
}
