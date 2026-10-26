// xdesign · WishFlow Studio — Checkpoint library.
//
// Project-level, Git-tracked authoring source: reusable syntactic rule sets
// (with their scripts/executors) and semantic criteria. Built-ins are read-only
// templates; user entries can be created, edited and versioned through
// POST /api/checkpoints.

import { api, ApiError } from "../api.js";
import {
  esc,
  icon,
  relativeTime,
  truncate,
  parseHash,
  slugId,
} from "../util.js";
import {
  openModal,
  closeModal,
  toastOk,
  toastError,
  toastWarn,
  toastInfo,
  emptyState,
  errorState,
  loadingBlock,
  executorSummary,
  checkpointKindChip,
} from "../components.js";
import { getSampleCheckpoint, getSampleCheckpointLibrary } from "../sample-data.mjs";

const KIND_FILTERS = ["all", "syntactic", "semantic"];
const SOURCE_FILTERS = ["all", "builtin", "user"];
const CHECKPOINT_ID_RE = /^[a-z0-9][a-z0-9_-]{0,63}$/;

let filters = { q: "", kind: "all", source: "all", tag: "all" };

/* ------------------------------------------------------------------ */
/* View                                                               */
/* ------------------------------------------------------------------ */

export const checkpointsView = {
  title: "Checkpoints",
  poll: false,

  crumb() {
    return "Checkpoints";
  },

  skeleton() {
    return `<div class="skeleton skeleton--line" style="width:280px;height:28px"></div>
      <div class="skeleton-grid" style="margin-top:18px">
        <div class="skeleton skeleton--stat"></div><div class="skeleton skeleton--stat"></div>
        <div class="skeleton skeleton--stat"></div><div class="skeleton skeleton--stat"></div>
      </div>
      <div class="skeleton" style="height:64px;margin-top:16px"></div>
      <div class="skeleton skeleton--panel" style="height:420px"></div>`;
  },

  async load(app) {
    if (!app.state.flows.length) await app.loadMeta();
    await app.loadCheckpoints({ force: true });
    return {
      checkpoints: app.state.checkpoints,
      error: app.state.checkpointsError,
      sample: app.state.sampleMode,
    };
  },

  async refresh(app) {
    await app.loadCheckpoints({ force: true });
    return { checkpoints: app.state.checkpoints, error: app.state.checkpointsError, sample: app.state.sampleMode };
  },

  render(app, data) {
    filters = readFilters();
    return renderPage(app, data);
  },

  mount(app, root, data) {
    let current = { ...data };
    const openTarget = parseHash(location.hash).params.get("open");

    function paint() {
      root.innerHTML = renderPage(app, current);
      bindToolbar();
    }

    function syncUrl(patch) {
      const { params } = parseHash(location.hash);
      for (const [key, value] of Object.entries(patch)) {
        if (value && value !== "all") params.set(key, value);
        else params.delete(key);
      }
      const query = params.toString();
      history.replaceState(null, "", `#/checkpoints${query ? "?" + query : ""}`);
    }

    function applyFilters() {
      syncUrl({ q: filters.q, kind: filters.kind, source: filters.source, tag: filters.tag });
      const list = root.querySelector("[data-lib-list]");
      if (list) list.innerHTML = renderList(app, current);
      const count = root.querySelector("[data-lib-count]");
      if (count) count.textContent = countLabel(current);
      const kindGroup = root.querySelector("[data-kind-filters]");
      if (kindGroup) kindGroup.innerHTML = renderKindFilters(current);
      const sourceGroup = root.querySelector("[data-source-filters]");
      if (sourceGroup) sourceGroup.innerHTML = renderSourceFilters(current);
      bindToolbar();
      const search = root.querySelector("[data-lib-search]");
      if (search) {
        search.value = filters.q;
        search.focus({ preventScroll: true });
        if (document.activeElement === search) search.setSelectionRange(search.value.length, search.value.length);
      }
      const clear = root.querySelector("[data-lib-clear]");
      if (clear) clear.hidden = !filters.q;
    }

    function reload({ repaint = true } = {}) {
      return app.loadCheckpoints({ force: true }).then(() => {
        current = {
          checkpoints: app.state.checkpoints,
          error: app.state.checkpointsError,
          sample: app.state.sampleMode,
        };
        if (repaint) paint();
      });
    }

    function openForm(entry = null, duplicate = false) {
      openCheckpointForm(app, {
        entry,
        duplicate,
        onSaved: async (savedId) => {
          await reload({ repaint: false });
          paint();
          openDetail(savedId);
        },
      });
    }

    function openDetail(checkpointId, { sync = true } = {}) {
      if (sync) syncUrl({ open: checkpointId });
      openCheckpointDetail(app, checkpointId, {
        onClose: () => syncUrl({ open: "" }),
        onDuplicate: (entry) => openForm(entry, true),
        onEdit: (entry) => openForm(entry, false),
        onSaved: async () => {
          await reload({ repaint: false });
          paint();
        },
      });
    }

    function bindToolbar() {
      const search = root.querySelector("[data-lib-search]");
      let timer = null;
      if (search && !search.dataset.bound) {
        search.dataset.bound = "1";
        search.addEventListener("input", () => {
          clearTimeout(timer);
          timer = setTimeout(() => {
            filters.q = search.value.trim();
            applyFilters();
          }, 140);
        });
        search.addEventListener("keydown", (event) => {
          if (event.key === "Escape") {
            search.value = "";
            filters.q = "";
            applyFilters();
          }
        });
      }
      const clear = root.querySelector("[data-lib-clear]");
      if (clear && !clear.dataset.bound) {
        clear.dataset.bound = "1";
        clear.addEventListener("click", () => {
          filters.q = "";
          applyFilters();
        });
      }
      root.querySelectorAll("[data-kind-filter]").forEach((chip) => {
        if (chip.dataset.bound) return;
        chip.dataset.bound = "1";
        chip.addEventListener("click", () => {
          filters.kind = chip.dataset.kindFilter;
          applyFilters();
        });
      });
      root.querySelectorAll("[data-source-filter]").forEach((chip) => {
        if (chip.dataset.bound) return;
        chip.dataset.bound = "1";
        chip.addEventListener("click", () => {
          filters.source = chip.dataset.sourceFilter;
          applyFilters();
        });
      });
      const tag = root.querySelector("[data-tag-filter]");
      if (tag && !tag.dataset.bound) {
        tag.dataset.bound = "1";
        tag.addEventListener("change", () => {
          filters.tag = tag.value;
          applyFilters();
        });
      }
      const refresh = root.querySelector("[data-lib-refresh]");
      if (refresh && !refresh.dataset.bound) {
        refresh.dataset.bound = "1";
        refresh.addEventListener("click", async () => {
          refresh.classList.add("is-busy");
          try {
            await reload();
            toastInfo("Library refreshed", "Read from the project checkpoint library.");
          } finally {
            refresh.classList.remove("is-busy");
          }
        });
      }
    }

    // Delegated clicks survive list/by-toolbar re-renders.
    const onClick = (event) => {
      const card = event.target.closest("[data-lib-open]");
      if (card) {
        openDetail(card.dataset.libOpen);
        return;
      }
      const newButton = event.target.closest("[data-lib-new]");
      if (newButton) {
        if (app.state.sampleMode) {
          toastWarn("Sample mode: writes disabled", "The labelled demo library is read-only; start `xdesign web` to create checkpoints.");
          return;
        }
        openForm();
        return;
      }
      const clearFilters = event.target.closest("[data-lib-clear-filters]");
      if (clearFilters) {
        filters = { q: "", kind: "all", source: "all", tag: "all" };
        syncUrl({ q: "", kind: "", source: "", tag: "" });
        paint();
        return;
      }
      const retry = event.target.closest("[data-lib-retry]");
      if (retry) app.handleRoute();
    };
    root.addEventListener("click", onClick);

    bindToolbar();
    if (openTarget) openDetail(decodeURIComponent(openTarget), { sync: false });
    return () => root.removeEventListener("click", onClick);
  },
};

