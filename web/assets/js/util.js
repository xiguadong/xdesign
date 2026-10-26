// xdesign · WishFlow Studio — shared UI utilities, icons and formatters.
// No dependencies; everything is local and self-contained.

const ICON_PATHS = {
  overview:
    '<circle cx="12" cy="12" r="9"/><path d="m15.6 8.4-2.3 5.2-5.2 2.3 2.3-5.2z"/>',
  wishes:
    '<path d="M9 6h12M9 12h12M9 18h12"/><circle cx="4.4" cy="6" r="1"/><circle cx="4.4" cy="12" r="1"/><circle cx="4.4" cy="18" r="1"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  flows:
    '<circle cx="6" cy="5" r="2"/><circle cx="18" cy="7" r="2"/><circle cx="6" cy="19" r="2"/><path d="M6 7v10M6 12h6.5A4.5 4.5 0 0 0 17 7.5"/>',
  refresh:
    '<path d="M20 11a8 8 0 1 0-2.35 5.65"/><path d="M20 5v6h-6"/>',
  search: '<circle cx="11" cy="11" r="6.5"/><path d="m20 20-4.4-4.4"/>',
  close: '<path d="M6 6l12 12M18 6 6 18"/>',
  check: '<path d="m5 12.5 4.5 4.5L19 7.5"/>',
  "check-circle":
    '<circle cx="12" cy="12" r="9"/><path d="m8.2 12.4 2.6 2.6 5-5.2"/>',
  "alert-triangle":
    '<path d="M12 3.5 2.8 20h18.4z"/><path d="M12 9.5v4.5"/><circle cx="12" cy="17" r=".6" fill="currentColor" stroke="none"/>',
  "alert-circle":
    '<circle cx="12" cy="12" r="9"/><path d="M12 7.5v5.5"/><circle cx="12" cy="16.5" r=".6" fill="currentColor" stroke="none"/>',
  clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5.2l3.2 2"/>',
  cpu:
    '<rect x="7" y="7" width="10" height="10" rx="2.2"/><path d="M10 2.5v3M14 2.5v3M10 18.5v3M14 18.5v3M2.5 10h3M2.5 14h3M18.5 10h3M18.5 14h3"/>',
  database:
    '<ellipse cx="12" cy="5.8" rx="7" ry="2.8"/><path d="M5 5.8v6.2c0 1.55 3.13 2.8 7 2.8s7-1.25 7-2.8V5.8"/><path d="M5 12v6.2c0 1.55 3.13 2.8 7 2.8s7-1.25 7-2.8V12"/>',
  "arrow-right": '<path d="M5 12h13.5M13 6.5l5.5 5.5-5.5 5.5"/>',
  "arrow-up-right": '<path d="M7 17 17 7M9 7h8v8"/>',
  folder:
    '<path d="M3.2 7.2A2 2 0 0 1 5.2 5.2h3.6l2 2h8A2 2 0 0 1 20.8 9.2v7.6a2 2 0 0 1-2 2H5.2a2 2 0 0 1-2-2z"/>',
  shield:
    '<path d="M12 3 4.8 6v5.3c0 4.3 3.1 7.6 7.2 9.2 4.1-1.6 7.2-4.9 7.2-9.2V6z"/><path d="m8.8 11.8 2.2 2.3 4.2-4.4"/>',
  play: '<path d="M8.2 5.2 18.4 12 8.2 18.8z"/>',
  file: '<path d="M13.5 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8.5z"/><path d="M13.5 3v5.5H19"/>',
  code: '<path d="m8.5 8-4 4 4 4M15.5 8l4 4-4 4"/>',
  sparkle:
    '<path d="m12 3.2 2 5.3 5.3 2-5.3 2-2 5.3-2-5.3-5.3-2 5.3-2z"/>',
  layers:
    '<path d="m12 3.4 8.6 4.6L12 12.6 3.4 8z"/><path d="m3.4 12.4 8.6 4.6 8.6-4.6"/><path d="m3.4 16.4 8.6 4.6 8.6-4.6"/>',
  library:
    '<path d="M4 5.5A2.5 2.5 0 0 1 6.5 3H20v15.5H6.5A2.5 2.5 0 0 0 4 21z"/><path d="M4 18.5A2.5 2.5 0 0 1 6.5 16H20"/><path d="M8 7.5h8M8 11h6"/>',
  "book-open":
    '<path d="M12 6.5C10.4 5 8.4 4.5 5.5 4.5v13c3 0 5 .5 6.5 2 1.5-1.5 3.5-2 6.5-2v-13c-2.9 0-4.9.5-6.5 2z"/><path d="M12 6.5v13"/>',
  handoff:
    '<path d="M4 12h11M9.5 6.5 15 12l-5.5 5.5"/><path d="M18 4.5v15"/>',
  "shield-check":
    '<path d="M12 3 4.8 6v5.3c0 4.3 3.1 7.6 7.2 9.2 4.1-1.6 7.2-4.9 7.2-9.2V6z"/><path d="m8.8 11.8 2.2 2.3 4.2-4.4"/>',
  diamond: '<path d="M12 2.8 21.2 12 12 21.2 2.8 12z"/>',
  "git-commit":
    '<circle cx="12" cy="12" r="3.2"/><path d="M2.8 12h6M15.2 12h6"/>',
  terminal: '<path d="m5.5 8 4 4-4 4M12.5 16H18"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v5.5"/><circle cx="12" cy="7.8" r=".65" fill="currentColor" stroke="none"/>',
  chevron: '<path d="m9 6 6 6-6 6"/>',
  "chevron-down": '<path d="m6 9 6 6 6-6"/>',
  "chevron-left": '<path d="m15 6-6 6 6 6"/>',
  filter: '<path d="M4 5.5h16l-6.2 7.2V19l-3.6 1.8v-8.1z"/>',
  copy:
    '<rect x="9" y="9" width="11" height="11" rx="2.2"/><path d="M5 14.5V5.2A2.2 2.2 0 0 1 7.2 3h9.3"/>',
  eye: '<path d="M2.5 12S6 5.8 12 5.8 21.5 12 21.5 12 18 18.2 12 18.2 2.5 12 2.5 12z"/><circle cx="12" cy="12" r="2.8"/>',
  lock: '<rect x="4.8" y="10.2" width="14.4" height="10" rx="2.2"/><path d="M8 10.2V7.4a4 4 0 0 1 8 0v2.8"/>',
  "inbox":
    '<path d="M4 13.5 6.2 5.2A2 2 0 0 1 8.1 3.7h7.8a2 2 0 0 1 1.9 1.5L20 13.5"/><path d="M4 13.5v4.3a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-4.3h-5.2L13 16h-2l-1.8-2.5z"/>',
  "bar-chart": '<path d="M5 20V10M12 20V4M19 20v-7"/>',
  pulse: '<path d="M3 12h4l2.5-6 4 12 2.5-6h5"/>',
  sliders: '<path d="M4 8h9M17 8h3M4 16h3M11 16h9"/><circle cx="15" cy="8" r="2.2"/><circle cx="7" cy="16" r="2.2"/>',
  external: '<path d="M14 4h6v6"/><path d="m20 4-9 9"/><path d="M18.5 14.5V18a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V7.5a2 2 0 0 1 2-2h3.5"/>',
  flag: '<path d="M6 3.5v17"/><path d="M6 4.5h11.5l-2.2 4 2.2 4H6z"/>',
  send: '<path d="m20.5 3.5-7.2 17-2.6-7.2-7.2-2.6z"/><path d="M20.5 3.5 10.7 13.3"/>',
};

