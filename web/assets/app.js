// xdesign · WishFlow Studio — application bootstrap, hash router, polling and
// shared state. Everything below runs in the browser; there is no build step.

import { api } from "./js/api.js";
import {
  hydrateIcons,
  parseHash,
  relativeTime,
  sortWishes,
  statusChip,
} from "./js/util.js";
import { toastError, toastWarn, toastInfo, closeModal, errorState } from "./js/components.js";
import {
  SAMPLE_HEALTH,
  SAMPLE_FLOWS,
  SAMPLE_WISHES,
  getSampleCheckpointLibrary,
  getSampleCheckpoint,
} from "./js/sample-data.mjs";
import { overviewView } from "./js/views/overview.js";
import { wishesView } from "./js/views/wishes.js";
import { wizardView } from "./js/views/wizard.js";
import { wishDetailView } from "./js/views/wish-detail.js";
import { flowsView } from "./js/views/flows.js";
import { checkpointsView } from "./js/views/checkpoints.js";

const APP_VERSION = "0.1.0";
const POLL_INTERVAL_MS = 4000;
// Sample data is display-only and used only when the API is unreachable or the
// operator explicitly asks for a demo. A reachable store with no wishes is a
// real empty state (SPEC §7 empty-versus-sample rule).
const DEMO_REQUESTED = new URLSearchParams(window.location.search).get("demo") === "1";

class WishFlowApp {
  constructor() {
    this.state = {
      version: APP_VERSION,
      health: null,
      flows: [],
      agents: [],
      flowById: new Map(),
      wishes: [],
      checkpoints: [],
      checkpointById: new Map(),
      checkpointsLoaded: false,
      checkpointsError: null,
      checkpointDetailById: new Map(),
      sampleMode: false,
      sampleReason: "",
      demoRequested: DEMO_REQUESTED,
      metaFromSample: false,
      lastUpdated: null,
      navToken: 0,
      booted: false,
      bootError: null,
    };
    this.views = {
      overview: overviewView,
      wishes: wishesView,
      new: wizardView,
      "wish-detail": wishDetailView,
      flows: flowsView,
      checkpoints: checkpointsView,
    };
    this.current = null;
    this.pollTimer = null;
    this.liveTimer = null;
    this.polling = false;
    this.cleanup = null;
  }

  /* ---------------------------------------------------------------- */
  /* Boot                                                              */
  /* ---------------------------------------------------------------- */

  async boot() {
    hydrateIcons(document);
    this.installMobileNav();
    this.bindShell();
    this.bindGlobalKeys();
    this.startLiveTicker();
    window.addEventListener("hashchange", () => this.handleRoute());

    try {
      await this.loadMeta();
      if (this.state.demoRequested) this.enterSampleMode("demo");
      else await this.loadWishes();
    } catch (error) {
      // Sample mode keeps the console usable; nothing here is fatal.
      this.enterSampleMode("offline");
    }
    this.state.booted = true;
    await this.handleRoute({ initial: true });
    window.XDESIGN_WEB = {
      version: APP_VERSION,
      ready: true,
      sample: this.state.sampleMode,
      demo: this.state.demoRequested,
      views: Object.keys(this.views),
    };
  }

  async loadMeta() {
    let health = null;
    let flowData = null;
    try {
      [health, flowData] = await Promise.all([api.health(), api.flows()]);
      this.state.metaFromSample = false;
    } catch (error) {
      health = SAMPLE_HEALTH;
      flowData = SAMPLE_FLOWS;
      this.state.metaFromSample = true;
    }
    this.state.health = health;
    this.state.flows = flowData.flows || [];
    this.state.agents = flowData.agents || [];
    this.state.flowById = new Map(this.state.flows.map((flow) => [flow.flow_id, flow]));
    this.updateShell();
  }

  async loadWishes() {
    if (this.state.demoRequested) {
      this.enterSampleMode("demo");
      return this.state.wishes;
    }
    try {
      const payload = await api.wishes();
      const list = Array.isArray(payload && payload.wishes) ? payload.wishes : [];
      this.exitSampleMode();
      this.state.wishes = sortWishes(list);
      this.state.lastUpdated = Date.now();
      this.updateShell();
      return this.state.wishes;
    } catch (error) {
      this.enterSampleMode("offline");
      return this.state.wishes;
    }
  }