/* Filters                                                            */
/* ------------------------------------------------------------------ */

function readFilters() {
  const { params } = parseHash(location.hash);
  return {
    q: params.get("q") || "",
    kind: KIND_FILTERS.includes(params.get("kind")) ? params.get("kind") : "all",
    source: SOURCE_FILTERS.includes(params.get("source")) ? params.get("source") : "all",
    tag: params.get("tag") || "all",
  };
}

function matches(entry) {
  if (filters.kind !== "all" && entry.kind !== filters.kind) return false;
  if (filters.source === "builtin" && !entry.builtin) return false;
  if (filters.source === "user" && entry.builtin) return false;
  if (filters.tag !== "all" && !(entry.tags || []).includes(filters.tag)) return false;
  if (filters.q) {
    const haystack = `${entry.checkpoint_id} ${entry.title}`.toLowerCase();
    if (!haystack.includes(filters.q.toLowerCase())) return false;
  }
  return true;
}

function allTags(checkpoints) {
  const tags = new Set();
  for (const entry of checkpoints) for (const tag of entry.tags || []) tags.add(tag);
  return [...tags].sort();
}

function countLabel(data) {
  const total = data.checkpoints.length;
  const shown = data.checkpoints.filter(matches).length;
  return `${shown} of ${total} checkpoint${total === 1 ? "" : "s"}`;
}

/* ------------------------------------------------------------------ */
/* Page render                                                        */
/* ------------------------------------------------------------------ */

function renderPage(app, data) {
  const checkpoints = data.checkpoints || [];
  const sample = data.sample || app.state.sampleMode;
  const liveError = data.error && !sample ? data.error : null;
  return `<div class="page-head animate-rise">
    <div class="page-head__titles">
      <span class="eyebrow">Authoring source · project library</span>
      <h1 class="h1">Checkpoint library</h1>
      <p class="lede">Reusable syntactic rule sets and semantic criteria that flows bind as check executors. Entries are versioned project source; every run keeps the snapshot frozen at initialization.</p>
    </div>
    <div class="page-head__actions">
      <button class="icon-btn" type="button" data-lib-refresh title="Refresh library" aria-label="Refresh library">${icon("refresh")}</button>
      ${
        sample
          ? `<button class="btn btn--primary is-disabled" type="button" data-lib-new title="Writes are disabled in sample mode">${icon("plus")}New checkpoint</button>`
          : `<button class="btn btn--primary" type="button" data-lib-new>${icon("plus")}New checkpoint</button>`
      }
    </div>
  </div>
  ${sample ? renderSampleNotice() : ""}
  ${liveError ? `<div class="panel" style="margin-bottom:16px">${errorState({
    title: "Checkpoint library unavailable",
    text: `${liveError.message || "The server did not return the checkpoint library."} The wish pages still work; retry once the server is available.`,
    actions: `<button class="btn" type="button" data-lib-retry>${icon("refresh")}Retry</button>`,
  })}</div>` : ""}
  ${renderStats(checkpoints)}
  ${renderToolbar(checkpoints)}
  <div class="animate-rise is-delay-2" data-lib-list>${renderList(app, data)}</div>`;
}

function renderSampleNotice() {
  return `<div class="banner banner--sample" style="margin:0 0 16px;width:100%">
    <span class="banner__icon">${icon("info")}</span>
    <div class="banner__text"><strong>Sample mode</strong><span>The library is a labelled demo dataset. Entries are read-only here; start the real server to create or edit checkpoints.</span></div>
  </div>`;
}

function renderStats(checkpoints) {
  const syntactic = checkpoints.filter((entry) => entry.kind === "syntactic").length;
  const semantic = checkpoints.filter((entry) => entry.kind === "semantic").length;
  const builtin = checkpoints.filter((entry) => entry.builtin).length;
  return `<div class="stat-grid animate-rise is-delay-1">
    ${libStat("Entries", checkpoints.length, "library", "ink", "project authoring source")}
    ${libStat("Syntactic", syntactic, "shield", "cyan", "rule sets executed by a script")}
    ${libStat("Semantic", semantic, "sparkle", "violet", "criteria judged by an agent or human")}
    ${libStat("Built-in", builtin, "lock", "green", "read-only templates from the binary")}
  </div>`;
}

function libStat(label, value, iconName, tone, meta) {
  return `<div class="stat tone-${tone}">
    <span class="stat__head"><span>${esc(label)}</span><span class="stat__icon">${icon(iconName)}</span></span>
    <span class="stat__value">${value}</span>
    <span class="stat__meta">${esc(meta)}</span>
  </div>`;
}

function renderToolbar(checkpoints) {
  const kinds = { all: checkpoints.length, syntactic: 0, semantic: 0 };
  const sources = { all: checkpoints.length, builtin: 0, user: 0 };
  for (const entry of checkpoints) {
    kinds[entry.kind] = (kinds[entry.kind] || 0) + 1;
    if (entry.builtin) sources.builtin += 1;
    else sources.user += 1;
  }
  return `<div class="toolbar animate-rise is-delay-1" style="margin-top:16px">
    <label class="search">
      <span class="icon">${icon("search")}</span>
      <input type="search" data-lib-search placeholder="Search checkpoints…" autocomplete="off" spellcheck="false" aria-label="Search checkpoints" value="${esc(filters.q)}" />
      <button class="search__clear" type="button" data-lib-clear aria-label="Clear search" ${filters.q ? "" : "hidden"}>${icon("close")}</button>
    </label>
    <div class="filter-group" data-kind-filters>${renderKindFilters({ checkpoints })}</div>
    <div class="filter-group" data-source-filters>${renderSourceFilters({ checkpoints })}</div>
    <select class="toolbar__select" data-tag-filter aria-label="Filter by tag">
      <option value="all">All tags</option>
      ${allTags(checkpoints).map((tag) => `<option value="${esc(tag)}" ${filters.tag === tag ? "selected" : ""}>${esc(tag)}</option>`).join("")}
    </select>
    <span class="toolbar__spacer"></span>
    <span class="toolbar__count" data-lib-count>${esc(countLabel({ checkpoints }))}</span>
  </div>`;
}

