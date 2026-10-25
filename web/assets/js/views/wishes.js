// xdesign · WishFlow Studio — Wishes: searchable, filterable register with
// status chips, flow filter and a miniature stage pipeline per wish.

import { esc, icon, parseHash, statusMeta } from "../util.js";
import { wishCard, emptyState, toastWarn } from "../components.js";

const STATUS_FILTERS = ["all", "active", "blocked", "delivered", "received"];
const FLOW_LABEL = { "simple-flow": "SimpleFlow", "wish-verify": "WishVerifyFlow" };

export const wishesView = {
  title: "Wishes",
  poll: true,

  crumb() {
    return "Wishes";
  },

  skeleton() {
    return `<div class="skeleton skeleton--line" style="width:240px;height:28px"></div>
      <div class="skeleton" style="height:64px;margin-top:18px"></div>
      <div class="skeleton skeleton--panel" style="height:420px"></div>`;
  },

  async load(app) {
    await app.loadWishes();
    return { wishes: app.state.wishes };
  },

  async refresh(app) {
    await app.loadWishes();
    return { wishes: app.state.wishes };
  },

  async pollRefresh(app, data, param, root) {
    const before = signature(app.state.wishes);
    await app.loadWishes();
    const after = signature(app.state.wishes);
    if (before === after || !root) return { wishes: app.state.wishes };
    const wishes = app.state.wishes;
    const filters = readFilters(app, wishes);
    const countEl = root.querySelector("[data-result-count]");
    if (countEl) countEl.textContent = countLabel(wishes, filters);
    const filtersEl = root.querySelector("[data-status-filters]");
    if (filtersEl) {
      filtersEl.innerHTML = renderStatusFilters(wishes, filters);
      bindStatusChips(app, root, wishes);
    }
    const listEl = root.querySelector("[data-wish-list]");
    if (listEl) listEl.innerHTML = renderList(app, wishes, filters);
    return { wishes };
  },

  render(app, data) {
    const wishes = (data && data.wishes) || app.state.wishes;
    const filters = readFilters(app, wishes);
    const flows = app.state.flows || [];
    const sample = app.state.sampleMode;

    return `<div class="page-head animate-rise">
      <div class="page-head__titles">
        <span class="eyebrow">Wish register</span>
        <h1 class="h1">Wishes in this workspace</h1>
        <p class="lede">Search the operational store and legacy traces, filter by status or flow, and open a wish to inspect its frozen plan, assessments and retained evidence.</p>
      </div>
      <div class="page-head__actions">
        ${
          sample
            ? `<button class="btn btn--primary is-disabled" type="button" data-sample-write title="Writes are disabled in sample mode">${icon("plus")}New wish</button>`
            : `<a class="btn btn--primary" href="#/new">${icon("plus")}New wish</a>`
        }
      </div>
    </div>

    <div class="toolbar animate-rise is-delay-1">
      <label class="search">
        <span class="icon">${icon("search")}</span>
        <input type="search" data-wish-search placeholder="Search wishes…" autocomplete="off" spellcheck="false" aria-label="Search wishes" value="${esc(filters.q)}" />
        <button class="search__clear" type="button" data-search-clear aria-label="Clear search" ${filters.q ? "" : 'hidden'}>${icon("close")}</button>
      </label>
      <div class="filter-group" data-status-filters>
        ${renderStatusFilters(wishes, filters)}
      </div>
      <select class="toolbar__select" data-flow-filter aria-label="Filter by flow">
        <option value="all">All flows</option>
        ${flows.map((flow) => `<option value="${esc(flow.flow_id)}" ${filters.flow === flow.flow_id ? "selected" : ""}>${esc(FLOW_LABEL[flow.flow_id] || flow.title || flow.flow_id)}</option>`).join("")}
      </select>
      <span class="toolbar__spacer"></span>
      <span class="toolbar__count" data-result-count>${esc(countLabel(wishes, filters))}</span>
    </div>

    <div class="animate-rise is-delay-2" data-wish-list>
      ${renderList(app, wishes, filters)}
    </div>`;
  },

  mount(app, root, data) {
    const wishes = (data && data.wishes) || app.state.wishes;
    const search = root.querySelector("[data-wish-search]");
    const clear = root.querySelector("[data-search-clear]");
    const flowFilter = root.querySelector("[data-flow-filter]");
    let debounceTimer = null;

    const apply = () => {
      const filters = readFilters(app, wishes);
      const listEl = root.querySelector("[data-wish-list]");
      const countEl = root.querySelector("[data-result-count]");
      if (countEl) countEl.textContent = countLabel(wishes, filters);
      if (listEl) listEl.innerHTML = renderList(app, wishes, filters);
      const filtersEl = root.querySelector("[data-status-filters]");
      if (filtersEl) {
        filtersEl.innerHTML = renderStatusFilters(wishes, filters);
        bindStatusChips(app, root, wishes);
      }
      if (search) {
        search.value = filters.q;
        search.focus({ preventScroll: true });
        const position = search.value.length;
        if (document.activeElement === search) search.setSelectionRange(position, position);
      }
      if (clear) clear.hidden = !filters.q;
    };

    bindStatusChips(app, root, wishes);

    if (search) {
      search.addEventListener("input", () => {
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => {
          updateFilter("q", search.value.trim());
          apply();
        }, 140);
      });
      search.addEventListener("keydown", (event) => {
        if (event.key === "Escape") {
          search.value = "";
          updateFilter("q", "");
          apply();
        }
      });
    }
    if (clear) {
      clear.addEventListener("click", () => {
        if (search) search.value = "";
        updateFilter("q", "");
        apply();
      });
    }
    if (flowFilter) {
      flowFilter.addEventListener("change", () => {
        updateFilter("flow", flowFilter.value === "all" ? "" : flowFilter.value);
        apply();
      });
    }

    const sampleGuard = root.querySelector("[data-sample-write]");
    if (sampleGuard) {
      sampleGuard.addEventListener("click", (event) => {
        event.preventDefault();
        toastWarn("Sample mode: writes disabled", "The labelled demo dataset cannot create wishes.");
      });
    }

    return () => clearTimeout(debounceTimer);
  },
};

