// Live reload client for the AQELITH dev server (web-dev-loop, S5). The dev server injects it into
// every page it serves; static builds do not include it. It runs no code from the page and reads
// no page data: it listens to one same-origin event stream and reloads or shows a message.
(function () {
  "use strict";

  var banner = null;
  var wasDown = false;

  function showError(text) {
    if (!banner) {
      banner = document.createElement("pre");
      banner.setAttribute("role", "alert");
      banner.style.cssText =
        "position:fixed;top:0;left:0;right:0;margin:0;padding:1rem;" +
        "z-index:2147483647;background:#fff1f1;color:#7a0000;white-space:pre-wrap;";
      document.body.appendChild(banner);
    }
    // textContent, never innerHTML: the diagnostic text is data.
    banner.textContent = text;
  }

  function parse(data) {
    try {
      var value = JSON.parse(data);
      return typeof value === "string" ? value : String(value);
    } catch (e) {
      return "dev server sent an unreadable message";
    }
  }

  var es = new EventSource("/__aq/events");

  es.addEventListener("reload", function () {
    location.reload();
  });

  es.addEventListener("build-error", function (event) {
    showError(parse(event.data));
  });

  es.onerror = function () {
    wasDown = true;
  };

  // After the dev server restarts, the open page is stale: reload it once the stream is back.
  es.onopen = function () {
    if (wasDown) {
      location.reload();
    }
  };
})();