function renderKindFilters(data) {
  const checkpoints = data.checkpoints || [];
  const counts = { all: checkpoints.length, syntactic: 0, semantic: 0 };
  for (const entry of checkpoints) counts[entry.kind] = (counts[entry.kind] || 0) + 1;
  const labels = { all: "All", syntactic: "Syntactic", semantic: "Semantic" };
  const tones = { all: "ink", syntactic: "cyan", semantic: "violet" };
  return `<span class="filter-label">Kind</span>${KIND_FILTERS.map((kind) => {
    const active = filters.kind === kind;
    return `<button class="filter-chip tone-${tones[kind]}${active ? " is-active" : ""}" type="button" data-kind-filter="${kind}" aria-pressed="${active}">
      <span>${esc(labels[kind])}</span><span class="count">${counts[kind] || 0}</span>
    </button>`;
  }).join("")}`;
}

function renderSourceFilters(data) {
  const checkpoints = data.checkpoints || [];
  const counts = { all: checkpoints.length, builtin: 0, user: 0 };
  for (const entry of checkpoints) counts[entry.builtin ? "builtin" : "user"] += 1;
  const labels = { all: "All", builtin: "Built-in", user: "User" };
  return `<span class="filter-label">Source</span>${SOURCE_FILTERS.map((source) => {
    const active = filters.source === source;
    return `<button class="filter-chip tone-ink${active ? " is-active" : ""}" type="button" data-source-filter="${source}" aria-pressed="${active}">
      <span>${esc(labels[source])}</span><span class="count">${counts[source] || 0}</span>
    </button>`;
  }).join("")}`;
}

/* ------------------------------------------------------------------ */
/* List                                                               */
/* ------------------------------------------------------------------ */

function renderList(app, data) {
  const checkpoints = data.checkpoints || [];
  if (checkpoints.length === 0) {
    if (data.sample) {
      return `<div class="panel">${emptyState({ title: "The sample library is empty", text: "Start the real server to inspect the built-in checkpoint templates." })}</div>`;
    }
    return `<div class="panel">${emptyState({
      title: "No checkpoints yet",
      text: "The built-in templates are always listed when the project library is available. Create a user entry to add your own rules or semantic criteria.",
      actions: `<button class="btn btn--primary btn--sm" type="button" data-lib-new>${icon("plus")}New checkpoint</button>`,
    })}</div>`;
  }
  const shown = checkpoints.filter(matches);
  if (shown.length === 0) {
    return `<div class="panel">${emptyState({
      title: "No checkpoints match these filters",
      text: "Adjust the search, kind, source or tag filter to see more of the library.",
      actions: `<button class="btn btn--sm" type="button" data-lib-clear-filters>${icon("close")}Clear filters</button>`,
    })}</div>`;
  }
  return `<div class="lib-grid">${shown.map((entry) => renderCard(entry)).join("")}</div>`;
}

function renderCard(entry) {
  const semantic = entry.kind === "semantic";
  const parts = semantic
    ? `${entry.criteria_count || 0} criteria`
    : `${entry.rule_count || 0} rule${entry.rule_count === 1 ? "" : "s"}`;
  const executor = semantic ? "judged · no executor" : executorSummary(entry.executor);
  return `<button class="lib-card ${semantic ? "is-semantic" : "is-syntactic"}" type="button" data-lib-open="${esc(entry.checkpoint_id)}" aria-label="Open checkpoint ${esc(entry.title)}">
    <span class="lib-card__top">
      ${checkpointKindChip(entry)}
      <span class="chip chip--sm ${entry.builtin ? "chip--green" : "chip--blue"}">${entry.builtin ? icon("lock") + "built-in" : icon("user") + "user"}</span>
      <span class="lib-card__version mono">v${esc(entry.version)}</span>
    </span>
    <span class="lib-card__title">${esc(entry.title)}</span>
    <span class="lib-card__id mono">${esc(entry.checkpoint_id)}</span>
    <span class="lib-card__meta">
      <span class="lib-card__executor" title="${esc(executor)}">${icon(semantic ? "sparkle" : entry.executor && entry.executor.mode === "cli" ? "cpu" : "terminal")}<span>${esc(truncate(executor, 42))}</span></span>
      <span class="sep-dot"></span>
      <span>${esc(parts)}</span>
      <span class="sep-dot"></span>
      <span>updated ${esc(relativeTime(entry.updated_at))}</span>
    </span>
    ${(entry.tags || []).length ? `<span class="lib-card__tags">${entry.tags.map((tag) => `<span class="chip chip--sm">#${esc(tag)}</span>`).join("")}</span>` : ""}
  </button>`;
}

/* ------------------------------------------------------------------ */
/* Detail drawer                                                      */
/* ------------------------------------------------------------------ */

async function openCheckpointDetail(app, checkpointId, { onClose = null, onChanged = null, onDuplicate = null, onEdit = null } = {}) {
  const sample = app.state.sampleMode;
  const modal = openModal({
    title: "Checkpoint",
    subtitle: checkpointId,
    iconName: "library",
    size: "lg",
    bodyHtml: loadingBlock("Loading checkpoint definition…"),
    footerHtml: '<span class="modal__foot-meta">project library</span>',
  });
  if (!modal) return;
  const body = modal.panel.querySelector("[data-modal-body]");
  let entry = null;
  let error = null;
  try {
    if (sample) {
      const sampleReply = getSampleCheckpoint(checkpointId);
      if (!sampleReply) throw new Error(`No sample checkpoint named ${checkpointId}.`);
      entry = sampleReply.checkpoint;
    } else {
      const reply = await api.checkpoint(checkpointId);
      entry = reply.checkpoint;
    }
  } catch (caught) {
    error = caught;
  }
  if (!body || !document.body.contains(body)) return;
  if (error) {
    body.innerHTML = `<div class="material-error">${icon("alert-circle")}<div><strong>Checkpoint unavailable</strong>
      <div class="text-faint" style="font-size:11.5px;margin-top:2px">${esc(error.message)}</div></div></div>
      <pre class="material-pre" style="margin-top:14px">${esc(checkpointId)}</pre>`;
    return;
  }
  body.innerHTML = renderDetailBody(entry, { sample });
  const footer = modal.panel.querySelector(".modal__foot");
  if (footer) {
    footer.innerHTML = `
      <span class="modal__foot-meta">${entry.builtin ? "built-in template · read-only" : `user entry · v${esc(entry.version)}`}</span>
      <div class="row">
        ${entry.builtin
          ? `<button class="btn btn--sm" type="button" data-lib-duplicate>${icon("copy")}Duplicate as new entry</button>`
          : `<button class="btn btn--sm" type="button" data-lib-edit>${icon("sliders")}Edit entry</button>
             <button class="btn btn--ghost btn--sm" type="button" data-lib-duplicate>${icon("copy")}Duplicate</button>`}
        <button class="btn btn--primary btn--sm" type="button" data-modal-close>Close</button>
      </div>`;
    footer.querySelectorAll("[data-modal-close]").forEach((button) => button.addEventListener("click", () => closeModal()));
    const duplicate = footer.querySelector("[data-lib-duplicate]");
    if (duplicate) {
      duplicate.addEventListener("click", () => {
        closeModal();
        if (onClose) onClose();
        if (onDuplicate) onDuplicate(entry);
      });
    }
    const edit = footer.querySelector("[data-lib-edit]");
    if (edit) {
      edit.addEventListener("click", () => {
        closeModal();
        if (onClose) onClose();
        if (onEdit) onEdit(entry);
      });
    }
  }
  modal.panel.querySelectorAll("[data-modal-close], [data-modal-backdrop]").forEach((element) => {
    element.addEventListener("click", () => {
      if (onClose) onClose();
    });
  });
  const closeButton = modal.panel.querySelector(".modal__close");
  if (closeButton) {
    closeButton.addEventListener("click", () => {
      if (onClose) onClose();
    });
  }
  if (onChanged) modal.panel.__onChanged = onChanged;
}

