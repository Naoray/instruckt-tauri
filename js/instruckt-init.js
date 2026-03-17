/**
 * Tauri IPC shim + auto-initialization for instruckt.
 *
 * This script is injected into every webview via `js_init_script`.
 * It does three things:
 * 1. Monkey-patches fetch() to intercept instruckt API calls → Tauri IPC
 * 2. Waits for the DOM to be ready
 * 3. Initializes instruckt with the Tauri-appropriate config
 *
 * The instruckt IIFE bundle is injected separately (before this script)
 * and exposes the global `Instruckt` object.
 */
(function () {
  "use strict";

  // Use __TAURI_INTERNALS__ directly — always available in init scripts,
  // regardless of withGlobalTauri setting. The public window.__TAURI__ API
  // isn't ready yet when init scripts run (before DOM parsing).
  if (!window.__TAURI_INTERNALS__) {
    console.warn("[instruckt] Tauri internals not found, shim not activated");
    return;
  }

  const invoke = window.__TAURI_INTERNALS__.invoke;

  // --- Fetch Shim ---
  // Intercept instruckt's HTTP API calls and route through Tauri IPC.

  const originalFetch = window.fetch;
  window.fetch = async function (input, init) {
    const url =
      typeof input === "string"
        ? input
        : input instanceof URL
          ? input.toString()
          : input instanceof Request
            ? input.url
            : null;

    if (!url) return originalFetch.call(this, input, init);

    const method = (init?.method || "GET").toUpperCase();

    // GET /instruckt/annotations
    if (url.match(/\/instruckt\/annotations\/?$/) && method === "GET") {
      const data = await invoke("plugin:instruckt|get_annotations");
      return new Response(JSON.stringify(data), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }

    // POST /instruckt/annotations
    if (url.match(/\/instruckt\/annotations\/?$/) && method === "POST") {
      const body =
        init?.body && typeof init.body === "string"
          ? JSON.parse(init.body)
          : init?.body || {};
      const data = await invoke("plugin:instruckt|create_annotation", {
        data: body,
      });
      return new Response(JSON.stringify(data), {
        status: 201,
        headers: { "Content-Type": "application/json" },
      });
    }

    // PATCH /instruckt/annotations/{id}
    const patchMatch = url.match(/\/instruckt\/annotations\/([^/?]+)/);
    if (patchMatch && method === "PATCH") {
      const id = patchMatch[1];
      const body =
        init?.body && typeof init.body === "string"
          ? JSON.parse(init.body)
          : init?.body || {};
      const data = await invoke("plugin:instruckt|update_annotation", {
        id,
        data: body,
      });
      return new Response(JSON.stringify(data), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }

    return originalFetch.call(this, input, init);
  };

  // --- Auto-initialize instruckt ---

  function initInstruckt() {
    if (typeof Instruckt === "undefined" || !Instruckt.init) {
      console.warn("[instruckt] Instruckt global not found, retrying...");
      setTimeout(initInstruckt, 100);
      return;
    }

    Instruckt.init({
      endpoint: "/instruckt",
      theme: "auto",
      position: "bottom-right",
      adapters: ["react"],
      mcp: true,
    });

    console.log("[instruckt] Initialized via Tauri plugin");

    // --- Inject visible drag handle into toolbar ---
    // The IIFE toolbar supports drag but has no visual affordance.
    // We inject a grip handle at the top so users know it's draggable.
    requestAnimationFrame(() => {
      const host = document.querySelector('[data-instruckt="toolbar"]');
      if (!host || !host.shadowRoot) return;

      const toolbar = host.shadowRoot.querySelector(".toolbar");
      if (!toolbar) return;

      // Add drag handle CSS
      const style = document.createElement("style");
      style.textContent = `
        .drag-handle {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 100%;
          height: 10px;
          cursor: grab;
          opacity: 0.35;
          transition: opacity 0.15s ease;
          flex-shrink: 0;
          margin-bottom: 2px;
        }
        .drag-handle:hover { opacity: 0.7; }
        .drag-handle:active { cursor: grabbing; opacity: 0.9; }
        .drag-handle svg { pointer-events: none; }
      `;
      host.shadowRoot.appendChild(style);

      // Create grip dots SVG (6-dot pattern)
      const handle = document.createElement("div");
      handle.className = "drag-handle";
      handle.setAttribute("aria-label", "Drag to reposition toolbar");
      handle.innerHTML = `<svg width="16" height="6" viewBox="0 0 16 6" fill="currentColor">
        <circle cx="4" cy="1.5" r="1.2"/><circle cx="8" cy="1.5" r="1.2"/><circle cx="12" cy="1.5" r="1.2"/>
        <circle cx="4" cy="4.5" r="1.2"/><circle cx="8" cy="4.5" r="1.2"/><circle cx="12" cy="4.5" r="1.2"/>
      </svg>`;

      // Insert at the top of the toolbar
      toolbar.prepend(handle);
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initInstruckt);
  } else {
    initInstruckt();
  }
})();
