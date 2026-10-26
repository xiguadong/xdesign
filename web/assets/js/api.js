// xdesign · WishFlow Studio — API client.
//
// All requests go to the embedding `xdesign web` server with root-relative
// URLs. Responses use the envelope {ok:true,data} | {ok:false,error}; this
// module unwraps `data`, turns failures into ApiError instances, and never
// leaves a request hanging (per-request timeout + abort).

export class ApiError extends Error {
  constructor(message, { status = 0, code = "request_failed", detail = null } = {}) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.code = code;
    this.detail = detail;
  }
}

const DEFAULT_TIMEOUT_MS = 25_000;

async function request(path, { method = "GET", body = null, timeout = DEFAULT_TIMEOUT_MS, raw = false } = {}) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeout);
  let response;
  try {
    response = await fetch(path, {
      method,
      headers: body ? { "Content-Type": "application/json" } : undefined,
      body: body === null ? undefined : JSON.stringify(body),
      signal: controller.signal,
      cache: "no-store",
      credentials: "same-origin",
    });
  } catch (error) {
    if (error && error.name === "AbortError") {
      throw new ApiError("The local xdesign server did not respond in time.", { code: "timeout" });
    }
    throw new ApiError("Cannot reach the local xdesign server.", { code: "unreachable", detail: String(error) });
  } finally {
    clearTimeout(timer);
  }

  const contentType = response.headers.get("content-type") || "";

  if (raw) {
    if (!response.ok) {
      let message = `Request failed with HTTP ${response.status}.`;
      try {
        const payload = await response.json();
        if (payload && payload.ok === false && payload.error) message = payload.error.message || message;
      } catch {
        /* keep the status-based message */
      }
      throw new ApiError(message, { status: response.status });
    }
    return {
      text: await response.text(),
      contentType,
      status: response.status,
    };
  }

  let payload = null;
  if (contentType.includes("application/json")) {
    try {
      payload = await response.json();
    } catch {
      throw new ApiError("The server returned malformed JSON.", { status: response.status, code: "bad_json" });
    }
  }

  if (!response.ok || !payload || payload.ok === false) {
    const error = (payload && payload.error) || {};
    throw new ApiError(error.message || `Request failed with HTTP ${response.status}.`, {
      status: response.status,
      code: error.code || (response.ok ? "bad_envelope" : `http_${response.status}`),
      detail: error.detail ?? null,
    });
  }

  return payload.data;
}

export const api = {
  health: () => request("/api/health"),
  flows: () => request("/api/flows"),
  wishes: () => request("/api/wishes"),
  wish: (wishId) => request(`/api/wishes/${encodeURIComponent(wishId)}`),
  work: (wishId) => request(`/api/wishes/${encodeURIComponent(wishId)}/work`),
  check: (wishId, body) =>
    request(`/api/wishes/${encodeURIComponent(wishId)}/check`, { method: "POST", body, timeout: 620_000 }),
  create: (body) => request("/api/wishes", { method: "POST", body, timeout: 620_000 }),
  checkpoints: () => request("/api/checkpoints"),
  checkpoint: (checkpointId) => request(`/api/checkpoints/${encodeURIComponent(checkpointId)}`),
  createCheckpoint: (body) =>
    request("/api/checkpoints", { method: "POST", body, timeout: 120_000 }),
  material: (wishId, uri) =>
    request(`/api/wishes/${encodeURIComponent(wishId)}/material?uri=${encodeURIComponent(uri)}`, {
      raw: true,
    }),
};
