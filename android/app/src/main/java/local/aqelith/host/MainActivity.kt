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
class MainActivity : Activity() {
    private lateinit var pages: List<JSONObject>

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val text = assets.open("landing.json").bufferedReader().use { it.readText() }
        val array = JSONObject(text).getJSONArray("pages")
        pages = (0 until array.length()).map { array.getJSONObject(it) }
        show(pages.first { it.getString("route") == "/" })
    }

    private fun show(page: JSONObject) {
        title = page.getString("title")
        val column = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(48, 48, 48, 48)
        }
        val items = page.getJSONArray("items")
        for (i in 0 until items.length()) {
            val item = items.getJSONObject(i)
            when (item.getString("kind")) {
                "heading" -> column.addView(TextView(this).apply {
                    text = item.getString("text")
                    textSize = if (item.optInt("level", 1) == 1) 28f else 20f
                    setTypeface(typeface, Typeface.BOLD)
                    contentDescription = item.getString("text")
                })
                "para" -> column.addView(TextView(this).apply {
                    text = item.getString("text")
                    textSize = 16f
                    contentDescription = item.getString("text")
                })
                // The exporter names a link's visible text "label" (headings and paragraphs use "text").
                "link" -> column.addView(Button(this).apply {
                    text = item.getString("label")
                    contentDescription = item.getString("label")
                    setOnClickListener {
                        val target = pages.firstOrNull { it.getString("route") == item.getString("href") }
                        if (target != null) show(target)
                    }
                })
            }
        }
        setContentView(ScrollView(this).apply {
            layoutParams = ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT)
            addView(column)
        })
    }
}