  async loadCheckpoints({ force = false } = {}) {
    if (this.state.checkpointsLoaded && !force) return this.state.checkpoints;
    if (this.state.sampleMode) {
      this.setCheckpoints(getSampleCheckpointLibrary().checkpoints);
      this.state.checkpointsError = null;
      return this.state.checkpoints;
    }
    try {
      const payload = await api.checkpoints();
      const list = Array.isArray(payload && payload.checkpoints) ? payload.checkpoints : [];
      this.setCheckpoints(list);
      this.state.checkpointsError = null;
    } catch (error) {
      this.setCheckpoints([]);
      this.state.checkpointsError = error;
    }
    return this.state.checkpoints;
  }

  setCheckpoints(list) {
    this.state.checkpoints = list || [];
    this.state.checkpointById = new Map(this.state.checkpoints.map((entry) => [entry.checkpoint_id, entry]));
    this.state.checkpointsLoaded = true;
    this.state.checkpointDetailById = new Map();
  }

  async loadCheckpointDetail(checkpointId, { force = false } = {}) {
    if (!checkpointId) return null;
    if (!force && this.state.checkpointDetailById.has(checkpointId)) {
      return this.state.checkpointDetailById.get(checkpointId);
    }
    try {
      const reply = this.state.sampleMode
        ? getSampleCheckpoint(checkpointId)
        : await api.checkpoint(checkpointId);
      if (!reply || !reply.checkpoint) return null;
      this.state.checkpointDetailById.set(checkpointId, reply.checkpoint);
      return reply.checkpoint;
    } catch (error) {
      return null;
    }
  }

  checkpointSuggestions(kind) {
    return this.state.checkpoints.filter((entry) => !kind || entry.kind === kind);
  }

  enterSampleMode(reason = "offline") {
    this.state.sampleMode = true;
    this.state.sampleReason = reason;
    this.state.wishes = sortWishes(SAMPLE_WISHES);
    this.setCheckpoints(getSampleCheckpointLibrary().checkpoints);
    if (this.state.metaFromSample || this.state.flows.length === 0) {
      this.state.flows = SAMPLE_FLOWS.flows;
      this.state.agents = SAMPLE_FLOWS.agents;
      this.state.flowById = new Map(this.state.flows.map((flow) => [flow.flow_id, flow]));
    }
    this.updateShell();
  }

  exitSampleMode() {
    const wasSample = this.state.sampleMode;
    this.state.sampleMode = false;
    this.state.sampleReason = "";
    if (wasSample) {
      this.state.checkpointsLoaded = false;
      this.state.checkpoints = [];
      this.state.checkpointById = new Map();
    }
  }

  /* ---------------------------------------------------------------- */
  /* Shell chrome                                                      */
  /* ---------------------------------------------------------------- */

  installMobileNav() {
    if (document.querySelector(".mobile-nav")) return;
    const nav = document.createElement("nav");
    nav.className = "mobile-nav";
    nav.setAttribute("aria-label", "Views");
    nav.innerHTML = `
      <a class="mobile-nav__link" href="#/overview" data-route="overview" aria-label="Overview"><span data-icon="overview"></span><span>Overview</span></a>
      <a class="mobile-nav__link" href="#/wishes" data-route="wishes" aria-label="Wishes"><span data-icon="wishes"></span><span>Wishes</span></a>
      <a class="mobile-nav__link mobile-nav__link--primary" href="#/new" data-route="new" aria-label="New wish"><span data-icon="plus"></span></a>
      <a class="mobile-nav__link" href="#/checkpoints" data-route="checkpoints" aria-label="Checkpoints"><span data-icon="library"></span><span>Library</span></a>
      <a class="mobile-nav__link" href="#/flows" data-route="flows" aria-label="Flows"><span data-icon="flows"></span><span>Flows</span></a>`;
    document.body.appendChild(nav);
    hydrateIcons(nav);
  }

  bindShell() {
    const refresh = document.getElementById("refresh-btn");
    if (refresh) refresh.addEventListener("click", () => this.refreshCurrent({ userInitiated: true }));

    const newWish = document.getElementById("topbar-new");
    if (newWish) {
      newWish.addEventListener("click", (event) => {
        if (this.state.sampleMode) {
          event.preventDefault();
          toastWarn(
            "Sample mode: writes disabled",
            "The demo dataset cannot create wishes. Start `xdesign web` against a real store to enable this action.",
          );
        }
      });
    }

    window.addEventListener("online", () => {
      if (this.state.sampleMode) toastInfo("Network state changed", "Refresh to retry the local xdesign server.");
    });
  }

