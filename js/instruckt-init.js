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
      const annotations = await invoke("plugin:instruckt|get_annotations");
      return new Response(JSON.stringify(annotations), {
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
      const created = await invoke("plugin:instruckt|create_annotation", {
        input: body,
      });
      return new Response(JSON.stringify(created), {
        status: 201,
        headers: { "Content-Type": "application/json" },
      });
    }

    // Match annotation ID routes for PATCH and DELETE
    const idMatch = url.match(/\/instruckt\/annotations\/([^/?]+)/);

    // PATCH /instruckt/annotations/{id}
    if (idMatch && method === "PATCH") {
      const id = idMatch[1];
      const body =
        init?.body && typeof init.body === "string"
          ? JSON.parse(init.body)
          : init?.body || {};
      const updated = await invoke("plugin:instruckt|update_annotation", {
        id,
        input: body,
      });
      return new Response(JSON.stringify(updated), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }

    // DELETE /instruckt/annotations/{id}
    if (idMatch && method === "DELETE") {
      const id = idMatch[1];
      await invoke("plugin:instruckt|delete_annotation", { id });
      return new Response(null, { status: 204 });
    }

    return originalFetch.call(this, input, init);
  };

  // --- Auto-initialize instruckt ---

  let retryCount = 0;
  const MAX_RETRIES = 50;

  function initInstruckt() {
    if (typeof Instruckt === "undefined" || !Instruckt.init) {
      if (retryCount >= MAX_RETRIES) {
        console.warn("[instruckt] Failed to initialize after maximum retries");
        return;
      }
      retryCount++;
      console.warn("[instruckt] Instruckt global not found, retrying...");
      setTimeout(initInstruckt, 100);
      return;
    }

    Instruckt.init({
      endpoint: "/instruckt",
      theme: "auto",
      position: "bottom-right",
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