function renderDetailBody(entry, { sample = false } = {}) {
  const semantic = entry.kind === "semantic";
  return `<div class="lib-detail">
    <div class="lib-detail__chips">
      ${checkpointKindChip(entry, "chip")}
      <span class="chip ${entry.builtin ? "chip--green" : "chip--blue"}">${entry.builtin ? icon("lock") + "built-in template" : icon("user") + "user entry"}</span>
      <span class="mono chip chip--mono">${esc(entry.checkpoint_id)}</span>
      <span class="mono chip chip--mono">v${esc(entry.version)}</span>
      <span class="chip chip--sm">${icon("clock")}updated ${esc(relativeTime(entry.updated_at))}</span>
      <span class="chip chip--mono chip--sm">${esc(entry.source || "manual")}</span>
    </div>
    <p class="lib-detail__description">${esc(entry.description || "No description recorded.")}</p>
    ${(entry.tags || []).length ? `<div class="lib-detail__tags">${entry.tags.map((tag) => `<span class="chip chip--sm">#${esc(tag)}</span>`).join("")}</div>` : ""}
    ${
      semantic
        ? `<section class="lib-detail__section">
            <div class="lib-detail__section-title">${icon("sparkle")}Semantic criteria</div>
            <ul class="lib-detail__criteria">${(entry.criteria || []).map((criterion) => `<li>${esc(criterion)}</li>`).join("") || '<li class="text-faint">No criteria recorded.</li>'}</ul>
            <div class="lib-detail__section-title" style="margin-top:14px">${icon("shield")}Judges</div>
            <div class="row">
              <span class="chip ${entry.judges && entry.judges.agent ? "chip--violet" : "chip--slate"}">${entry.judges && entry.judges.agent ? icon("check") : icon("close")}agent may judge</span>
              <span class="chip ${entry.judges && entry.judges.human ? "chip--green" : "chip--slate"}">${entry.judges && entry.judges.human ? icon("check") : icon("close")}human may judge</span>
            </div>
            <div class="lib-detail__note">A semantic checkpoint has no script or executor: the caller records a permitted judgment with an authority, assessor, outcome and rationale.</div>
          </section>`
        : `<section class="lib-detail__section">
            <div class="lib-detail__section-title">${icon("shield")}Rules</div>
            <div class="lib-rules">
              <div class="lib-rules__head"><span>Rule id</span><span>Condition</span><span>Reads</span></div>
              ${(entry.rules || []).map((rule) => `<div class="lib-rules__row">
                <span class="lib-rules__id mono">${esc(rule.rule_id)}</span>
                <span class="lib-rules__condition">${esc(rule.condition)}</span>
                <span class="lib-rules__reads">${(rule.reads || []).map((uri) => `<span class="mono">${esc(uri)}</span>`).join("") || '<span class="text-faint">—</span>'}</span>
              </div>`).join("") || '<div class="lib-rules__row"><span class="text-faint">No rules recorded.</span></div>'}
            </div>
            <div class="lib-detail__section-title" style="margin-top:14px">Inputs</div>
            <div class="row">${(entry.inputs || []).map((uri) => `<span class="chip chip--mono chip--sm">${esc(uri)}</span>`).join("") || '<span class="text-faint" style="font-size:11.5px">No inputs recorded.</span>'}</div>
            <div class="lib-detail__section-title" style="margin-top:14px">Executor</div>
            ${renderExecutorBlock(entry.executor)}
          </section>`
    }
    <section class="lib-detail__section">
      <div class="lib-detail__section-title">${icon("clock")}Version history</div>
      ${
        (entry.history || []).length
          ? `<div class="lib-history">${entry.history
              .map((snapshot) => `<div class="lib-history__row">
                <span class="mono">v${esc(snapshot.version)}</span>
                <span>${esc(truncate(snapshot.description || "", 96))}</span>
                <span class="text-faint mono">${esc(relativeTime(snapshot.updated_at))}</span>
              </div>`)
              .join("")}</div>`
          : '<div class="text-faint" style="font-size:11.5px">Initial version — no earlier snapshots.</div>'
      }
    </section>
    ${entry.builtin ? `<div class="lib-detail__note">${icon("lock")}<span>This entry is a built-in template from the binary. It cannot be edited or deleted; duplicate it as a new entry to customise the rules, inputs or executor.</span></div>` : ""}
    ${sample ? `<div class="lib-detail__note">${icon("info")}<span>Sample dataset entry, shown read-only.</span></div>` : ""}
  </div>`;
}

function renderExecutorBlock(executor) {
  if (!executor) {
    return `<div class="lib-detail__note">No executor: this checkpoint is judged, not script-executed.</div>`;
  }
  if (executor.mode === "cli") {
    return `<div class="lib-detail__executor">
      <span class="agent-card__icon tone-violet" style="width:30px;height:30px">${icon("cpu")}</span>
      <div><div class="lib-detail__executor-title">Agent CLI</div>
      <div class="mono lib-detail__executor-meta">${esc(executor.cli)}:${esc(executor.model)}</div>
      <div class="lib-detail__note" style="padding:6px 0 0">The agent CLI judges the declared rules; the assessment is recorded with agent authority.</div></div>
    </div>`;
  }
  return `<div class="lib-detail__executor">
    <span class="agent-card__icon tone-green" style="width:30px;height:30px">${icon("terminal")}</span>
    <div><div class="lib-detail__executor-title">Project script</div>
    <div class="mono lib-detail__executor-meta">${esc(executor.command_uri)}</div>
    <div class="lib-detail__note" style="padding:6px 0 0">The script runs the declared rules inside xdesign; the assessment is recorded with script authority.</div></div>
  </div>`;
}