export function icon(name, extra = "") {
  const path = ICON_PATHS[name];
  if (!path) return "";
  return `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" class="icon-svg${extra ? " " + extra : ""}" aria-hidden="true">${path}</svg>`;
}

export function hydrateIcons(root = document) {
  root.querySelectorAll("[data-icon]").forEach((el) => {
    if (el.children.length > 0) return;
    el.innerHTML = icon(el.dataset.icon);
  });
}

export function esc(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

/** Wrap text in an escaped span, preserving nothing else. */
export function e(value) {
  return esc(value);
}

export function attr(value) {
  return esc(value);
}

const STATUS_META = {
  delivered: { label: "delivered", tone: "green" },
  active: { label: "active", tone: "cyan" },
  blocked: { label: "blocked", tone: "rose" },
  received: { label: "received", tone: "slate" },
};

export function statusMeta(status) {
  return STATUS_META[status] || { label: status || "unknown", tone: "slate" };
}

export function statusChip(status, extra = "") {
  const meta = statusMeta(status);
  return `<span class="chip chip--status tone-${meta.tone}${extra ? " " + extra : ""}"><span class="dot"></span>${esc(meta.label)}</span>`;
}

export function outcomeTone(outcome) {
  switch (outcome) {
    case "passed":
      return "green";
    case "failed":
      return "rose";
    case "unevaluated":
      return "amber";
    default:
      return "slate";
  }
}

export function outcomeChip(outcome, extra = "") {
  const tone = outcomeTone(outcome);
  return `<span class="badge tone-${tone}${extra ? " " + extra : ""}"><span class="dot"></span>${esc(outcome || "none")}</span>`;
}

export function flowChip(flowId, extra = "") {
  return `<span class="chip chip--flow${extra ? " " + extra : ""}">${esc(flowId || "—")}</span>`;
}

export function sourceChip(source) {
  if (!source) return "";
  return `<span class="chip chip--source">${esc(source)}</span>`;
}

export function revChip(revision) {
  if (revision === null || revision === undefined) return `<span class="chip chip--rev">no head</span>`;
  return `<span class="chip chip--rev">rev ${esc(revision)}</span>`;
}

const DIVISIONS = [
  { sec: 60, unit: "s" },
  { sec: 3600, unit: "m" },
  { sec: 86400, unit: "h" },
  { sec: 2592000, unit: "d" },
];

export function relativeTime(input, now = Date.now()) {
  if (!input) return "—";
  const ts = typeof input === "number" ? input : Date.parse(input);
  if (!Number.isFinite(ts)) return "—";
  let diff = Math.max(0, Math.round((now - ts) / 1000));
  if (diff < 5) return "just now";
  if (diff < 60) return `${diff}s ago`;
  if (diff < 3600) return `${Math.round(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.round(diff / 3600)}h ago`;
  if (diff < 2592000) return `${Math.round(diff / 86400)}d ago`;
  return new Date(ts).toLocaleDateString("en-US", { month: "short", day: "numeric" });
}

export function absoluteTime(input) {
  if (!input) return "—";
  const ts = typeof input === "number" ? input : Date.parse(input);
  if (!Number.isFinite(ts)) return "—";
  return new Date(ts).toISOString().replace("T", " ").replace(/\.\d+Z$/, " UTC");
}

export function shortHash(hash) {
  if (!hash) return "—";
  return String(hash).slice(0, 12);
}

/** Human age label for a wish row: relative when the store reports a time,
 *  otherwise an explicit "legacy" / "no timestamp" marker. */
export function wishAge(wish) {
  if (!wish) return "—";
  if (wish.updated_at) return relativeTime(wish.updated_at);
  return wish.source === "legacy" ? "legacy" : "no timestamp";
}

export function truncate(value, max = 120) {
  const text = String(value ?? "");
  return text.length > max ? `${text.slice(0, max - 1)}…` : text;
}

export function debounce(fn, delay = 160) {
  let timer = null;
  const wrapped = (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), delay);
  };
  wrapped.cancel = () => clearTimeout(timer);
  return wrapped;
}

