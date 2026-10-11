// Abax landing page: hero playback that respects reduced motion, and the
// download block filled from the latest GitHub release. Without JavaScript,
// or when the API cannot be reached, every button links to the releases page.
(function () {
  "use strict";

  var RELEASES = "https://github.com/ZahakJ/abax/releases/latest";
  var API = "https://api.github.com/repos/ZahakJ/abax/releases/latest";

  // ------------------------------------------------------------- hero video
  var video = document.getElementById("hero-video");
  if (video) {
    var reduce = window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)");
    var start = function () {
      if (reduce && reduce.matches) {
        video.pause();
        video.removeAttribute("autoplay");
        return;
      }
      video.muted = true;
      var p = video.play();
      if (p && p.catch) p.catch(function () { /* autoplay refused: poster stays */ });
    };
    start();
    if (reduce && reduce.addEventListener) reduce.addEventListener("change", start);
  }

  // ------------------------------------------------------ platform guessing
  function guessPlatform() {
    var ua = navigator.userAgent || "";
    // Android first: its user agent also says "Linux".
    if (/Android/i.test(ua)) return Promise.resolve("android");
    if (/iPhone|iPad|iPod/i.test(ua)) return Promise.resolve("ios");
    if (/Windows/i.test(ua)) return Promise.resolve("windows");
    if (/Mac OS X|Macintosh/i.test(ua)) {
      // Browsers report Intel in the user agent even on Apple silicon; ask
      // for the architecture where the browser can tell.
      var uad = navigator.userAgentData;
      if (uad && uad.getHighEntropyValues) {
        return uad.getHighEntropyValues(["architecture"]).then(function (v) {
          return v.architecture === "x86" ? "mac-intel" : "mac-arm";
        }, function () { return "mac-unknown"; });
      }
      return Promise.resolve("mac-unknown");
    }
    if (/Linux|X11|CrOS/i.test(ua)) return Promise.resolve("linux");
    return Promise.resolve("");
  }

  function mark(platform) {
    var note = document.getElementById("dl-note");
    var show = function (text) {
      if (note) { note.textContent = text; note.hidden = false; }
    };
    var pick = platform;
    if (platform === "mac-unknown") {
      pick = "mac-arm";
      show("Macs from late 2020 on use Apple silicon. Apple menu > About This Mac shows \"Apple M…\" or \"Intel\".");
    }
    if (platform === "ios") {
      show("There is no iPhone or iPad version. Abax runs on Android, Linux, Windows and macOS.");
      return;
    }
    if (!pick) return;
    var el = document.querySelector('.dl[data-platform="' + pick + '"]');
    if (el) el.classList.add("is-mine");
  }

  guessPlatform().then(mark, function () {});

  // ------------------------------------------------------- latest release
  function size(bytes) {
    if (!bytes) return "";
    return (bytes / 1048576).toFixed(1) + " MB";
  }

  function date(iso) {
    var d = new Date(iso);
    if (isNaN(d.getTime())) return "";
    return d.toLocaleDateString("en-GB", { year: "numeric", month: "long", day: "numeric", timeZone: "UTC" });
  }

  function fill(rel) {
    var assets = rel.assets || [];
    var find = function (suffix) {
      for (var i = 0; i < assets.length; i++) {
        var n = assets[i].name || "";
        if (n.slice(-suffix.length) === suffix) return assets[i];
      }
      return null;
    };
    var buttons = document.querySelectorAll(".dl[data-suffix]");
    for (var i = 0; i < buttons.length; i++) {
      var b = buttons[i];
      var a = find(b.getAttribute("data-suffix"));
      if (!a) continue;
      b.href = a.browser_download_url;
      var meta = b.querySelector(".dl-meta");
      if (meta && a.size) meta.textContent = meta.textContent + ", " + size(a.size);
    }
    var sums = find("SHA256SUMS.txt");
    var sl = document.getElementById("sums");
    if (sums && sl) sl.href = sums.browser_download_url;
    else if (sl && rel.html_url) sl.href = rel.html_url;

    var line = document.getElementById("release-line");
    if (line && rel.tag_name) {
      line.textContent = "";
      line.appendChild(document.createTextNode("Latest release: "));
      var strong = document.createElement("strong");
      strong.textContent = rel.tag_name;
      line.appendChild(strong);
      var when = date(rel.published_at);
      if (when) line.appendChild(document.createTextNode(", " + when + ". "));
      else line.appendChild(document.createTextNode(". "));
      var notes = document.createElement("a");
      notes.href = rel.html_url || RELEASES;
      notes.textContent = "Release notes";
      line.appendChild(notes);
    }
  }

  if (window.fetch) {
    var ctl = window.AbortController ? new AbortController() : null;
    var timer = ctl ? setTimeout(function () { ctl.abort(); }, 8000) : null;
    fetch(API, { headers: { Accept: "application/vnd.github+json" }, signal: ctl ? ctl.signal : undefined })
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        return r.json();
      })
      .then(fill)
      .catch(function () {
        // Rate-limited or offline: the static links to the releases page stay.
      })
      .then(function () { if (timer) clearTimeout(timer); });
  }
})();