/* ------------------------------------------------------------------ */
/* Create / edit form                                                 */
/* ------------------------------------------------------------------ */

function openCheckpointForm(app, { entry = null, duplicate = false, onSaved = null } = {}) {
  const sample = app.state.sampleMode;
  const editing = Boolean(entry) && !duplicate;
  const baseKind = entry ? entry.kind : "";
  const derivedId = entry ? (duplicate ? `${entry.checkpoint_id}-copy` : entry.checkpoint_id) : "";
  const agents = app.state.agents || [];
  const defaultAgent = agents.find((agent) => agent.available) || agents[0] || null;
  const scriptExecutor = entry && entry.executor && entry.executor.mode === "script" ? entry.executor : null;
  const cliExecutor = entry && entry.executor && entry.executor.mode === "cli" ? entry.executor : null;
  const form = {
    mode: editing ? "edit" : "create",
    kind: baseKind,
    checkpoint_id: derivedId,
    title: entry ? (duplicate ? `${entry.title} (copy)` : entry.title) : "",
    description: entry ? entry.description || "" : "",
    tags: entry ? (entry.tags || []).join(", ") : "",
    rules: entry && entry.rules && entry.rules.length
      ? entry.rules.map((rule) => ({ rule_id: rule.rule_id, condition: rule.condition, reads: (rule.reads || []).join(", ") }))
      : [{ rule_id: "", condition: "", reads: "" }],
    inputs: entry && entry.inputs ? entry.inputs.join("\n") : "",
    execMode: scriptExecutor ? "reuse" : cliExecutor ? "cli" : "reuse",
    command_uri: scriptExecutor ? scriptExecutor.command_uri : "",
    createScript: false,
    cli: cliExecutor ? cliExecutor.cli : (defaultAgent ? defaultAgent.cli : "codex"),
    model: cliExecutor ? cliExecutor.model : (defaultAgent ? defaultAgent.default_model || "" : ""),
    criteria: entry && entry.criteria && entry.criteria.length ? [...entry.criteria] : [""],
    judges: {
      agent: entry && entry.judges ? Boolean(entry.judges.agent) : true,
      human: entry && entry.judges ? Boolean(entry.judges.human) : true,
    },
    editedBuiltin: Boolean(entry && entry.builtin && duplicate),
    editingId: editing ? entry.checkpoint_id : null,
    errors: {},
    serverError: null,
    submitting: false,
  };

  const modal = openModal({
    title: editing ? "Edit checkpoint" : duplicate ? "Duplicate checkpoint" : "New checkpoint",
    subtitle: editing ? `editing ${entry.checkpoint_id}` : "project library entry",
    iconName: "library",
    size: "lg",
    bodyHtml: '<div data-form-host></div>',
    footerHtml: '<span class="modal__foot-meta">saved to checkpoints/library.json</span>',
  });
  if (!modal) return;
  const host = modal.panel.querySelector("[data-form-host]");
  const footer = modal.panel.querySelector(".modal__foot");

  const paint = () => {
    host.innerHTML = renderFormBody(app, form, { editing, sample });
    host.querySelectorAll(".is-invalid").forEach((element) => element.classList.remove("is-invalid"));
    const firstError = host.querySelector("[data-form-error]");
    if (firstError) firstError.hidden = !form.serverError && Object.keys(form.errors).length === 0;
  };

  if (footer) {
    footer.innerHTML = `
      <span class="modal__foot-meta">${sample ? "sample mode · writes disabled" : editing ? "saves as a new version" : "version 1"}</span>
      <div class="row">
        <button class="btn btn--ghost btn--sm" type="button" data-form-cancel>Cancel</button>
        <button class="btn btn--primary btn--sm" type="button" data-form-save ${sample ? "disabled" : ""} title="${sample ? "Writes are disabled in sample mode" : ""}">${icon("check")}${editing ? "Save new version" : "Create checkpoint"}</button>
      </div>`;
    footer.querySelector("[data-form-cancel]").addEventListener("click", () => closeModal());
    footer.querySelector("[data-form-save]").addEventListener("click", () => submitForm());
  }

  const update = (patch) => {
    Object.assign(form, patch);
  };

  const onInput = (event) => {
    const target = event.target;
    const field = target.dataset.formField;
    if (field) {
      update({ [field]: target.value });
      if (form.errors[field]) {
        form.errors[field] = null;
        const errorEl = host.querySelector(`[data-form-error-field="${field}"]`);
        if (errorEl) errorEl.textContent = "";
        target.classList.remove("is-invalid");
      }
      return;
    }
    if (target.dataset.ruleIndex !== undefined) {
      const index = Number(target.dataset.ruleIndex);
      const key = target.dataset.ruleField;
      if (form.rules[index]) form.rules[index][key] = target.value;
      if (form.errors.rules) {
        form.errors.rules = null;
        const errorEl = host.querySelector('[data-form-error-field="rules"]');
        if (errorEl) errorEl.textContent = "";
      }
      return;
    }
    if (target.dataset.criterionIndex !== undefined) {
      const index = Number(target.dataset.criterionIndex);
      form.criteria[index] = target.value;
      if (form.errors.criteria) {
        form.errors.criteria = null;
        const errorEl = host.querySelector('[data-form-error-field="criteria"]');
        if (errorEl) errorEl.textContent = "";
      }
    }
  };

  const onClick = (event) => {
    const target = event.target.closest("[data-form-kind], [data-form-exec], [data-rule-add], [data-rule-remove], [data-criterion-add], [data-criterion-remove], [data-suggest-checkpoint-id], [data-judge]");
    if (!target) return;
    if (target.dataset.formKind) {
      const kind = target.dataset.formKind;
      if (form.kind === kind) return;
      form.kind = kind;
      form.errors = {};
      form.serverError = null;
      // Resize defaults when switching kind.
      if (kind === "semantic" && form.criteria.length === 0) form.criteria = [""];
      if (kind === "syntactic" && form.rules.length === 0) form.rules = [{ rule_id: "", condition: "", reads: "" }];
      paint();
      return;
    }
    if (target.dataset.formExec) {
      form.execMode = target.dataset.formExec;
      if (form.execMode === "scaffold") form.command_uri = scaffoldUri(form.checkpoint_id);
      paint();
      return;
    }
    if (target.dataset.ruleAdd !== undefined) {
      form.rules.push({ rule_id: "", condition: "", reads: "" });
      paint();
      return;
    }
    if (target.dataset.ruleRemove !== undefined) {
      form.rules.splice(Number(target.dataset.ruleRemove), 1);
      if (form.rules.length === 0) form.rules.push({ rule_id: "", condition: "", reads: "" });
      paint();
      return;
    }
    if (target.dataset.criterionAdd !== undefined) {
      form.criteria.push("");
      paint();
      return;
    }
    if (target.dataset.criterionRemove !== undefined) {
      form.criteria.splice(Number(target.dataset.criterionRemove), 1);
      if (form.criteria.length === 0) form.criteria.push("");
      paint();
      return;
    }
    if (target.dataset.suggestCheckpointId !== undefined) {
      form.checkpoint_id = slugId(form.title || "checkpoint") || "checkpoint";
      paint();
      const idInput = host.querySelector('[data-form-field="checkpoint_id"]');
      if (idInput) idInput.focus();
      return;
    }
    if (target.dataset.judge) {
      const judge = target.dataset.judge;
      form.judges[judge] = !form.judges[judge];
      if (form.errors.judges) form.errors.judges = null;
      paint();
    }
  };

  host.addEventListener("input", onInput);
  host.addEventListener("change", onInput);
  host.addEventListener("click", onClick);

  const submitForm = async () => {
    if (form.submitting) return;
    if (!app.requireWritable()) return;
    const errors = validateCheckpointForm(form);
    form.errors = errors;
    form.serverError = null;
    if (Object.keys(errors).length > 0) {
      paint();
      const firstKey = Object.keys(errors)[0];
      const errorEl = host.querySelector(`[data-form-error-field="${firstKey}"]`);
      if (errorEl) errorEl.scrollIntoView({ block: "center", behavior: "smooth" });
      return;
    }
    form.submitting = true;
    const saveButton = footer.querySelector("[data-form-save]");
    if (saveButton) {
      saveButton.classList.add("is-busy");
      saveButton.disabled = true;
    }
    try {
      const body = buildRequestBody(form);
      const reply = await api.createCheckpoint(body);
      const checkpointId = (reply && reply.checkpoint && reply.checkpoint.checkpoint_id) || body.checkpoint_id;
      toastOk(reply && reply.created ? "Checkpoint created" : "Checkpoint saved", `${checkpointId} · v${(reply && reply.version) || body.version || 1}${reply && reply.script_uri ? ` · ${reply.script_uri}` : ""}`);
      closeModal();
      if (onSaved) await onSaved(checkpointId, reply);
    } catch (error) {
      form.submitting = false;
      if (saveButton) {
        saveButton.classList.remove("is-busy");
        saveButton.disabled = sample;
      }
      const message = error.message || "The checkpoint could not be saved.";
      form.serverError = error.code === "builtin_checkpoint"
        ? "Built-in checkpoint ids are reserved and read-only. Duplicate the built-in as a new entry with a new id."
        : message;
      const mapped = mapServerErrorToField(message);
      if (mapped) form.errors = { ...form.errors, [mapped]: message };
      paint();
      const errorEl = host.querySelector("[data-form-error]");
      if (errorEl) {
        errorEl.hidden = false;
        errorEl.scrollIntoView({ block: "nearest", behavior: "smooth" });
      }
      toastError("Checkpoint not saved", form.serverError);
    }
  };

  paint();
}