export function uid(prefix = "id") {
  return `${prefix}-${Math.random().toString(36).slice(2, 9)}`;
}

export function classNames(...values) {
  return values.filter(Boolean).join(" ");
}

export function slugId(value) {
  return String(value ?? "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 48);
}

export function suggestWishId() {
  const date = new Date();
  const y = String(date.getFullYear());
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `wish-${y}${m}${d}-01`;
}

export function parseHash(hash) {
  let raw = String(hash || "").replace(/^#/, "");
  if (!raw || raw === "/") raw = "/overview";
  const [pathPart, queryPart] = raw.split("?");
  const segments = pathPart.split("/").filter(Boolean).map(decodeURIComponent);
  const params = new URLSearchParams(queryPart || "");
  let route = segments[0] || "overview";
  let param = segments[1] || null;
  if (route === "wishes" && param) route = "wish-detail";
  return { route, param, segments, params, path: pathPart || "/overview" };
}

export function buildHash(path, params) {
  const search = params ? new URLSearchParams(params).toString() : "";
  return `#${path}${search ? "?" + search : ""}`;
}

export function pluralize(count, singular, plural) {
  return `${count} ${count === 1 ? singular : plural || singular + "s"}`;
}

/** Copy text to the clipboard with a graceful fallback. */
export async function copyText(text) {
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    /* fall through to the hidden-textarea fallback */
  }
  try {
    const area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.opacity = "0";
    document.body.appendChild(area);
    area.select();
    const ok = document.execCommand("copy");
    area.remove();
    return ok;
  } catch {
    return false;
  }
}

export function prettyJson(value) {
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

export function parseJsonLoose(text) {
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

export function sumBy(list, fn) {
  return (list || []).reduce((total, item) => total + (fn(item) || 0), 0);
}

export function countBy(list, fn) {
  const counts = new Map();
  for (const item of list || []) {
    const key = fn(item);
    counts.set(key, (counts.get(key) || 0) + 1);
  }
  return counts;
}

export function sortWishes(wishes) {
  const priority = { blocked: 0, active: 1, received: 2, delivered: 3 };
  return [...(wishes || [])].sort((a, b) => {
    const pa = priority[a.status] ?? 9;
    const pb = priority[b.status] ?? 9;
    if (pa !== pb) return pa - pb;
    return String(a.wish_id).localeCompare(String(b.wish_id));
  });
}

/** Very small deterministic digest-shaped string for sample material only. */
export function pseudoDigest(seed) {
  let h1 = 0x811c9dc5;
  let h2 = 0x01000193;
  const text = String(seed);
  for (let i = 0; i < text.length; i += 1) {
    h1 ^= text.charCodeAt(i);
    h1 = Math.imul(h1, 0x01000193);
    h2 = Math.imul(h2 ^ text.charCodeAt(i), 0x85ebca6b);
  }
  const part = (n) => (n >>> 0).toString(16).padStart(8, "0");
  let out = "";
  let a = h1;
  let b = h2;
  for (let i = 0; i < 8; i += 1) {
    out += part(a) + part(b);
    a = Math.imul(a ^ (b >>> 5), 0x27d4eb2d);
    b = Math.imul(b ^ (a >>> 7), 0x165667b1);
  }
  return out.slice(0, 64);
}
