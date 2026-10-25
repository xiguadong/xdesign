// xdesign · WishFlow Studio — Overview: hero, lifecycle map, stat cards,
// flow pipelines, recent wishes and agent runtimes.

import {
  esc,
  icon,
  relativeTime,
  statusChip,
  flowChip,
  statusMeta,
  truncate,
} from "../util.js";
import { wishRow, emptyState, toastWarn } from "../components.js";

const FLOW_LABEL = { "simple-flow": "SimpleFlow", "wish-verify": "WishVerifyFlow" };

export const overviewView = {
  title: "Overview",
  poll: true,

  crumb() {
    return "Overview";
  },

  skeleton() {
    return `<div class="boot-skeleton" aria-busy="true" aria-label="Loading overview">
      <div class="skeleton skeleton--hero"></div>
      <div class="skeleton-grid">
        <div class="skeleton skeleton--stat"></div>
        <div class="skeleton skeleton--stat"></div>
        <div class="skeleton skeleton--stat"></div>
        <div class="skeleton skeleton--stat"></div>
      </div>
      <div class="skeleton skeleton--panel"></div>
    </div>`;
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
    if (before !== after || !root) return { wishes: app.state.wishes };
    const wishes = app.state.wishes;
    // Surgical refresh of the read-only regions only.
    const stats = root.querySelector("[data-stat-grid]");
    if (stats) stats.innerHTML = renderStats(wishes);
    const pipeline = root.querySelector("[data-pipeline-body]");
    if (pipeline) pipeline.innerHTML = renderPipeline(app, wishes);
    const recent = root.querySelector("[data-recent-body]");
    if (recent) recent.innerHTML = renderRecent(app, wishes);
    return { wishes };
  },

  render(app, data) {
    const wishes = (data && data.wishes) || app.state.wishes;
    const health = app.state.health || {};
    const agents = app.state.agents || [];
    const flows = app.state.flows || [];

    return `
      <div class="animate-rise">
        ${renderHero(app, wishes, flows, agents)}
      </div>
      <div class="stat-grid animate-rise is-delay-1" data-stat-grid>
        ${renderStats(wishes)}
      </div>
      <div class="overview-grid animate-rise is-delay-2">
        <div class="overview-col">
          <section class="panel">
            <div class="panel__head">
              <div class="panel__title">${icon("layers")}Flow pipelines</div>
              <div class="panel__subtitle">Stage status from wishes currently in each flow</div>
            </div>
            <div class="pipeline" data-pipeline-body>${renderPipeline(app, wishes)}</div>
          </section>
          <section class="panel">
            <div class="panel__head">
              <div class="panel__title">${icon("wishes")}Recent wishes</div>
              <a class="btn btn--ghost btn--sm" href="#/wishes">View all${icon("arrow-right")}</a>
            </div>
            <div class="wish-list" data-recent-body>${renderRecent(app, wishes)}</div>
          </section>
        </div>
        <div class="overview-col">
          <section class="panel">
            <div class="panel__head">
              <div class="panel__title">${icon("cpu")}Agent runtimes</div>
              <div class="panel__subtitle">${agents.filter((agent) => agent.available).length}/${agents.length} detected</div>
            </div>
            <div class="agent-list">${renderAgents(agents)}</div>
            <div class="agent-note">
              <span class="icon">${icon("info")}</span>
              <span><strong>Default bindings</strong> run the deterministic checker. <strong>Agent CLI bindings</strong> are executed by the CLI, recorded with agent authority, and frozen into the plan at creation.</span>
            </div>
          </section>
          <section class="panel">
            <div class="panel__head">
              <div class="panel__title">${icon("database")}Environment</div>
              <div class="panel__subtitle">local runtime</div>
            </div>
            <div class="panel__body" style="display:flex;flex-direction:column;gap:10px">
              ${envRow("Version", health.version ? `xdesign ${health.version}` : "0.1.0")}
              ${envRow("Project", health.project_root || "—", true)}
              ${envRow("Store", health.store_root || ".xdesign", true)}
              ${envRow("Store present", health.store_present ? "yes" : "no")}
              ${envRow("Flows", `${flows.length} catalogued`)}
              ${envRow("Last refresh", app.state.lastUpdated ? relativeTime(app.state.lastUpdated) : "—")}
            </div>
          </section>
        </div>
      </div>`;
  },

  mount(app, root) {
    hydrateSampleWriteGuards(app, root);
  },
};