function renderFormBody(app, form, { editing = false, sample = false } = {}) {
  const semantic = form.kind === "semantic";
  const syntactic = form.kind === "syntactic";
  const agents = app.state.agents || [];
  const agent = agents.find((entry) => entry.cli === form.cli) || agents[0] || null;
  const models = (agent && agent.models) || [];
  const errorFor = (key) => (form.errors[key] ? `<div class="field__error" data-form-error-field="${key}">${icon("alert-circle")}${esc(form.errors[key])}</div>` : `<div class="field__error" data-form-error-field="${key}"></div>`);
  const scaffold = scaffoldUri(form.checkpoint_id);
  return `<form class="lib-form" data-checkpoint-form autocomplete="off" novalidate>
    <div class="form-error" data-form-error ${form.serverError || Object.keys(form.errors).length ? "" : "hidden"}>${icon("alert-circle")}<span>${esc(form.serverError || Object.values(form.errors).find(Boolean) || "")}</span></div>
    <div class="lib-form__section">
      <div class="lib-form__section-title"><span class="lib-form__step">01</span>Checkpoint kind</div>
      <div class="kind-picker">
        <button class="kind-option ${form.kind === "syntactic" ? "is-selected" : ""}" type="button" data-form-kind="syntactic" aria-pressed="${form.kind === "syntactic"}">
          <span class="kind-option__icon tone-cyan">${icon("shield")}</span>
          <span class="kind-option__body">
            <span class="kind-option__name">Syntactic rule set</span>
            <span class="kind-option__sub">Deterministic rules run by a project script or a separate agent CLI. Rule ids, conditions and reads.</span>
          </span>
        </button>
        <button class="kind-option ${semantic ? "is-selected" : ""}" type="button" data-form-kind="semantic" aria-pressed="${semantic}">
          <span class="kind-option__icon tone-violet">${icon("sparkle")}</span>
          <span class="kind-option__body">
            <span class="kind-option__name">Semantic criteria</span>
            <span class="kind-option__sub">Criteria judged by an agent or human authority. No script and no executor.</span>
          </span>
        </button>
      </div>
      ${errorFor("kind")}
    </div>
    <div class="lib-form__section">
      <div class="lib-form__section-title"><span class="lib-form__step">02</span>Identity</div>
      <div class="form-grid">
        <div class="field">
          <div class="field__row">
            <label class="label" for="cp-id">Checkpoint id <span class="req">*</span></label>
            <span class="char-count">slug</span>
          </div>
          <div class="field__inline">
            <input class="input input--mono ${form.errors.checkpoint_id ? "is-invalid" : ""}" id="cp-id" data-form-field="checkpoint_id" value="${esc(form.checkpoint_id)}" placeholder="energy-budget-positive" spellcheck="false" ${editing ? "readonly" : ""} />
            ${editing ? "" : `<button class="btn btn--sm" type="button" data-suggest-checkpoint-id title="Derive a slug from the title">${icon("sparkle")}Slug</button>`}
          </div>
          <div class="field__hint">Slug pattern: <span class="mono">[a-z0-9][a-z0-9_-]{0,63}</span>; unique among user entries.</div>
          ${errorFor("checkpoint_id")}
        </div>
        <div class="field">
          <div class="field__row">
            <label class="label" for="cp-title">Title <span class="req">*</span></label>
            <span class="char-count">1–120</span>
          </div>
          <input class="input ${form.errors.title ? "is-invalid" : ""}" id="cp-title" data-form-field="title" value="${esc(form.title)}" placeholder="Energy budget stays non-negative" maxlength="120" />
          ${errorFor("title")}
        </div>
        <div class="field field--wide">
          <div class="field__row">
            <label class="label" for="cp-description">Description <span class="req">*</span></label>
            <span class="char-count">${esc((form.description || "").length)}/1000</span>
          </div>
          <textarea class="textarea ${form.errors.description ? "is-invalid" : ""}" id="cp-description" data-form-field="description" rows="3" maxlength="1000" placeholder="What this checkpoint establishes, written so a reviewer understands it without asking the author.">${esc(form.description)}</textarea>
          ${errorFor("description")}
        </div>
        <div class="field field--wide">
          <div class="field__row">
            <label class="label" for="cp-tags">Tags <span class="optional">optional · comma separated</span></label>
            <span class="char-count">≤ 8</span>
          </div>
          <input class="input input--mono ${form.errors.tags ? "is-invalid" : ""}" id="cp-tags" data-form-field="tags" value="${esc(form.tags)}" placeholder="energy, design" spellcheck="false" />
          <div class="field__hint">Lowercase slugs used to filter the library.</div>
          ${errorFor("tags")}
        </div>
      </div>
    </div>
    ${
      semantic
        ? `<div class="lib-form__section">
            <div class="lib-form__section-title"><span class="lib-form__step">03</span>Semantic criteria</div>
            <div class="criterion-editor">
              ${form.criteria
                .map(
                  (criterion, index) => `<div class="criterion-row">
                    <span class="criterion-row__num mono">${index + 1}</span>
                    <textarea class="textarea" data-criterion-index="${index}" rows="2" placeholder="A criterion a permitted judge can re-examine…">${esc(criterion)}</textarea>
                    <button class="icon-btn" type="button" data-criterion-remove="${index}" aria-label="Remove criterion">${icon("close")}</button>
                  </div>`,
                )
                .join("")}
              <button class="btn btn--ghost btn--sm" type="button" data-criterion-add>${icon("plus")}Add criterion</button>
              ${errorFor("criteria")}
            </div>
            <div class="lib-form__section-title" style="margin-top:16px"><span class="lib-form__step">04</span>Judge permissions <span class="req">*</span></div>
            <div class="row" style="gap:10px;flex-wrap:wrap">
              <button class="judge-toggle ${form.judges.agent ? "is-on" : ""}" type="button" data-judge="agent" aria-pressed="${form.judges.agent}">${icon(form.judges.agent ? "check" : "close")}agent may judge</button>
              <button class="judge-toggle ${form.judges.human ? "is-on" : ""}" type="button" data-judge="human" aria-pressed="${form.judges.human}">${icon(form.judges.human ? "check" : "close")}human may judge</button>
            </div>
            <div class="field__hint">At least one permitted authority is required. The judgment records authority, assessor, outcome and rationale.</div>
            ${errorFor("judges")}
          </div>`
        : `<div class="lib-form__section">
            <div class="lib-form__section-title"><span class="lib-form__step">03</span>Rules</div>
            <div class="rules-editor">
              <div class="rules-editor__head"><span>Rule id</span><span>Condition</span><span>Reads (comma separated)</span><span></span></div>
              ${form.rules
                .map(
                  (rule, index) => `<div class="rules-editor__row">
                    <input class="input input--mono" data-rule-index="${index}" data-rule-field="rule_id" value="${esc(rule.rule_id)}" placeholder="budget-declared" spellcheck="false" aria-label="Rule id ${index + 1}" />
                    <textarea class="textarea" data-rule-index="${index}" data-rule-field="condition" rows="2" placeholder="Every actuator declares a numeric energyBudget." aria-label="Rule condition ${index + 1}">${esc(rule.condition)}</textarea>
                    <input class="input input--mono" data-rule-index="${index}" data-rule-field="reads" value="${esc(rule.reads)}" placeholder="mbse/model/design.sysml" spellcheck="false" aria-label="Rule reads ${index + 1}" />
                    <button class="icon-btn" type="button" data-rule-remove="${index}" aria-label="Remove rule ${index + 1}">${icon("close")}</button>
                  </div>`,
                )
                .join("")}
              <button class="btn btn--ghost btn--sm" type="button" data-rule-add>${icon("plus")}Add rule</button>
              ${errorFor("rules")}
            </div>
            <div class="field" style="margin-top:14px">
              <label class="label" for="cp-inputs">Inputs <span class="req">*</span> <span class="optional">one project-relative path per line</span></label>
              <textarea class="textarea textarea--mono ${form.errors.inputs ? "is-invalid" : ""}" id="cp-inputs" data-form-field="inputs" rows="3" placeholder="mbse/model/design.sysml">${esc(form.inputs)}</textarea>
              <div class="field__hint">Frozen into stage work at initialization; these replace the checker's <span class="mono">--list-inputs</span> invocation for the bound gate.</div>
              ${errorFor("inputs")}
            </div>
            <div class="lib-form__section-title" style="margin-top:16px"><span class="lib-form__step">04</span>Executor</div>
            <div class="executor-picker">
              <button class="executor-option ${form.execMode === "reuse" ? "is-selected" : ""}" type="button" data-form-exec="reuse" aria-pressed="${form.execMode === "reuse"}">
                <span class="executor-option__icon tone-green">${icon("terminal")}</span>
                <span><span class="executor-option__name">Reuse existing script</span><span class="executor-option__sub">Point at a project checker script.</span></span>
              </button>
              <button class="executor-option ${form.execMode === "scaffold" ? "is-selected" : ""}" type="button" data-form-exec="scaffold" aria-pressed="${form.execMode === "scaffold"}">
                <span class="executor-option__icon tone-cyan">${icon("sparkle")}</span>
                <span><span class="executor-option__name">Generate scaffold script</span><span class="executor-option__sub">Write a deterministic checker with TODO rules.</span></span>
              </button>
              <button class="executor-option ${form.execMode === "cli" ? "is-selected" : ""}" type="button" data-form-exec="cli" aria-pressed="${form.execMode === "cli"}">
                <span class="executor-option__icon tone-violet">${icon("cpu")}</span>
                <span><span class="executor-option__name">Agent CLI</span><span class="executor-option__sub">A codex/kimi run judges the same rules.</span></span>
              </button>
            </div>
            ${
              form.execMode === "cli"
                ? `<div class="form-grid" style="margin-top:12px">
                    <div class="field">
                      <label class="label" for="cp-cli">CLI <span class="req">*</span></label>
                      <select class="select" id="cp-cli" data-form-field="cli">${agents.map((entry) => `<option value="${esc(entry.cli)}" ${entry.cli === form.cli ? "selected" : ""}>${esc(entry.cli)}${entry.available ? "" : " · off"}</option>`).join("")}</select>
                      ${errorFor("executor")}
                    </div>
                    <div class="field">
                      <label class="label" for="cp-model">Model <span class="req">*</span></label>
                      <select class="select" id="cp-model" data-form-field="model">${models.map((model) => `<option value="${esc(model.id)}" ${model.id === form.model ? "selected" : ""}>${esc(model.label || model.id)}</option>`).join("") || `<option value="${esc(form.model)}" selected>${esc(form.model || "default")}</option>`}</select>
                      <div class="field__hint">Executor summary: <span class="mono">${esc(form.cli)}:${esc(form.model)}</span></div>
                    </div>
                  </div>`
                : form.execMode === "scaffold"
                  ? `<div class="field" style="margin-top:12px">
                      <label class="label">Scaffold script</label>
                      <input class="input input--mono" value="${esc(scaffold)}" readonly />
                      <div class="field__hint">The runtime writes a deterministic Python checker with one TODO rule function per declared rule; edit it before the first real check.</div>
                      ${errorFor("executor")}
                    </div>`
                  : `<div class="field" style="margin-top:12px">
                      <label class="label" for="cp-command">Script path <span class="req">*</span></label>
                      <input class="input input--mono ${form.errors.executor ? "is-invalid" : ""}" id="cp-command" data-form-field="command_uri" value="${esc(form.command_uri)}" placeholder="checkpoints/scripts/energy_budget_positive.py" spellcheck="false" />
                      <div class="field__hint">Project-relative <span class="mono">.py</span> implementing the checker interface.</div>
                      ${errorFor("executor")}
                    </div>`
            }
          </div>`
    }
    ${sample ? `<div class="lib-detail__note">${icon("info")}<span>Sample mode: the form is fully interactive for exploration, but saving is disabled.</span></div>` : ""}
  </form>`;
}

