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
      '    background: "#070b1a",' +
      '    primaryColor: "#131c3d",' +
      '    primaryBorderColor: "#4da3f0",' +
      '    primaryTextColor: "#dfe6f2",' +
      '    secondaryColor: "#0c1330",' +
      '    tertiaryColor: "#0c1330",' +
      '    lineColor: "#a97ce0",' +
      '    textColor: "#dfe6f2",' +
      '    fontFamily: "Inter, sans-serif",' +
      '    actorBorder: "#4da3f0",' +
      '    actorBkg: "#131c3d",' +
      '    actorTextColor: "#dfe6f2",' +
      '    signalColor: "#93a1bd",' +
      '    signalTextColor: "#dfe6f2",' +
      '    labelBoxBkgColor: "#131c3d",' +
      '    labelBoxBorderColor: "#a97ce0",' +
      '    labelTextColor: "#dfe6f2",' +
      '    noteBkgColor: "#1b2547",' +
      '    noteBorderColor: "#a97ce0",' +
      '    noteTextColor: "#dfe6f2",' +
      '    clusterBkg: "#0b1330",' +
      '    clusterBorder: "#24325c"' +
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