/* ------------------------------------------------------------------ */
/* Filters                                                            */
/* ------------------------------------------------------------------ */

function readFilters(app, wishes) {
  const { params } = parseHash(location.hash);
  const statusParam = params.get("status");
  const status = STATUS_FILTERS.includes(statusParam) ? statusParam : "all";
  const flow = params.get("flow") || "all";
  const q = params.get("q") || "";
  return { status, flow, q };
}

function updateFilter(key, value) {
  const { params } = parseHash(location.hash);
  const search = new URLSearchParams(params);
  if (value) search.set(key, value);
  else search.delete(key);
  // Drop default values so the URL stays tidy.
  if (key === "status" && value === "all") search.delete(key);
  if (key === "flow" && value === "all") search.delete(key);
  const query = search.toString();
  const next = `#/wishes${query ? "?" + query : ""}`;
  history.replaceState(null, "", next);
}

function matches(wish, filters, app) {
  if (filters.status !== "all" && wish.status !== filters.status) return false;
  if (filters.flow !== "all" && wish.flow_id !== filters.flow) return false;
  if (filters.q) {
    const haystack = `${wish.wish_id} ${wish.title} ${wish.description || ""}`.toLowerCase();
    if (!haystack.includes(filters.q.toLowerCase())) return false;
  }
  return true;
}

function filteredWishes(wishes, filters, app) {
  return wishes.filter((wish) => matches(wish, filters, app));
}

function countLabel(wishes, filters) {
  const shown = filteredWishes(wishes, filters).length;
  return `${shown} of ${wishes.length} wish${wishes.length === 1 ? "" : "es"}`;
}

function renderStatusFilters(wishes, filters) {
  const counts = { all: wishes.length };
  for (const wish of wishes) counts[wish.status] = (counts[wish.status] || 0) + 1;
  return `<span class="filter-label">Status</span>${STATUS_FILTERS.map((status) => {
    const active = filters.status === status;
    const label = status === "all" ? "All" : statusMeta(status).label;
    return `<button class="filter-chip tone-${status === "all" ? "ink" : statusMeta(status).tone}${active ? " is-active" : ""}" type="button" data-filter-status="${status}" aria-pressed="${active}">
      ${status !== "all" ? '<span class="dot"></span>' : ""}<span>${esc(label)}</span><span class="count">${counts[status] || 0}</span>
    </button>`;
  }).join("")}`;
}

function bindStatusChips(app, root, wishes) {
  root.querySelectorAll("[data-filter-status]").forEach((chip) => {
    if (chip.dataset.bound) return;
    chip.dataset.bound = "1";
    chip.addEventListener("click", () => {
      const value = chip.dataset.filterStatus;
      updateFilter("status", value === "all" ? "" : value);
      const filters = readFilters(app, wishes);
      const countEl = root.querySelector("[data-result-count]");
      const listEl = root.querySelector("[data-wish-list]");
      if (countEl) countEl.textContent = countLabel(wishes, filters);
      if (listEl) listEl.innerHTML = renderList(app, wishes, filters);
      const filtersEl = root.querySelector("[data-status-filters]");
      if (filtersEl) {
        filtersEl.innerHTML = renderStatusFilters(wishes, filters);
        bindStatusChips(app, root, wishes);
      }
    });
  });
}

function renderList(app, wishes, filters) {
  const shown = filteredWishes(wishes, filters, app);
  if (wishes.length === 0) {
    return `<div class="panel">${emptyState({
      title: "No wishes yet",
      text: "Create the first wish to start a run. The wizard writes a wish source and freezes a plan binding configuration.",
      actions: `<a class="btn btn--primary btn--sm" href="#/new">${icon("plus")}Create the first wish</a>`,
    })}</div>`;
  }
  if (shown.length === 0) {
    return `<div class="panel">${emptyState({
      title: "No wishes match these filters",
      text: `Nothing in the register matches the current search and filter combination.`,
      actions: `<button class="btn btn--sm" type="button" data-clear-filters>${icon("close")}Clear filters</button>`,
    })}</div>`;
  }
  return `<div class="wish-grid">
    <div class="wish-grid__head">
      <span></span><span>Wish</span><span>Flow</span><span>Status</span><span style="text-align:right">Revision · age</span>
    </div>
    ${shown.map((wish) => wishCard(wish, app)).join("")}
  </div>`;
}

function signature(wishes) {
  return (wishes || []).map((wish) => `${wish.wish_id}:${wish.status}:${wish.current_stage}:${wish.head_revision}:${wish.updated_at}`).join("|");
}
