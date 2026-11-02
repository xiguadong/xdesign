#!/usr/bin/env node
// xdesign · WishFlow Studio — standalone dev mock server.
//
// Serves web/assets/ with root-relative URLs plus a SPEC-shaped /api surface
// built from web/assets/js/sample-data.mjs. This directory is NOT embedded into
// the xdesign binary; it exists so the page can be iterated without cargo.
//
// Usage:
//   node web/dev/mock_server.mjs [--port 7789] [--latency 0] [--empty] [--offline]
//
//   --empty    a real, reachable project with no wishes (write actions enabled)
//   --offline  every /api route fails (the page falls back to labelled sample mode)
//   --latency  artificial per-request delay in ms, for loading-state screenshots

import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  SAMPLE_HEALTH,
  SAMPLE_FLOWS,
  SAMPLE_WISHES,
  SAMPLE_FLOWS as FLOWS_PAYLOAD,
  SAMPLE_CHECKPOINTS,
  getSampleDetail,
  getSampleWork,
  getSampleCheckResponse,
  getSampleInitResponse,
  getSampleMaterial,
  toCheckpointListEntry,
  findSampleCheckpointEntry,
} from "../assets/js/sample-data.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ASSETS_ROOT = path.resolve(HERE, "../assets");
const MAX_BODY_BYTES = 1024 * 1024;

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".webp": "image/webp",
  ".ico": "image/x-icon",
  ".woff2": "font/woff2",
  ".woff": "font/woff",
  ".txt": "text/plain; charset=utf-8",
  ".md": "text/plain; charset=utf-8",
};

function parseArgs(argv) {
  const options = { port: 7789, latency: 0, empty: false, offline: false };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--port") options.port = Number(argv[++index] || options.port);
    else if (arg === "--latency") options.latency = Number(argv[++index] || 0);
    else if (arg === "--empty") options.empty = true;
    else if (arg === "--offline") options.offline = true;
    else if (arg === "--help" || arg === "-h") {
      console.log(`Usage: node web/dev/mock_server.mjs [--port 7789] [--latency 0] [--empty] [--offline]`);
      process.exit(0);
    }
  }
  if (!Number.isFinite(options.port) || options.port < 0 || options.port > 65535) options.port = 7789;
  if (!Number.isFinite(options.latency) || options.latency < 0) options.latency = 0;
  return options;
}

const options = parseArgs(process.argv.slice(2));

/** Runtime wish list: seeded from the sample dataset (or empty with --empty) and
 *  extended by POST /api/wishes. --empty models a real, reachable, empty project. */
const wishes = options.empty ? [] : SAMPLE_WISHES.map((wish) => ({ ...wish }));

/** In-memory project checkpoint library. --empty models a project whose
 *  checkpoints/library.json does not exist yet (built-ins only). */
const library = (options.empty ? SAMPLE_CHECKPOINTS.filter((entry) => entry.builtin) : SAMPLE_CHECKPOINTS).map(
  (entry) => structuredClone(entry),
);

const LIBRARY_CHECKPOINT_RE = /^[a-z0-9][a-z0-9_-]{0,63}$/;

function libraryListReply() {
  return {
    schema_version: 1,
    library_uri: "checkpoints/library.json",
    issues: [],
    checkpoints: library.map((entry) => toCheckpointListEntry(entry)),
  };
}

function libraryShowReply(entry) {
  return {
    schema_version: 1,
    checkpoint: {
      ...structuredClone(entry),
      builtin: Boolean(entry.builtin),
      script_exists: entry.executor && entry.executor.mode === "script" ? true : false,
    },
  };
}