  bindGlobalKeys() {
    document.addEventListener("keydown", (event) => {
      const target = event.target;
      const typing =
        target &&
        (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.tagName === "SELECT" || target.isContentEditable);
      if (typing || event.metaKey || event.ctrlKey || event.altKey) return;
      if (document.querySelector(".modal-root.is-open")) return;
      if (event.key === "/") {
        const search = document.querySelector("[data-wish-search]");
        if (search) {
          event.preventDefault();
          search.focus();
        }
      } else if (event.key === "r") {
        this.refreshCurrent({ userInitiated: true });
      } else if (event.key === "n") {
        location.hash = "#/new";
      }
    });
  }

  updateShell() {
    const { health, agents, wishes, sampleMode, flows } = this.state;

    const agentChips = document.getElementById("agent-chips");
    if (agentChips) {
      agentChips.innerHTML = agents
        .map((agent) => {
          const on = Boolean(agent.available);
          return `<span class="agent-chip ${on ? "is-on" : "is-off"}" title="${on ? `${agent.cli} detected${agent.version ? " · " + agent.version : ""}` : `${agent.cli} not detected on this machine`}">
            <span class="dot"></span><span>${agent.cli}</span>
          </span>`;
        })
        .join("");
    }

    const projectPath = document.getElementById("project-path-value");
    if (projectPath && health) {
      projectPath.textContent = health.project_root || "…";
      projectPath.title = health.project_root || "";
      const container = document.getElementById("project-path");
      if (container) container.title = `Project root: ${health.project_root || "unknown"}`;
    }

    const storeCard = document.getElementById("store-card");
    const storeStatus = document.getElementById("store-status");
    const storeDot = document.getElementById("store-dot");
    const storePath = document.getElementById("store-path");
    const storeVersion = document.getElementById("store-version");
    const storeMissing = !sampleMode && health && health.store_present === false;
    if (storeCard && storeStatus) {
      storeCard.classList.toggle("is-sample", sampleMode);
      storeCard.classList.toggle("is-empty", storeMissing);
      storeStatus.textContent = sampleMode ? "Sample data" : storeMissing ? "No store yet" : "Store connected";
    }
    if (storeDot) {
      storeDot.title = sampleMode
        ? "Display-only demo dataset"
        : storeMissing
          ? "The project has no .xdesign store yet; create the first wish to initialise it"
          : "Operational store reachable";
    }
    if (storePath) {
      storePath.textContent = health?.store_root || (sampleMode ? ".xdesign (demo)" : ".xdesign");
      storePath.title = health?.store_root || "";
    }
    if (storeVersion) storeVersion.textContent = `xdesign ${health?.version || APP_VERSION} · ${sampleMode ? "demo" : storeMissing ? "empty" : "live"}`;

    const footerBuild = document.getElementById("footer-build");
    if (footerBuild) footerBuild.textContent = `xdesign web ${APP_VERSION} · embedded assets`;

    const railCount = document.getElementById("rail-wish-count");
    if (railCount) {
      railCount.hidden = wishes.length === 0;
      railCount.textContent = String(wishes.length);
    }

    const topbarNew = document.getElementById("topbar-new");
    if (topbarNew) {
      topbarNew.classList.toggle("is-disabled", sampleMode);
      if (sampleMode) topbarNew.setAttribute("aria-disabled", "true");
      else topbarNew.removeAttribute("aria-disabled");
      topbarNew.title = sampleMode ? "Writes are disabled in sample mode" : "Create a wish";
    }

    const banner = document.getElementById("sample-banner");
    const bannerText = document.getElementById("sample-banner-text");
    if (banner) banner.hidden = !sampleMode;
    if (bannerText && sampleMode) {
      bannerText.innerHTML =
        this.state.sampleReason === "demo"
          ? "Demo mode was requested with <span class=\"mono\">?demo=1</span>, so this console is showing a labelled demo dataset. Every write action is disabled; remove the query parameter to operate the real store."
          : "The xdesign API is unreachable, so this console is showing a labelled demo dataset. Every write action is disabled; start <span class=\"mono\">xdesign web</span> against a real project to operate a store.";
    }
    this.updateLiveChip();
  }