function validateCheckpointForm(form) {
  const errors = {};
  if (!form.kind) errors.kind = "Choose whether this checkpoint is a syntactic rule set or a semantic criterion set.";
  const checkpointId = (form.checkpoint_id || "").trim();
  if (!checkpointId) errors.checkpoint_id = "A checkpoint id is required.";
  else if (!CHECKPOINT_ID_RE.test(checkpointId)) errors.checkpoint_id = "Use 1–64 lowercase letters, digits, '-' or '_', starting with a letter or digit.";
  const title = (form.title || "").trim();
  if (!title) errors.title = "A title is required.";
  else if (title.length > 120) errors.title = "The title must be 120 characters or fewer.";
  const description = (form.description || "").trim();
  if (!description) errors.description = "A description is required: it is what a reviewer reads first.";
  else if (description.length > 1000) errors.description = "The description must be 1000 characters or fewer.";
  const tags = parseTags(form.tags);
  if (tags.length > 8) errors.tags = "Use at most eight tags.";
  else if (tags.some((tag) => !/^[a-z0-9][a-z0-9-]{0,31}$/.test(tag))) errors.tags = "Tags are lowercase slugs (letters, digits, '-').";
  if (form.kind === "syntactic") {
    const rules = form.rules.filter((rule) => (rule.rule_id || "").trim() || (rule.condition || "").trim() || (rule.reads || "").trim());
    if (rules.length === 0) errors.rules = "A syntactic checkpoint needs at least one rule.";
    else if (rules.some((rule) => !(rule.rule_id || "").trim() || !(rule.condition || "").trim())) {
      errors.rules = "Every rule needs a rule id and a condition.";
    } else if (rules.some((rule) => !/^[a-z0-9][a-z0-9-]{0,63}$/.test((rule.rule_id || "").trim()))) {
      errors.rules = "Rule ids are lowercase slugs (letters, digits, '-').";
    }
    const inputs = parseLines(form.inputs);
    if (inputs.length === 0) errors.inputs = "A syntactic checkpoint needs at least one input path.";
    if (form.execMode === "cli") {
      if (!["codex", "kimi"].includes(form.cli)) errors.executor = "Choose codex or kimi.";
      else if (!(form.model || "").trim()) errors.executor = "An agent CLI executor needs a model id.";
      else if ((form.model || "").trim().length > 200) errors.executor = "The model id must be 200 characters or fewer.";
    } else if (form.execMode === "reuse") {
      const command = (form.command_uri || "").trim();
      if (!command) errors.executor = "Enter the project-relative script path.";
      else if (!command.endsWith(".py")) errors.executor = "The script path must end with .py.";
    }
  } else if (form.kind === "semantic") {
    const criteria = form.criteria.map((criterion) => (criterion || "").trim()).filter(Boolean);
    if (criteria.length === 0) errors.criteria = "A semantic checkpoint needs at least one criterion.";
    if (!form.judges.agent && !form.judges.human) errors.judges = "Allow at least one judge (agent or human).";
  }
  return Object.fromEntries(Object.entries(errors).filter(([, value]) => Boolean(value)));
}