function validateCheckpointSave(body) {
  if (!body || typeof body !== "object" || Array.isArray(body)) return "The request body must be a checkpoint object.";
  const checkpointId = String(body.checkpoint_id || "");
  if (!LIBRARY_CHECKPOINT_RE.test(checkpointId)) {
    return "checkpoint_id must be a slug matching [a-z0-9][a-z0-9_-]{0,63}.";
  }
  const existing = library.find((entry) => entry.checkpoint_id === checkpointId);
  if (existing && existing.builtin) return "__builtin__";
  const kind = String(body.kind || "");
  if (!["syntactic", "semantic"].includes(kind)) return "kind must be syntactic or semantic.";
  const title = String(body.title || "").trim();
  if (title.length < 1 || title.length > 120) return "title must contain between 1 and 120 characters.";
  const description = String(body.description || "").trim();
  if (description.length < 1 || description.length > 1000) return "description must contain between 1 and 1000 characters.";
  const tags = Array.isArray(body.tags) ? body.tags : [];
  if (tags.length > 8) return "tags must contain at most 8 entries.";
  for (const tag of tags) {
    if (!/^[a-z0-9][a-z0-9-]{0,31}$/.test(String(tag))) return `tag ${JSON.stringify(tag)} is not a lowercase slug.`;
  }
  if (kind === "syntactic") {
    const rules = Array.isArray(body.rules) ? body.rules : [];
    if (rules.length === 0) return "a syntactic checkpoint needs at least one rule.";
    for (const rule of rules) {
      if (!rule || !LIBRARY_CHECKPOINT_RE.test(String(rule.rule_id || ""))) return "every rule needs a slug rule_id.";
      if (!String(rule.condition || "").trim()) return `rule ${rule.rule_id} needs a non-empty condition.`;
    }
    const inputs = Array.isArray(body.inputs) ? body.inputs : [];
    if (inputs.length === 0) return "a syntactic checkpoint needs at least one input.";
    const executor = body.executor || {};
    if (executor.mode !== "script") return "executor.mode must be script.";
    if (executor.mode === "script" && !String(executor.command_uri || "").endsWith(".py")) {
      return "a script executor needs a project-relative .py command_uri.";
    }
    if (body.create_script !== undefined && body.create_script !== "scaffold") return "create_script must be scaffold when present.";
    if (body.create_script === "scaffold") {
      const expected = `checkpoints/scripts/${checkpointId.replace(/-/g, "_")}.py`;
      if (executor.mode !== "script" || executor.command_uri !== expected) {
        return `create_script scaffold requires the script executor ${expected}.`;
      }
    }
  } else {
    const criteria = Array.isArray(body.criteria) ? body.criteria : [];
    if (criteria.length === 0) return "a semantic checkpoint needs at least one criterion.";
    const judges = body.judges || {};
    if (!judges.agent && !judges.human) return "a semantic checkpoint needs at least one permitted judge.";
    if ((Array.isArray(body.rules) && body.rules.length > 0) || (Array.isArray(body.inputs) && body.inputs.length > 0) || body.executor) {
      return "a semantic checkpoint must not carry rules, inputs or an executor.";
    }
  }
  return null;
}

function saveCheckpoint(body) {
  const now = new Date().toISOString();
  const checkpointId = String(body.checkpoint_id);
  const existing = library.find((entry) => entry.checkpoint_id === checkpointId);
  let scriptUri = null;
  const entry = {
    checkpoint_id: checkpointId,
    title: String(body.title).trim(),
    description: String(body.description).trim(),
    kind: body.kind,
    tags: Array.isArray(body.tags) ? body.tags : [],
    version: 1,
    created_at: now,
    updated_at: now,
    source: body.create_script === "scaffold" ? "generated" : body.source || "manual",
    builtin: false,
    history: [],
  };
  if (body.kind === "syntactic") {
    entry.rules = body.rules.map((rule) => ({ rule_id: rule.rule_id, condition: rule.condition, reads: Array.isArray(rule.reads) ? rule.reads : [] }));
    entry.inputs = Array.isArray(body.inputs) ? body.inputs : [];
    entry.executor = { mode: body.executor.mode, command_uri: body.executor.command_uri };
    if (body.create_script === "scaffold") scriptUri = entry.executor.command_uri;
  } else {
    entry.criteria = body.criteria.map((criterion) => String(criterion)).filter(Boolean);
    entry.judges = { agent: Boolean(body.judges.agent), human: Boolean(body.judges.human) };
  }
  if (existing) {
    const snapshot = {
      version: existing.version,
      updated_at: existing.updated_at,
      description: existing.description,
      kind: existing.kind,
      rules: existing.rules,
      inputs: existing.inputs,
      executor: existing.executor,
      criteria: existing.criteria,
      judges: existing.judges,
      tags: existing.tags,
    };
    entry.version = existing.version + 1;
    entry.created_at = existing.created_at;
    entry.source = existing.source === "generated" ? "generated" : entry.source;
    entry.history = [snapshot, ...(existing.history || [])].slice(0, 10);
    const index = library.indexOf(existing);
    library[index] = entry;
  } else {
    library.push(entry);
  }
  return { entry, scriptUri, created: !existing };
}

