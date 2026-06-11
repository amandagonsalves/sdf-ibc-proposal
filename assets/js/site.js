(function () {
  // Mobile nav toggle
  function bindNav() {
    var btn = document.querySelector(".menu-btn");
    var scrim = document.querySelector(".scrim");
    if (btn) {
      btn.addEventListener("click", function () {
        document.body.classList.toggle("nav-open");
      });
    }
    if (scrim) {
      scrim.addEventListener("click", function () {
        document.body.classList.remove("nav-open");
      });
    }
    document.querySelectorAll(".nav-link").forEach(function (a) {
      a.addEventListener("click", function () {
        document.body.classList.remove("nav-open");
      });
    });
  }

  // Convert kramdown/rouge ```mermaid blocks into mermaid containers,
  // pulling the raw text (textContent strips the syntax-highlight spans).
  function renderMermaid() {
    var blocks = document.querySelectorAll(
      ".language-mermaid, pre > code.language-mermaid"
    );
    if (!blocks.length) return;
    blocks.forEach(function (el) {
      var host = el.closest(".language-mermaid") || el;
      var code = (el.textContent || host.textContent).trim();
      var div = document.createElement("div");
      div.className = "mermaid";
      div.textContent = code;
      host.replaceWith(div);
    });

    var s = document.createElement("script");
    s.type = "module";
    s.textContent =
      'import mermaid from "https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs";' +
      'mermaid.initialize({' +
      '  startOnLoad: false,' +
      '  theme: "base",' +
      '  themeVariables: {' +
      '    background: "#0c0e22",' +
      '    primaryColor: "#181c3a",' +
      '    primaryBorderColor: "#8b6dff",' +
      '    primaryTextColor: "#e8eaf6",' +
      '    secondaryColor: "#12152e",' +
      '    tertiaryColor: "#12152e",' +
      '    lineColor: "#8b6dff",' +
      '    textColor: "#cdd1ef",' +
      '    fontFamily: "Inter, sans-serif",' +
      '    actorBorder: "#8b6dff",' +
      '    actorBkg: "#181c3a",' +
      '    actorTextColor: "#e8eaf6",' +
      '    signalColor: "#9aa0c6",' +
      '    signalTextColor: "#cdd1ef",' +
      '    labelBoxBkgColor: "#181c3a",' +
      '    labelBoxBorderColor: "#ff4d97",' +
      '    labelTextColor: "#e8eaf6",' +
      '    noteBkgColor: "#1c2247",' +
      '    noteBorderColor: "#38d6e6",' +
      '    noteTextColor: "#e8eaf6",' +
      '    clusterBkg: "#0e1228",' +
      '    clusterBorder: "#2a2f55"' +
      '  }' +
      '});' +
      'mermaid.run({ querySelector: ".mermaid" });';
    document.body.appendChild(s);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      bindNav();
      renderMermaid();
    });
  } else {
    bindNav();
    renderMermaid();
  }
})();
