/* Model diagrams remain ordinary images when JavaScript or dialog is unavailable. */
(() => {
  if (typeof HTMLDialogElement === "undefined") return;

  function enhanceDiagrams() {
    const diagrams = [...document.querySelectorAll(".md-content img")].filter((img) => {
      const source = new URL(img.currentSrc || img.src, document.baseURI);
      return source.pathname.endsWith(".svg") &&
        !img.closest("a, button, [role='button']");
    });
    if (!diagrams.length) return;

    const dialog = document.createElement("dialog");
    dialog.className = "xd-diagram-dialog";
    dialog.setAttribute("aria-labelledby", "xd-diagram-title");
    const toolbar = document.createElement("div");
    toolbar.className = "xd-diagram-toolbar";
    const title = document.createElement("h2");
    title.id = "xd-diagram-title";
    title.className = "xd-diagram-title";
    const source = document.createElement("a");
    source.className = "xd-diagram-source";
    source.textContent = "Open SVG with source details";
    source.target = "_blank";
    source.rel = "noopener";
    const close = document.createElement("button");
    close.type = "button";
    close.className = "xd-diagram-close";
    close.textContent = "Close";
    close.autofocus = true;
    const viewport = document.createElement("div");
    viewport.className = "xd-diagram-viewport";
    viewport.tabIndex = 0;
    viewport.setAttribute("role", "region");
    viewport.setAttribute("aria-label", "Full-size diagram; scroll to explore");
    const fullImage = document.createElement("img");
    viewport.append(fullImage);
    toolbar.append(title, source, close);
    dialog.append(toolbar, viewport);
    document.body.append(dialog);
    close.addEventListener("click", () => dialog.close());
    dialog.addEventListener("click", (event) => {
      if (event.target !== dialog) return;
      const bounds = dialog.getBoundingClientRect();
      if (event.clientX < bounds.left || event.clientX > bounds.right ||
          event.clientY < bounds.top || event.clientY > bounds.bottom) dialog.close();
    });

    diagrams.forEach((img) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "xd-diagram-button";
      button.setAttribute("aria-label", `Expand diagram: ${img.alt || "System model"}`);
      button.setAttribute("aria-haspopup", "dialog");
      const hint = document.createElement("span");
      hint.className = "xd-diagram-hint";
      hint.textContent = "Open full-size diagram";
      hint.setAttribute("aria-hidden", "true");
      img.replaceWith(button);
      button.append(img, hint);
      button.addEventListener("click", () => {
        title.textContent = img.alt || "System model";
        fullImage.src = img.currentSrc || img.src;
        source.href = fullImage.src;
        fullImage.alt = img.alt;
        dialog.showModal();
        viewport.scrollTo(0, 0);
      });
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", enhanceDiagrams, { once: true });
  } else {
    enhanceDiagrams();
  }
})();
