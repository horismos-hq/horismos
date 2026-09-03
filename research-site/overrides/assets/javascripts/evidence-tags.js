/* Renders the wiki's evidence-labeling convention visually.
 *
 * The research/ markdown is canonical and tags are literal text there:
 * "[measured]", "[inference — …]", "[unverified — preprint]", "[DECIDE]".
 * This script wraps those spans in styled chips at view time only — the
 * source is never rewritten — and never touches code, pre, or links. */
(function () {
  "use strict";

  var TAG_RE = /\[(measured|inference|unverified|DECIDE)(\b[^\]]*)\]/g;
  var SKIP = { A: 1, CODE: 1, PRE: 1, SCRIPT: 1, STYLE: 1, KBD: 1, TEXTAREA: 1 };

  function shouldSkip(node) {
    for (var p = node.parentNode; p; p = p.parentNode) {
      if (p.nodeType !== 1) continue;
      if (SKIP[p.tagName]) return true;
      if (p.classList && p.classList.contains("ev")) return true;
    }
    return false;
  }

  function decorate(root) {
    if (!root) return;
    var walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, null);
    var targets = [];
    var n;
    while ((n = walker.nextNode())) {
      if (n.nodeValue.indexOf("[") === -1) continue;
      if (shouldSkip(n)) continue;
      TAG_RE.lastIndex = 0;
      if (TAG_RE.test(n.nodeValue)) targets.push(n);
    }
    targets.forEach(function (textNode) {
      var text = textNode.nodeValue;
      var frag = document.createDocumentFragment();
      var last = 0;
      var m;
      TAG_RE.lastIndex = 0;
      while ((m = TAG_RE.exec(text))) {
        if (m.index > last) frag.appendChild(document.createTextNode(text.slice(last, m.index)));
        var span = document.createElement("span");
        var kind = m[1].toLowerCase();
        span.className = "ev ev-" + kind + (m[0].length > 24 ? " ev-long" : "");
        span.setAttribute("title", kindTitle(kind));
        span.textContent = m[0];
        frag.appendChild(span);
        last = m.index + m[0].length;
      }
      if (last < text.length) frag.appendChild(document.createTextNode(text.slice(last)));
      textNode.parentNode.replaceChild(frag, textNode);
    });
  }

  function kindTitle(kind) {
    switch (kind) {
      case "measured":
        return "[measured] — backed by a cited paper, spec, standard, or a fact verified against the repo; the citation accompanies the tag.";
      case "inference":
        return "[inference] — synthesis or judgment connecting measured facts; not itself an independently citable fact.";
      case "unverified":
        return "[unverified] — a qualifier: the source was relayed only via a search snippet because the primary-text fetch failed.";
      case "decide":
        return "[DECIDE] — an open design question owned by exactly one PRP; resolved there, never ad hoc.";
      default:
        return "";
    }
  }

  function run() {
    decorate(document.querySelector(".md-content"));
  }

  // Material exposes document$ when navigation.instant is on; fall back otherwise.
  if (typeof window.document$ !== "undefined" && window.document$.subscribe) {
    window.document$.subscribe(run);
  } else if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", run);
  } else {
    run();
  }
})();