function hydrateSampleWriteGuards(app, root) {
  root.querySelectorAll("[data-sample-write]").forEach((element) => {
    element.addEventListener("click", (event) => {
      event.preventDefault();
      toastWarn(
        "Sample mode: writes disabled",
        "This labelled demo dataset is display-only. Start `xdesign web` against a real store to create wishes.",
      );
    });
  });
}

function signature(wishes) {
  return (wishes || []).map((wish) => `${wish.wish_id}:${wish.status}:${wish.current_stage}:${wish.head_revision}:${wish.updated_at}`).join("|");
}

/* ------------------------------------------------------------------ */
/* Hero                                                               */
/* ------------------------------------------------------------------ */

function renderHero(app, wishes, flows, agents) {
  const sample = app.state.sampleMode;
  const activeCount = wishes.filter((wish) => wish.status === "active").length;
  const blockedCount = wishes.filter((wish) => wish.status === "blocked").length;
  const gateCount = flows.reduce(
    (total, flow) => total + (flow.checkpoints || []).filter((checkpoint) => checkpoint.kind === "syntactic").length,
    0,
  );
  const availableAgents = agents.filter((agent) => agent.available).length;

  return `<section class="hero">
    <div class="hero__top">
      <div class="hero__copy">
        <span class="hero__kicker"><span class="beacon"><span></span></span>${sample ? "Sample dataset" : "Local operator console"}</span>
        <h1 class="hero__title">Every wish, from intent to <em>retained evidence</em>.</h1>
        <p class="hero__lede">
          WishFlow Studio reads the local <span class="mono">.xdesign</span> store and shows how each wish moves through its gated stages — what the checker verified, who judged the semantic gates, and which evidence was frozen into the plan.
        </p>
        <div class="hero__actions">
          ${
            sample
              ? `<button class="btn btn--primary is-disabled" type="button" data-sample-write title="Writes are disabled in sample mode">${icon("plus")}New wish</button>`
              : `<a class="btn btn--primary" href="#/new">${icon("plus")}New wish</a>`
          }
          <a class="btn" href="#/wishes">${icon("wishes")}Browse wishes</a>
          <a class="btn btn--ghost" href="#/flows">${icon("flows")}Inspect flows</a>
        </div>
      </div>
      <div class="hero__side">
        <div class="hero__meta">${icon("pulse")}<span>active</span><span class="mono">${activeCount}</span></div>
        <div class="hero__meta">${icon("alert-triangle")}<span>blocked</span><span class="mono">${blockedCount}</span></div>
        <div class="hero__meta">${icon("shield")}<span>deterministic gates</span><span class="mono">${gateCount}</span></div>
        <div class="hero__meta">${icon("cpu")}<span>agent runtimes</span><span class="mono">${availableAgents}/${agents.length}</span></div>
      </div>
    </div>
    <div class="hero__art">
      <img src="/lifecycle.svg" alt="Illustration of a wish lifecycle: requirements, design, implementation and verification stages, each with a gate that accepts the stage, ending in a delivered wish with retained evidence." width="1352" height="351" />
    </div>
    <div class="hero__art-caption">
      <span>lifecycle · four gated stages · frozen plan</span>
      <span>${esc(truncate((app.state.health && app.state.health.store_root) || ".xdesign", 56))}</span>
    </div>
  </section>`;
}

/* ------------------------------------------------------------------ */
/* Stats                                                              */
/* ------------------------------------------------------------------ */

