// xdesign · WishFlow Studio — shared UI components: toasts, modal, material
// viewer, status/flow chips, wish rows, pipeline miniatures and empty states.

import { api } from "./api.js";
import {
  esc,
  icon,
  relativeTime,
  statusChip,
  flowChip,
  revChip,
  sourceChip,
  statusMeta,
  wishAge,
  outcomeChip,
  prettyJson,
  copyText,
} from "./util.js";
import { getSampleMaterial } from "./sample-data.mjs";

/* ------------------------------------------------------------------ */
/* Toasts                                                             */
/* ------------------------------------------------------------------ */

const TOAST_ICONS = {
  ok: "check-circle",
  error: "alert-circle",
  warn: "alert-triangle",
  info: "info",
};

const activeToasts = [];

export function toast({ title, text = "", tone = "cyan", iconName = null, timeout = 4800 } = {}) {
  const root = document.getElementById("toast-root");
  if (!root) return;
  // Collapse duplicates and keep the stack short so it never buries a form.
  const duplicate = activeToasts.find(
    (entry) => entry.title === title && entry.text === text && Date.now() - entry.at < 4000,
  );
  if (duplicate) return duplicate.remove;
  while (activeToasts.length >= 3) {
    const oldest = activeToasts.shift();
    oldest.remove();
  }
  const node = document.createElement("div");
  const iconMap = { cyan: "info", green: "ok", rose: "error", amber: "warn", violet: "info" };
  const kind = iconName || TOAST_ICONS[iconMap[tone]] || "info";
  node.className = `toast tone-${tone}`;
  node.setAttribute("role", "status");
  node.innerHTML = `
    <span class="toast__icon">${icon(kind)}</span>
    <span class="toast__body">
      <span class="toast__title">${esc(title)}</span>
      ${text ? `<span class="toast__text">${esc(text)}</span>` : ""}
    </span>`;
  root.appendChild(node);
  const remove = () => {
    const index = activeToasts.findIndex((entry) => entry.node === node);
    if (index >= 0) activeToasts.splice(index, 1);
    node.classList.add("is-leaving");
    setTimeout(() => node.remove(), 240);
  };
  const timer = setTimeout(remove, timeout);
  node.addEventListener("click", () => {
    clearTimeout(timer);
    remove();
  });
  activeToasts.push({ node, title, text, at: Date.now(), remove });
  return remove;
}

export function toastOk(title, text = "") {
  return toast({ title, text, tone: "green" });
}
export function toastError(title, text = "") {
  return toast({ title, text, tone: "rose", timeout: 7000 });
}
export function toastWarn(title, text = "") {
  return toast({ title, text, tone: "amber", timeout: 6500 });
}
export function toastInfo(title, text = "") {
  return toast({ title, text, tone: "cyan" });
}

/* ------------------------------------------------------------------ */
/* Modal                                                              */
/* ------------------------------------------------------------------ */

let modalCloser = null;

export function closeModal() {
  if (modalCloser) {
    const closer = modalCloser;
    modalCloser = null;
    closer();
  }
}

