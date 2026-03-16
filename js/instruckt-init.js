/**
 * Tauri IPC shim for instruckt.
 *
 * This script is injected into every webview via `js_init_script`.
 * It provides a global `__instrucktTauriShim` that routes instruckt's
 * API calls through Tauri's IPC instead of HTTP fetch.
 *
 * It also monkey-patches `fetch()` so that any requests to `/instruckt/*`
 * endpoints are intercepted and routed through Tauri IPC — this way the
 * instruckt JS core works without modification.
 */
(function () {
  "use strict";

  // Only activate in dev builds — this script is only injected when
  // cfg!(debug_assertions) is true, but double-check just in case.
  if (!window.__TAURI__) {
    console.warn("[instruckt] Tauri API not found, shim not activated");
    return;
  }

  const invoke = window.__TAURI__.core.invoke;

  // Expose the shim globally for direct use
  window.__instrucktTauriShim = {
    async getAnnotations() {
      return await invoke("plugin:instruckt|get_annotations");
    },
    async createAnnotation(data) {
      return await invoke("plugin:instruckt|create_annotation", { data });
    },
    async updateAnnotation(id, data) {
      return await invoke("plugin:instruckt|update_annotation", { id, data });
    },
  };

  // Monkey-patch fetch to intercept instruckt API calls.
  // The instruckt JS core uses fetch() to call HTTP endpoints like:
  //   GET  /instruckt/annotations
  //   POST /instruckt/annotations
  //   PATCH /instruckt/annotations/{id}
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

    // Match instruckt API routes
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
      const body = init?.body && typeof init.body === "string" ? JSON.parse(init.body) : (init?.body || {});
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
      const body = init?.body && typeof init.body === "string" ? JSON.parse(init.body) : (init?.body || {});
      const data = await invoke("plugin:instruckt|update_annotation", {
        id,
        data: body,
      });
      return new Response(JSON.stringify(data), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }

    // Not an instruckt route — pass through to real fetch
    return originalFetch.call(this, input, init);
  };

  console.log("[instruckt] Tauri IPC shim activated");
})();