function envelope(res, status, payload) {
  const body = Buffer.from(JSON.stringify(payload));
  res.writeHead(status, {
    "Content-Type": "application/json; charset=utf-8",
    "Content-Length": body.length,
    "Cache-Control": "no-store",
    "X-Content-Type-Options": "nosniff",
  });
  res.end(body);
}

function ok(res, data) {
  envelope(res, 200, { ok: true, data });
}

function fail(res, status, code, message, detail = null) {
  envelope(res, status, { ok: false, error: { code, message, ...(detail === null ? {} : { detail }) } });
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    let size = 0;
    req.on("data", (chunk) => {
      size += chunk.length;
      if (size > MAX_BODY_BYTES) {
        reject(Object.assign(new Error("request body too large"), { code: "payload_too_large" }));
        req.destroy();
        return;
      }
      chunks.push(chunk);
    });
    req.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
    req.on("error", reject);
  });
}

function contentFor(relative) {
  const filePath = path.join(ASSETS_ROOT, relative);
  if (!filePath.startsWith(ASSETS_ROOT)) return null;
  try {
    const stat = fs.statSync(filePath);
    if (!stat.isFile()) return null;
    return { filePath, stat };
  } catch {
    return null;
  }
}

function serveStatic(res, pathname) {
  let relative = decodeURIComponent(pathname).replace(/^\/+/, "");
  if (relative === "" || relative === "/") relative = "index.html";
  let asset = contentFor(relative);
  if (!asset && !path.extname(relative)) {
    // Hash-route fallback: unknown extensionless paths serve the app shell.
    relative = "index.html";
    asset = contentFor(relative);
  }
  if (!asset) return false;
  const type = MIME[path.extname(relative).toLowerCase()] || "application/octet-stream";
  const body = fs.readFileSync(asset.filePath);
  res.writeHead(200, {
    "Content-Type": type,
    "Content-Length": body.length,
    "Cache-Control": "no-store",
    "X-Content-Type-Options": "nosniff",
  });
  res.end(body);
  return true;
}

function findWish(wishId) {
  return wishes.find((wish) => wish.wish_id === wishId) || null;
}

function wishIdFromPath(pathname) {
  const match = pathname.match(/^\/api\/wishes\/([^/]+)(\/[^/]+)?$/);
  if (!match) return null;
  return { wishId: decodeURIComponent(match[1]), action: match[2] ? match[2].slice(1) : null };
}

const WISH_ID_RE = /^[A-Za-z0-9][A-Za-z0-9_-]{0,127}$/;
const SYNTHETIC_GATE_IDS = [
  "simpleRequirementsAccepted",
  "simpleDesignAccepted",
  "simpleImplementationAccepted",
  "simpleRecordAccepted",
];
const SEMANTIC_GATE_IDS = ["verificationAccepted"];

function validateWishChecks(checks) {
  for (const entry of checks) {
    const checkpointId = String(entry.checkpoint_id || "");
    const allowed = SYNTHETIC_GATE_IDS.includes(checkpointId) || SEMANTIC_GATE_IDS.includes(checkpointId);
    if (!allowed) return `checkpoint ${checkpointId} cannot take a check binding.`;
    const mode = entry.mode || "default";
    if (mode === "default") continue;
    if (mode === "library") {
      const libraryEntry = library.find((candidate) => candidate.checkpoint_id === entry.library_id);
      if (!libraryEntry) return `checkpoint ${checkpointId} binds unknown library id ${entry.library_id || "(missing)"}.`;
      const expectedKind = SYNTHETIC_GATE_IDS.includes(checkpointId) ? "syntactic" : "semantic";
      if (libraryEntry.kind !== expectedKind) {
        return `checkpoint ${checkpointId} requires a ${expectedKind} library entry; ${entry.library_id} is ${libraryEntry.kind}.`;
      }
      if (libraryEntry.executor?.mode === "cli") {
        return `checkpoint ${checkpointId} cannot bind a historical Agent CLI entry.`;
      }
      continue;
    }
    return `checkpoint ${checkpointId} has unknown mode ${mode}.`;
  }
  return null;
}
const SYNTHETIC_IDS = [
  "simpleRequirementsAccepted",
  "simpleDesignAccepted",
  "simpleImplementationAccepted",
  "simpleRecordAccepted",
];