function buildRequestBody(form) {
  const body = {
    checkpoint_id: (form.checkpoint_id || "").trim(),
    title: (form.title || "").trim(),
    description: (form.description || "").trim(),
    kind: form.kind,
    tags: parseTags(form.tags),
  };
  if (form.kind === "semantic") {
    body.criteria = form.criteria.map((criterion) => (criterion || "").trim()).filter(Boolean);
    body.judges = { agent: Boolean(form.judges.agent), human: Boolean(form.judges.human) };
  } else {
    body.rules = form.rules
      .filter((rule) => (rule.rule_id || "").trim() && (rule.condition || "").trim())
      .map((rule) => ({
        rule_id: rule.rule_id.trim(),
        condition: rule.condition.trim(),
        reads: parseLines(rule.reads),
      }));
    body.inputs = parseLines(form.inputs);
    if (form.execMode === "cli") {
      body.executor = { mode: "cli", cli: form.cli, model: (form.model || "").trim() };
    } else if (form.execMode === "scaffold") {
      body.executor = { mode: "script", command_uri: scaffoldUri(body.checkpoint_id) };
      body.create_script = "scaffold";
    } else {
      body.executor = { mode: "script", command_uri: (form.command_uri || "").trim() };
    }
  }
  if (form.editingId) body.version = 0; // server derives the next version; field is advisory only
  return body;
}

function scaffoldUri(checkpointId) {
  const slug = (checkpointId || "checkpoint").trim().replace(/-/g, "_").replace(/[^a-z0-9_]/g, "");
  return `checkpoints/scripts/${slug || "checkpoint"}.py`;
}

function parseLines(value) {
  return String(value || "")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
}

function parseTags(value) {
  return String(value || "")
    .split(",")
    .map((tag) => tag.trim().toLowerCase())
    .filter(Boolean);
}

function mapServerErrorToField(message) {
  const text = String(message || "").toLowerCase();
  if (/(checkpoint|slug).*id|duplicate/.test(text)) return "checkpoint_id";
  if (text.includes("title")) return "title";
  if (text.includes("description")) return "description";
  if (text.includes("rule")) return "rules";
  if (text.includes("input")) return "inputs";
  if (text.includes("executor") || text.includes("command_uri") || text.includes("cli") || text.includes("model")) return "executor";
  if (text.includes("criterion") || text.includes("criteria") || text.includes("judge")) return "criteria";
  if (text.includes("tag")) return "tags";
  return null;
}