export function openModal({ title, subtitle = "", bodyHtml = "", footerHtml = "", size = "md", iconName = "file", onMount = null }) {
  const root = document.getElementById("modal-root");
  if (!root) return null;
  closeModal();
  const previousFocus = document.activeElement;
  root.className = "modal-root is-open";
  root.innerHTML = `
    <div class="modal__backdrop" data-modal-backdrop></div>
    <div class="modal" role="dialog" aria-modal="true" aria-label="${esc(title)}">
      <div class="modal__card ${size === "sm" ? "modal__card--sm" : ""}">
        <div class="modal__head">
          <div class="modal__titles">
            <div class="modal__title">${icon(iconName)}<span>${esc(title)}</span></div>
            <div class="modal__sub">${esc(subtitle)}</div>
          </div>
          <button class="icon-btn modal__close" type="button" data-modal-close aria-label="Close dialog">${icon("close")}</button>
        </div>
        <div class="modal__body" data-modal-body>${bodyHtml}</div>
        <div class="modal__foot">
          ${footerHtml || '<span class="modal__foot-meta">Esc to close</span>'}
          <button class="btn btn--ghost btn--sm" type="button" data-modal-close>Close</button>
        </div>
      </div>
    </div>`;

  const panel = root.querySelector(".modal__card");

  const onKeyDown = (event) => {
    if (event.key === "Escape") {
      event.preventDefault();
      closeModal();
      return;
    }
    if (event.key !== "Tab") return;
    const focusable = panel.querySelectorAll(
      'button:not(:disabled), [href], input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])',
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  };

  const closer = () => {
    document.removeEventListener("keydown", onKeyDown);
    root.classList.remove("is-open");
    root.innerHTML = "";
    modalCloser = null;
    if (previousFocus && typeof previousFocus.focus === "function") {
      previousFocus.focus({ preventScroll: true });
    }
  };
  modalCloser = closer;
  document.addEventListener("keydown", onKeyDown);
  root.querySelectorAll("[data-modal-close], [data-modal-backdrop]").forEach((element) => {
    element.addEventListener("click", () => closeModal());
  });
  requestAnimationFrame(() => {
    const target = panel.querySelector("button, input, select, textarea, [tabindex]") || panel;
    if (target && typeof target.focus === "function") target.focus({ preventScroll: true });
  });
  if (onMount) onMount(panel);
  return { close: closer, panel };
}

/* ------------------------------------------------------------------ */
/* Material viewer                                                    */
/* ------------------------------------------------------------------ */

export async function openMaterialViewer(ctx, wishId, uri) {
  const loadingBody = `
    <div class="loading-block"><span class="spinner" aria-hidden="true"></span>
      <span>Loading material <span class="mono">${esc(uri)}</span>…</span></div>`;
  const modal = openModal({
    title: "Material viewer",
    subtitle: uri,
    bodyHtml: loadingBody,
    iconName: "file",
    footerHtml: `<span class="modal__foot-meta">read-only · head revision of ${esc(wishId)}</span>`,
  });
  if (!modal) return;
  const body = modal.panel.querySelector("[data-modal-body]");
  let material = null;
  let failure = null;
  try {
    if (ctx.state.sampleMode) {
      material = getSampleMaterial(wishId, uri);
      if (!material) throw new Error("This sample dataset has no material for that URI.");
    } else {
      const raw = await api.material(wishId, uri);
      material = { contentType: raw.contentType, text: raw.text };
    }
  } catch (error) {
    failure = error;
  }
  if (!body || !document.body.contains(body)) return;

  if (failure) {
    const message = failure.message || "The material could not be loaded.";
    body.innerHTML = `
      <div class="material-error">
        ${icon("alert-circle")}
        <div>
          <strong>Material unavailable</strong>
          <div class="text-faint" style="font-size:11.5px;margin-top:2px">${esc(message)}</div>
        </div>
      </div>
      <pre class="material-pre" style="margin-top:14px">${esc(uri)}</pre>`;
    return;
  }

  const contentType = material.contentType || "";
  const isJson = contentType.includes("json");
  let text = material.text || "";
  if (isJson) {
    try {
      text = prettyJson(JSON.parse(text));
    } catch {
      /* keep the raw text */
    }
  }
  const lineCount = text.split("\n").length;
  body.innerHTML = `
    <pre class="material-pre${isJson ? " is-json" : ""}" tabindex="0">${esc(text)}</pre>
    <div class="row" style="margin-top:12px;justify-content:flex-end;gap:8px">
      <span class="modal__foot-meta">${esc(contentType || "text/plain")} · ${lineCount} lines</span>
      <button class="btn btn--ghost btn--sm" type="button" data-copy-material>${icon("copy")}<span>Copy</span></button>
    </div>`;
  const copyButton = body.querySelector("[data-copy-material]");
  if (copyButton) {
    copyButton.addEventListener("click", async () => {
      const ok = await copyText(text);
      if (ok) toastOk("Copied to clipboard", uri);
      else toastError("Copy failed", "The browser blocked clipboard access.");
    });
  }
}

/* ------------------------------------------------------------------ */
/* Empty / error states                                               */
/* ------------------------------------------------------------------ */

const EMPTY_ART = `<svg class="empty__art" viewBox="0 0 128 96" fill="none" aria-hidden="true">
  <defs>
    <linearGradient id="emptyArtGrad" x1="16" y1="12" x2="112" y2="84" gradientUnits="userSpaceOnUse">
      <stop stop-color="#22d3ee" stop-opacity=".8"/><stop offset="1" stop-color="#a78bfa" stop-opacity=".8"/>
    </linearGradient>
  </defs>
  <rect x="10" y="18" width="108" height="64" rx="12" stroke="rgba(148,163,184,.28)" stroke-dasharray="3 4"/>
  <path d="M36 48h12M36 58h20M36 68h8" stroke="rgba(148,163,184,.4)" stroke-width="2" stroke-linecap="round"/>
  <path d="M62 32 82 48 62 64" stroke="url(#emptyArtGrad)" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"/>
  <circle cx="96" cy="48" r="9" stroke="rgba(34,211,238,.55)" stroke-width="1.4" stroke-dasharray="2 3"/>
  <circle cx="96" cy="48" r="3" fill="#22d3ee"/>
</svg>`;

export function emptyState({ title, text, actions = "", art = EMPTY_ART, small = false }) {
  return `<div class="empty${small ? " empty--sm" : ""}">
    ${art}
    <div class="empty__title">${esc(title)}</div>
    <div class="empty__text">${text}</div>
    ${actions ? `<div class="empty__actions">${actions}</div>` : ""}
  </div>`;
}

export function errorState({ title = "Could not load data", text = "", actions = "" }) {
  return `<div class="state-panel">
    <span class="state-panel__icon tone-rose">${icon("alert-circle")}</span>
    <div class="state-panel__title">${esc(title)}</div>
    <div class="state-panel__text">${esc(text)}</div>
    ${actions ? `<div class="state-panel__actions">${actions}</div>` : ""}
  </div>`;
}

/* ------------------------------------------------------------------ */
/* Stage miniature                                                    */
/* ------------------------------------------------------------------ */

export function stageMini(wish, flow, { compact = false } = {}) {
  if (!flow) return "";
  const stages = flow.stages || [];
  const order = stages.map((stage) => stage.stage_id);
  const currentIndex = wish.current_stage ? order.indexOf(wish.current_stage) : -1;
  const delivered = wish.status === "delivered";
  const blocked = wish.status === "blocked";
  return `<span class="wish-row__pipe${compact ? " is-compact" : ""}" aria-hidden="true">${stages
    .map((stage, index) => {
      let state = "";
      if (delivered) state = "is-done";
      else if (index < currentIndex) state = "is-done";
      else if (index === currentIndex) state = blocked ? "is-blocked" : "is-current";
      return `<i class="${state}" title="${esc(stage.title)}"></i>`;
    })
    .join("")}</span>`;
}

export function stageName(flow, stageId) {
  if (!flow || !stageId) return stageId || "—";
  return (flow.stages.find((stage) => stage.stage_id === stageId) || {}).title || stageId;
}

/* ------------------------------------------------------------------ */
/* Wish list rows                                                     */
/* ------------------------------------------------------------------ */

export function wishRow(wish, ctx) {
  const flow = ctx.state.flowById.get(wish.flow_id);
  const current = wish.current_stage ? stageName(flow, wish.current_stage) : statusMeta(wish.status).label;
  const age = wishAge(wish);
  const titleIsId = wish.title === wish.wish_id;
  return `<a class="wish-row" href="#/wishes/${encodeURIComponent(wish.wish_id)}" aria-label="${esc(wish.title)} (${esc(wish.wish_id)})">
    <span class="wish-row__status tone-${statusMeta(wish.status).tone}" aria-hidden="true"></span>
    <span class="wish-row__main">
      <span class="wish-row__title${titleIsId ? " is-id" : ""}">${esc(wish.title)}${titleIsId ? "" : `<span class="id">${esc(wish.wish_id)}</span>`}</span>
      <span class="wish-row__meta">
        ${statusChip(wish.status, "chip--sm")}
        ${flowChip(wish.flow_id, "chip--sm")}
        ${wish.source === "legacy" ? sourceChip("legacy") : ""}
        <span class="sep">·</span>
        <span>${esc(current)}</span>
      </span>
    </span>
    <span class="wish-row__pipe-wrap">${stageMini(wish, flow)}</span>
    <span class="wish-row__side">
      ${revChip(wish.head_revision)}
      <span class="wish-row__age">${esc(age)}</span>
      <span class="wish-row__arrow">${icon("chevron")}</span>
    </span>
  </a>`;
}

export function wishCard(wish, ctx) {
  const flow = ctx.state.flowById.get(wish.flow_id);
  const age = wishAge(wish);
  const titleIsId = wish.title === wish.wish_id;
  return `<a class="wish-card" href="#/wishes/${encodeURIComponent(wish.wish_id)}" aria-label="${esc(wish.title)} (${esc(wish.wish_id)})">
    <span class="wish-row__status tone-${statusMeta(wish.status).tone}" aria-hidden="true"></span>
    <span class="wish-card__main">
      <span class="wish-card__title${titleIsId ? " is-id" : ""}">${esc(wish.title)}</span>
      <span class="wish-card__id">${titleIsId ? "" : `<span>${esc(wish.wish_id)}</span><span class="sep-dot"></span>`}${stageMini(wish, flow, { compact: true })}<span class="sep-dot"></span><span>${esc(stageName(flow, wish.current_stage) || "—")}</span></span>
    </span>
    <span class="wish-card__flow">${flowChip(wish.flow_id)}</span>
    <span class="wish-card__state">${statusChip(wish.status)}</span>
    <span class="wish-card__updated">${wish.head_revision !== null ? `rev ${esc(wish.head_revision)}` : "no head"}<br>${esc(age)}</span>
  </a>`;
}

/* ------------------------------------------------------------------ */
/* Assessment / evidence helpers                                      */
/* ------------------------------------------------------------------ */

export function evidenceLinks(ctx, wishId, uris, { label = null } = {}) {
  if (!uris || uris.length === 0) return "";
  return `<div class="assessment__evidence">${uris
    .map(
      (uri) =>
        `<button class="evidence-link" type="button" data-evidence="${esc(uri)}" title="Open ${esc(uri)}">${icon("file")}<span>${esc(label || uri)}</span></button>`,
    )
    .join("")}</div>`;
}

export function mountEvidenceLinks(ctx, root, wishId) {
  root.querySelectorAll("[data-evidence]").forEach((button) => {
    button.addEventListener("click", () => {
      openMaterialViewer(ctx, wishId, button.dataset.evidence);
    });
  });
}

export function artifactChips(ctx, wishId, artifacts) {
  if (!artifacts || artifacts.length === 0) return "";
  return `<div class="assessment__artifacts">${artifacts
    .map(
      (artifact) => `<button class="artifact-chip" type="button" data-evidence="${esc(artifact.uri)}" title="${esc(artifact.uri)}">
        ${icon("file")}<span>${esc(artifact.uri)}</span>
        ${artifact.sha256 ? `<span class="digest">${esc(String(artifact.sha256).slice(0, 8))}</span>` : ""}
      </button>`,
    )
    .join("")}</div>`;
}

/* ------------------------------------------------------------------ */
/* Raw JSON disclosure                                                */
/* ------------------------------------------------------------------ */

export function rawJsonBlock(data, { id = null, label = "Raw JSON" } = {}) {
  return `<div class="raw-block">
    <button class="raw-toggle" type="button" data-raw-toggle="${esc(id || "")}" aria-expanded="false">
      ${icon("code")}<span>${esc(label)}</span>${icon("chevron-down")}
    </button>
    <pre class="raw-json" ${id ? `id="${esc(id)}"` : ""} hidden>${esc(prettyJson(data))}</pre>
  </div>`;
}

export function mountRawJson(root) {
  root.querySelectorAll("[data-raw-toggle]").forEach((button) => {
    button.addEventListener("click", () => {
      const targetId = button.dataset.rawToggle;
      const panel = targetId ? root.querySelector(`#${CSS.escape(targetId)}`) : button.nextElementSibling;
      if (!panel) return;
      const open = panel.hidden;
      panel.hidden = !open;
      button.setAttribute("aria-expanded", String(open));
      button.classList.toggle("is-open", open);
    });
  });
}

/* ------------------------------------------------------------------ */
/* Misc helpers                                                       */
/* ------------------------------------------------------------------ */

export function loadingBlock(label = "Loading…") {
  return `<div class="loading-block"><span class="spinner" aria-hidden="true"></span><span>${esc(label)}</span></div>`;
}

export function fmtBinding(binding, ctx) {
  if (!binding) return `<span class="text-faint">no binding</span>`;
  if (binding.mode === "cli") {
    return `<span class="badge chip--violet badge--mono">${icon("cpu")} ${esc(binding.cli)}:${esc(binding.model)}</span>`;
  }
  if (binding.mode === "command") {
    return `<span class="badge badge--soft badge--mono">${esc(binding.command_uri || "command")}</span>`;
  }
  return `<span class="badge badge--soft badge--mono">${icon("shield")} default</span>`;
}

export function outcomeBadge(outcome) {
  return outcomeChip(outcome);
}
