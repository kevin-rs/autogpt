(function () {
  document.addEventListener("DOMContentLoaded", function () {
    document.querySelectorAll('a[href^="#"]').forEach(function (anchor) {
      anchor.addEventListener("click", function (e) {
        const target = document.querySelector(anchor.getAttribute("href"));
        if (target) {
          e.preventDefault();
          target.scrollIntoView({ behavior: "smooth", block: "start" });
        }
      });
    });

    const currentPath = window.location.pathname
      .replace(/\/$/, "")
      .replace(/\.html$/, "");

    let matched = false;
    document.querySelectorAll(".sidebar a").forEach(function (link) {
      try {
        const linkPath = new URL(link.href).pathname
          .replace(/\/$/, "")
          .replace(/\.html$/, "");
        if (linkPath === currentPath) {
          matched = true;
          document
            .querySelectorAll(".chapter li.chapter-item.active")
            .forEach((li) => li.classList.remove("active"));
          link.parentElement.classList.add("active");
        }
      } catch (_) {}
    });
  });
})();

(function () {
  function syncSidebarActive() {
    var currentPath = window.location.pathname
      .replace(/\/$/, "")
      .replace(/\.html$/, "");

    document.querySelectorAll(".sidebar a").forEach(function (link) {
      try {
        var linkPath = new URL(link.href).pathname
          .replace(/\/$/, "")
          .replace(/\.html$/, "");

        if (linkPath === currentPath) {
          document.querySelectorAll(".sidebar a.active").forEach(function (a) {
            a.classList.remove("active");
          });
          document
            .querySelectorAll(".chapter li.chapter-item.active")
            .forEach(function (li) {
              li.classList.remove("active");
            });

          link.classList.add("active");
          if (link.parentElement) {
            link.parentElement.classList.add("active");
          }
        }
      } catch (_) {}
    });
  }

  document.addEventListener("DOMContentLoaded", syncSidebarActive);

  document.addEventListener("DOMContentLoaded", function () {
    setTimeout(syncSidebarActive, 50);
    setTimeout(syncSidebarActive, 300);
  });

  window.addEventListener("popstate", syncSidebarActive);
})();