  updateLiveChip() {
    const chip = document.getElementById("live-chip");
    const label = document.getElementById("live-chip-label");
    if (!chip || !label) return;
    if (this.state.sampleMode) {
      chip.classList.add("is-paused");
      label.textContent = "sample";
      chip.title =
        this.state.sampleReason === "demo"
          ? "Demo mode (?demo=1): polling is paused and writes are disabled"
          : "Sample data: the API is unreachable, so polling is paused and writes are disabled";
    } else if (document.hidden) {
      chip.classList.add("is-paused");
      label.textContent = "paused";
      chip.title = "Polling pauses while the tab is hidden";
    } else {
      chip.classList.remove("is-paused");
      label.textContent = "live";
      chip.title = this.state.lastUpdated
        ? `Polling every 4 s · updated ${relativeTime(this.state.lastUpdated)}`
        : "Polling every 4 s";
    }
  }

  startLiveTicker() {
    if (this.liveTimer) clearInterval(this.liveTimer);
    this.liveTimer = setInterval(() => this.updateLiveChip(), 1000);
    document.addEventListener("visibilitychange", () => this.updateLiveChip());
  }

  /* ---------------------------------------------------------------- */
  /* Routing                                                           */
  /* ---------------------------------------------------------------- */

  async handleRoute({ initial = false } = {}) {
    const parsed = parseHash(location.hash);
    let { route, param } = parsed;
    if (!this.views[route]) {
      route = "overview";
      param = null;
      if (!initial) history.replaceState(null, "", "#/overview");
    }
    const view = this.views[route];
    const token = ++this.state.navToken;

    this.teardownCurrent();
    closeModal();

    if (!initial) window.scrollTo({ top: 0, behavior: "auto" });

    this.updateCrumb(view, route, param);
    this.setActiveNav(route);
    document.title =
      route === "overview"
        ? "xdesign · WishFlow Studio"
        : `${this.crumbText(view, route, param)} · WishFlow Studio`;

    const root = document.getElementById("view");
    if (!root) return;
    root.setAttribute("aria-busy", "true");
    root.innerHTML = view.skeleton ? view.skeleton(this) : `<div class="loading-block"><span class="spinner"></span><span>Loading…</span></div>`;

    let data;
    try {
      data = await view.load(this, param);
    } catch (error) {
      if (token !== this.state.navToken) return;
      root.removeAttribute("aria-busy");
      root.innerHTML = errorState({
        title: "This view could not be loaded",
        text: `${error.message || "Unexpected error"}${this.state.sampleMode ? "" : " Check that `xdesign web` is still running."}`,
        actions: `<button class="btn" type="button" data-retry-route>${"Retry"}</button><a class="btn btn--ghost" href="#/overview">Back to overview</a>`,
      });
      const retry = root.querySelector("[data-retry-route]");
      if (retry) retry.addEventListener("click", () => this.handleRoute());
      return;
    }
    if (token !== this.state.navToken) return;

    root.removeAttribute("aria-busy");
    root.innerHTML = view.render(this, data, param);
    this.current = { view, route, param, data };
    if (view.mount) {
      const cleanup = view.mount(this, root, data, param);
      this.cleanup = typeof cleanup === "function" ? cleanup : null;
    }
    hydrateIcons(root);
    this.startPolling();
    this.updateLiveChip();
    const main = document.getElementById("main");
    if (main && !initial) main.focus({ preventScroll: true });
  }

  crumbText(view, route, param) {
    if (typeof view.crumb === "function") return view.crumb(this, param);
    return view.title || route;
  }

  updateCrumb(view, route, param) {
    const here = document.getElementById("crumb-here");
    if (here) here.textContent = this.crumbText(view, route, param);
  }

  setActiveNav(route) {
    const activeRoute = route === "wish-detail" ? "wishes" : route;
    document.querySelectorAll("[data-route]").forEach((link) => {
      const isActive = link.dataset.route === activeRoute;
      link.classList.toggle("is-active", isActive);
      if (isActive) link.setAttribute("aria-current", "page");
      else link.removeAttribute("aria-current");
    });
  }

  teardownCurrent() {
    if (this.pollTimer) {
      clearInterval(this.pollTimer);
      this.pollTimer = null;
    }
    if (this.cleanup) {
      try {
        this.cleanup();
      } catch {
        /* cleanup is best-effort */
      }
      this.cleanup = null;
    }
    this.current = null;
  }