function renderStats(wishes) {
  const total = wishes.length;
  const active = wishes.filter((wish) => wish.status === "active").length;
  const blocked = wishes.filter((wish) => wish.status === "blocked").length;
  const delivered = wishes.filter((wish) => wish.status === "delivered").length;
  const received = wishes.filter((wish) => wish.status === "received").length;
  const storeCount = wishes.filter((wish) => wish.source === "store").length;
  const legacyCount = total - storeCount;
  const spark = (tone, seed) => {
    const bars = [0.35, 0.5, 0.42, 0.68, 0.58, 0.8, 0.72].map((height, index) => {
      const h = Math.max(0.18, Math.min(1, height + ((seed + index * 5) % 9) / 40));
      return `<i style="height:${Math.round(h * 100)}%"></i>`;
    });
    return `<span class="stat__spark" aria-hidden="true">${bars.join("")}</span>`;
  };
  return `
    <a class="stat stat--total tone-ink" href="#/wishes" aria-label="${total} wishes in total">
      <span class="stat__head"><span>Total wishes</span><span class="stat__icon">${icon("layers")}</span></span>
      <span class="stat__value">${total}</span>
      <span class="stat__meta">${storeCount} store · ${legacyCount} legacy</span>
    </a>
    <a class="stat tone-cyan" href="#/wishes?status=active" aria-label="${active} active wishes">
      <span class="stat__head"><span>Active</span><span class="stat__icon">${icon("pulse")}</span></span>
      <span class="stat__value">${active}</span>
      <span class="stat__meta">moving through a stage</span>
      ${spark("cyan", active)}
    </a>
    <a class="stat tone-rose" href="#/wishes?status=blocked" aria-label="${blocked} blocked wishes">
      <span class="stat__head"><span>Blocked</span><span class="stat__icon">${icon("alert-triangle")}</span></span>
      <span class="stat__value">${blocked}</span>
      <span class="stat__meta">awaiting a correction</span>
      ${spark("rose", blocked)}
    </a>
    <a class="stat tone-green" href="#/wishes?status=delivered" aria-label="${delivered} delivered wishes">
      <span class="stat__head"><span>Delivered</span><span class="stat__icon">${icon("check-circle")}</span></span>
      <span class="stat__value">${delivered}</span>
      <span class="stat__meta">${received} received · ${delivered} with retained evidence</span>
      ${spark("green", delivered)}
    </a>`;
}

/* ------------------------------------------------------------------ */
/* Flow pipelines                                                     */
/* ------------------------------------------------------------------ */

function pipelineStageState(flow, stageId, wishes) {
  let state = "idle";
  for (const wish of wishes) {
    if (wish.flow_id !== flow.flow_id) continue;
    if (wish.status === "blocked" && wish.current_stage === stageId) return "blocked";
    if (wish.status === "active" && wish.current_stage === stageId) state = "current";
    if (wish.status === "delivered" && state === "idle") state = "done";
  }
  return state;
}

function flowGateState(flow, stageId, wishes) {
  const inFlow = wishes.filter((wish) => wish.flow_id === flow.flow_id);
  if (inFlow.some((wish) => wish.status === "blocked" && wish.current_stage === stageId)) return "blocked";
  if (inFlow.some((wish) => wish.status === "active" && wish.current_stage === stageId)) return "current";
  if (inFlow.length > 0 && inFlow.every((wish) => wish.status === "delivered")) return "passed";
  return "none";
}

function gateLabel(checkpoint) {
  if (checkpoint.kind === "semantic") return `${checkpoint.title} · judgment`;
  const checker = checkpoint.checker;
  if (checker && checker.checker_id) {
    const short = checker.checker_id.replace(/^simple/, "").replace(/Check$/, "");
    return `${short.toLowerCase()} gate`;
  }
  return `${checkpoint.checkpoint_id} gate`;
}