async function handleApi(req, res, pathname, searchParams) {
  if (options.offline) {
    fail(res, 503, "offline", "The dev mock server is running in --offline mode.");
    return;
  }

  // GET /api/health
  if (pathname === "/api/health" && req.method === "GET") {
    ok(res, { ...SAMPLE_HEALTH, now: new Date().toISOString() });
    return;
  }

  // GET /api/flows
  if (pathname === "/api/flows" && req.method === "GET") {
    ok(res, FLOWS_PAYLOAD);
    return;
  }

  // GET/POST /api/checkpoints and GET /api/checkpoints/{id}
  if (pathname === "/api/checkpoints") {
    if (req.method === "GET") {
      ok(res, libraryListReply());
      return;
    }
    if (req.method === "POST") {
      const raw = await readBody(req);
      let body;
      try {
        body = JSON.parse(raw || "{}");
      } catch {
        fail(res, 400, "invalid_json", "The request body is not valid JSON.");
        return;
      }
      const validation = validateCheckpointSave(body);
      if (validation === "__builtin__") {
        fail(res, 409, "builtin_checkpoint", `Checkpoint ${body.checkpoint_id} is a built-in and cannot be saved.`);
        return;
      }
      if (validation) {
        fail(res, 400, "invalid_checkpoint", validation);
        return;
      }
      const saved = saveCheckpoint(body);
      ok(res, {
        schema_version: 1,
        checkpoint: { ...structuredClone(saved.entry), builtin: false, script_exists: Boolean(saved.scriptUri) || Boolean(saved.entry.executor && saved.entry.executor.mode === "script") },
        script_uri: saved.scriptUri,
        created: saved.created,
        version: saved.entry.version,
      });
      return;
    }
    fail(res, 405, "method_not_allowed", "Only GET and POST are supported for /api/checkpoints.");
    return;
  }

  if (pathname.startsWith("/api/checkpoints/")) {
    if (req.method !== "GET") {
      fail(res, 405, "method_not_allowed", "Only GET is supported for a checkpoint definition.");
      return;
    }
    const checkpointId = decodeURIComponent(pathname.slice("/api/checkpoints/".length));
    const entry = library.find((candidate) => candidate.checkpoint_id === checkpointId);
    if (!entry) {
      fail(res, 404, "unknown_checkpoint", `No checkpoint with id ${checkpointId}.`);
      return;
    }
    ok(res, libraryShowReply(entry));
    return;
  }

  // GET/POST /api/wishes
  if (pathname === "/api/wishes") {
    if (req.method === "GET") {
      ok(res, { wishes });
      return;
    }
    if (req.method === "POST") {
      const raw = await readBody(req);
      let body;
      try {
        body = JSON.parse(raw || "{}");
      } catch {
        fail(res, 400, "invalid_json", "The request body is not valid JSON.");
        return;
      }
      const wishId = String(body.wish_id || "");
      const title = String(body.title || "");
      const description = String(body.description || "");
      const flowId = String(body.flow_id || "simple-flow");
      if (!WISH_ID_RE.test(wishId)) {
        fail(res, 400, "invalid_wish_id", "wish_id must match [A-Za-z0-9][A-Za-z0-9_-]{0,127}.");
        return;
      }
      if (title.length < 1 || title.length > 200) {
        fail(res, 400, "invalid_title", "title must contain between 1 and 200 characters.");
        return;
      }
      if (description.length > 4000) {
        fail(res, 400, "invalid_description", "description must be 4000 characters or fewer.");
        return;
      }
      if (!["simple-flow", "wish-verify"].includes(flowId)) {
        fail(res, 400, "invalid_flow", "flow_id must be simple-flow or wish-verify.");
        return;
      }
      if (findWish(wishId)) {
        fail(res, 409, "already_exists", `An item with wish id ${wishId} already exists.`);
        return;
      }
      const checks = Array.isArray(body.checks) ? body.checks : [];
      const checkError = validateWishChecks(checks);
      if (checkError) {
        fail(res, 400, "invalid_checks", checkError);
        return;
      }
      const created = {
        wish_id: wishId,
        title,
        description: description || null,
        flow_id: flowId,
        status: "received",
        current_stage: null,
        head_revision: 1,
        source: "store",
        updated_at: new Date().toISOString(),
        checks: checks.map((entry) => ({ ...entry })),
      };
      wishes.push(created);
      // The detail/work/material builders are shared with the page's sample
      // mode and resolve wishes through SAMPLE_WISHES, so register it there too.
      SAMPLE_WISHES.push(created);
      ok(res, getSampleInitResponse({ ...body, wish_id: wishId, title, description, flow_id: flowId }));
      return;
    }
    fail(res, 405, "method_not_allowed", "Only GET and POST are supported for /api/wishes.");
    return;
  }

  const parsed = wishIdFromPath(pathname);
  if (!parsed) {
    fail(res, 404, "unknown_route", `Unknown API route: ${pathname}`);
    return;
  }
  const { wishId, action } = parsed;

  if (!findWish(wishId)) {
    fail(res, 404, "unknown_wish", `No wish item or legacy trace with id ${wishId}.`);
    return;
  }

  // GET /api/wishes/{id}
  if (!action && req.method === "GET") {
    const detail = getSampleDetail(wishId);
    if (!detail) {
      fail(res, 404, "unknown_wish", `No wish item or legacy trace with id ${wishId}.`);
      return;
    }
    ok(res, detail);
    return;
  }

  // GET /api/wishes/{id}/work
  if (action === "work" && req.method === "GET") {
    const work = getSampleWork(wishId);
    if (!work) {
      fail(res, 404, "unknown_wish", `No wish item or legacy trace with id ${wishId}.`);
      return;
    }
    ok(res, work);
    return;
  }

  // POST /api/wishes/{id}/check
  if (action === "check" && req.method === "POST") {
    const wish = findWish(wishId);
    if (wish.status === "delivered") {
      fail(res, 409, "already_delivered", "The wish is delivered; there is no stage left to check.");
      return;
    }
    const raw = await readBody(req);
    let body;
    try {
      body = JSON.parse(raw || "{}");
    } catch {
      fail(res, 400, "invalid_json", "The request body is not valid JSON.");
      return;
    }
    const response = getSampleCheckResponse(wishId, body);
    if (!response) {
      fail(res, 404, "unknown_wish", `No wish item or legacy trace with id ${wishId}.`);
      return;
    }
    ok(res, response);
    return;
  }

  // GET /api/wishes/{id}/material?uri=...
  if (action === "material" && req.method === "GET") {
    const uri = searchParams.get("uri") || "";
    if (!uri) {
      fail(res, 400, "missing_uri", "The material endpoint requires a uri query parameter.");
      return;
    }
    const material = getSampleMaterial(wishId, uri);
    if (!material) {
      fail(res, 404, "unknown_material", `The head revision has no material at ${uri}.`);
      return;
    }
    const body = Buffer.from(material.text);
    res.writeHead(200, {
      "Content-Type": material.contentType,
      "Content-Length": body.length,
      "Cache-Control": "no-store",
      "X-Content-Type-Options": "nosniff",
    });
    res.end(body);
    return;
  }

  fail(res, 405, "method_not_allowed", `Unsupported method ${req.method} for ${pathname}.`);
}