  /* ---------------------------------------------------------------- */
  /* Polling / refresh                                                 */
  /* ---------------------------------------------------------------- */

  startPolling() {
    if (this.pollTimer) clearInterval(this.pollTimer);
    const { current } = this;
    if (!current || this.state.sampleMode) return;
    const view = current.view;
    if (view.poll === false) return;
    this.pollTimer = setInterval(() => this.pollTick(), POLL_INTERVAL_MS);
  }

  async pollTick() {
    if (document.hidden || this.polling) return;
    if (!this.current || this.state.sampleMode) return;
    if (document.querySelector(".modal-root.is-open")) return;
    const active = document.activeElement;
    if (active && active.closest && active.closest("#view")) {
      if (active.matches("input, textarea, select, [contenteditable]")) return;
    }
    await this.refreshCurrent({ silent: true });
  }

  async refreshCurrent({ userInitiated = false, silent = false } = {}) {
    if (!this.current || this.state.sampleMode) {
      if (userInitiated && this.state.sampleMode) {
        toastInfo(
          this.state.sampleReason === "demo" ? "Demo mode" : "Sample mode",
          this.state.sampleReason === "demo"
            ? "Remove ?demo=1 from the URL to operate the real store."
            : "Start `xdesign web` against a real store, then refresh to leave the demo dataset.",
        );
      }
      return;
    }
    const refreshButton = document.getElementById("refresh-btn");
    if (userInitiated && refreshButton) {
      refreshButton.classList.add("is-busy");
      refreshButton.disabled = true;
    }
    const token = this.state.navToken;
    const { current } = this;
    try {
      let nextData = current.data;
      if (typeof current.view.pollRefresh === "function") {
        nextData = await current.view.pollRefresh(this, current.data, current.param, document.getElementById("view"));
      } else if (typeof current.view.refresh === "function") {
        nextData = await current.view.refresh(this, current.data, current.param);
        const root = document.getElementById("view");
        const scrollY = window.scrollY;
        if (token === this.state.navToken && root) {
          if (this.cleanup) this.cleanup();
          root.innerHTML = current.view.render(this, nextData, current.param);
          current.data = nextData;
          if (current.view.mount) {
            const cleanup = current.view.mount(this, root, nextData, current.param);
            this.cleanup = typeof cleanup === "function" ? cleanup : null;
          }
          hydrateIcons(root);
          window.scrollTo({ top: scrollY, behavior: "auto" });
        }
      }
      if (token !== this.state.navToken) return;
      if (nextData !== undefined) current.data = nextData;
      this.state.lastUpdated = Date.now();
      this.updateShell();
      if (userInitiated && !silent) toastInfo("View refreshed", "Read directly from the local xdesign store.");
    } catch (error) {
      if (userInitiated) toastError("Refresh failed", error.message || "Unexpected error");
    } finally {
      if (userInitiated && refreshButton) {
        refreshButton.classList.remove("is-busy");
        refreshButton.disabled = false;
      }
    }
  }

  /* ---------------------------------------------------------------- */
  /* Helpers exposed to views                                          */
  /* ---------------------------------------------------------------- */

  wishById(wishId) {
    return this.state.wishes.find((wish) => wish.wish_id === wishId) || null;
  }

  requireWritable() {
    if (this.state.sampleMode) {
      toastWarn("Sample mode: writes disabled", "This labelled demo dataset never writes to the repository.");
      return false;
    }
    return true;
  }

  openWish(wishId) {
    location.hash = `#/wishes/${encodeURIComponent(wishId)}`;
  }

  navigate(hash) {
    if (location.hash === hash) this.handleRoute();
    else location.hash = hash;
  }
}

const app = new WishFlowApp();

window.addEventListener("unhandledrejection", (event) => {
  if (!app.state.booted) return;
  const reason = event.reason;
  if (reason && reason.name === "ApiError") return;
  // Keep the console usable; surface unexpected failures as a toast.
  toastError("Unexpected error", String((reason && reason.message) || reason));
});

app.boot().catch((error) => {
  const root = document.getElementById("view");
  if (root) {
    root.innerHTML = errorState({
      title: "WishFlow Studio failed to start",
      text: String((error && error.message) || error),
      actions: `<button class="btn" type="button" onclick="location.reload()">Reload</button>`,
    });
  }
  window.XDESIGN_WEB = { version: APP_VERSION, ready: false, error: String(error) };
});

export { app };