function renderPipeline(app, wishes) {
  const flows = app.state.flows || [];
  if (flows.length === 0) {
    return `<div class="pipeline__empty">No flows are catalogued by the local server.</div>`;
  }
  return flows
    .map((flow) => {
      const inFlow = wishes.filter((wish) => wish.flow_id === flow.flow_id);
      const activeInFlow = inFlow.filter((wish) => wish.status === "active" || wish.status === "blocked");
      const stages = flow.stages || [];
      const stageMarkup = stages
        .map((stage, index) => {
          const state = pipelineStageState(flow, stage.stage_id, wishes);
          const checkpoints = (flow.checkpoints || []).filter((checkpoint) => checkpoint.stage_id === stage.stage_id);
          const gateState = flowGateState(flow, stage.stage_id, wishes);
          const gateMarkup = checkpoints
            .map(
              (checkpoint) => `<span class="pipeline__gate is-${gateState === "none" ? "idle" : gateState}">
                <span class="gate-mark"></span>${esc(gateLabel(checkpoint))}
              </span>`,
            )
            .join("");
          return `<div class="pipeline__stage" data-state="${state}">
            <div class="pipeline__stage-top">
              <span class="pipeline__num">${String(index + 1).padStart(2, "0")}</span>
              <span class="pipeline__num">${state === "blocked" ? "blocked" : state === "current" ? "now" : state === "done" ? "done" : "queued"}</span>
            </div>
            <div class="pipeline__stage-title">${esc(stage.title)}</div>
            ${gateMarkup}
          </div>`;
        })
        .join("");
      const statusMix = ["blocked", "active", "received", "delivered"]
        .map((status) => {
          const count = inFlow.filter((wish) => wish.status === status).length;
          if (!count) return "";
          const meta = statusMeta(status);
          return `<span class="chip chip--status tone-${meta.tone} chip--sm"><span class="dot"></span>${count} ${meta.label}</span>`;
        })
        .join("");
      return `<div class="pipeline__flow">
        <div class="pipeline__head">
          <div class="pipeline__name">
            ${icon("layers")}
            <span>${esc(FLOW_LABEL[flow.flow_id] || flow.title || flow.flow_id)}</span>
            <span class="mono">${esc(flow.flow_id)}</span>
          </div>
          <div class="pipeline__stats">
            ${statusMix || `<span class="chip chip--sm">${inFlow.length} wish${inFlow.length === 1 ? "" : "es"}</span>`}
            <span class="chip chip--mono chip--sm" title="active or blocked wishes in this flow">${activeInFlow.length} in progress</span>
          </div>
        </div>
        <div class="pipeline__track">${stageMarkup}</div>
      </div>`;
    })
    .join("");
}

/* ------------------------------------------------------------------ */
/* Recent wishes                                                      */
/* ------------------------------------------------------------------ */

function renderRecent(app, wishes) {
  const recent = wishes.slice(0, 6);
  if (recent.length === 0) {
    return emptyState({
      title: "No wishes yet",
      text: "Wishes appear here as soon as the store contains a wish item. Create one from the wizard or with <span class=\"mono\">xdesign init</span>.",
      actions: `<a class="btn btn--primary btn--sm" href="#/new">${icon("plus")}Create the first wish</a>`,
      small: true,
    });
  }
  return recent.map((wish) => wishRow(wish, app)).join("");
}

/* ------------------------------------------------------------------ */
/* Agents                                                             */
/* ------------------------------------------------------------------ */

function renderAgents(agents) {
  if (!agents || agents.length === 0) {
    return `<div class="text-faint" style="font-size:11.5px">No agent catalog reported by the local server.</div>`;
  }
  return agents
    .map((agent) => {
      const available = Boolean(agent.available);
      const models = agent.models || [];
      const shown = models.slice(0, 3);
      return `<div class="agent-card tone-${available ? "green" : "rose"}">
        <span class="agent-card__icon">${icon("terminal")}</span>
        <div class="agent-card__body">
          <div class="agent-card__head">
            <span class="agent-card__name">${esc(agent.label || agent.cli)}</span>
            <span class="chip ${available ? "chip--green" : "chip--rose"} chip--sm"><span class="dot"></span>${available ? "available" : "not found"}</span>
          </div>
          <div class="agent-card__version">${esc(available ? agent.version || "version unknown" : "not detected on this machine")}</div>
          <div class="agent-card__models">
            ${shown.map((model) => `<span class="chip chip--mono">${esc(model.label || model.id)}</span>`).join("")}
            ${models.length > shown.length ? `<span class="chip chip--mono">+${models.length - shown.length} more</span>` : ""}
            <span class="chip chip--mono" title="Default model when a binding is created">default · ${esc(agent.default_model || "—")}</span>
          </div>
        </div>
      </div>`;
    })
    .join("");
}

function envRow(label, value, mono = false) {
  return `<div class="summary-row">
    <span class="summary-row__key">${esc(label)}</span>
    <span class="summary-row__value${mono ? " mono" : ""}" title="${esc(value)}">${esc(value)}</span>
  </div>`;
}
