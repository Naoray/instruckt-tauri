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
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initInstruckt);
  } else {
    initInstruckt();
  }
})();