const server = http.createServer((req, res) => {
  const url = new URL(req.url || "/", `http://${req.headers.host || "127.0.0.1"}`);
  const pathname = url.pathname;

  const respond = () => {
    if (pathname.startsWith("/api/")) {
      handleApi(req, res, pathname, url.searchParams).catch((error) => {
        if (res.headersSent) return;
        if (error && error.code === "payload_too_large") {
          fail(res, 413, "payload_too_large", "The request body exceeds 1 MiB.");
        } else {
          fail(res, 500, "internal_error", String((error && error.message) || error));
        }
      });
      return;
    }
    if (req.method !== "GET") {
      res.writeHead(405, { Allow: "GET" });
      res.end("Method not allowed");
      return;
    }
    if (!serveStatic(res, pathname)) {
      res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
      res.end(`Not found: ${pathname}`);
    }
  };

  if (options.latency > 0) setTimeout(respond, options.latency);
  else respond();
});

server.listen(options.port, "127.0.0.1", () => {
  const address = server.address();
  const port = typeof address === "object" && address ? address.port : options.port;
  console.log(`xdesign web dev mock listening on http://127.0.0.1:${port}`);
  console.log(`  assets: ${ASSETS_ROOT}`);
  if (options.empty) console.log("  mode: empty wish list (page falls back to sample mode)");
  if (options.offline) console.log("  mode: offline API (page falls back to sample mode)");
  if (options.latency) console.log(`  latency: ${options.latency}ms`);
});

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => {
    server.close(() => process.exit(0));
    setTimeout(() => process.exit(0), 500).unref();
  });
}
