// xdesign · WishFlow Studio — Flows catalog: stages, checkpoints, checker
// rules, command URIs and custom-check availability for every flow.

import { esc, icon, truncate } from "../util.js";

export const flowsView = {
  title: "Flows",
  poll: false,

  crumb() {
    return "Flows";
  },

  skeleton() {
    return `<div class="skeleton skeleton--line" style="width:240px;height:28px"></div>
      <div class="catalog" style="margin-top:18px">
        <div class="skeleton skeleton--panel" style="height:560px;margin-top:0"></div>
        <div class="skeleton skeleton--panel" style="height:560px;margin-top:0"></div>
      </div>`;
  },

  async load(app) {
    if (!app.state.flows.length) await app.loadMeta();
    await app.loadCheckpoints();
    return { flows: app.state.flows, checkpoints: app.state.checkpoints };
  },

  render(app, data) {
    const flows = (data && data.flows) || app.state.flows;
    const checkpoints = (data && data.checkpoints) || app.state.checkpoints || [];
    const stageCount = flows.reduce((total, flow) => total + (flow.stages || []).length, 0);
    const checkpointCount = flows.reduce((total, flow) => total + (flow.checkpoints || []).length, 0);
    const ruleCount = flows.reduce(
      (total, flow) =>
        total + (flow.checkpoints || []).reduce((inner, checkpoint) => inner + ((checkpoint.checker && checkpoint.checker.rules) || []).length, 0),
      0,
    );

    return `<div class="page-head animate-rise">
      <div class="page-head__titles">
        <span class="eyebrow">Flow catalog</span>
        <h1 class="h1">Flows and their gates</h1>
        <p class="lede">Each flow is a frozen sequence of stages with declared exit checkpoints. Syntactic checkpoints run a deterministic checker by default; semantic checkpoints are judged and recorded with an explicit authority.</p>
      </div>
      <div class="page-head__actions">
        <a class="btn btn--ghost" href="#/wishes">${icon("wishes")}Wishes</a>
        <a class="btn btn--primary" href="#/new">${icon("plus")}New wish</a>
      </div>
    </div>

    <div class="stat-grid animate-rise is-delay-1" style="grid-template-columns:repeat(4,minmax(0,1fr))">
      ${catalogStat("Flows", flows.length, "layers", "ink", "catalogued by the local server")}
      ${catalogStat("Stages", stageCount, "git-commit", "cyan", "across every flow")}
      ${catalogStat("Checkpoints", checkpointCount, "diamond", "violet", "exit gates in total")}
      ${catalogStat("Checker rules", ruleCount, "shield", "green", "declared, deterministic conditions")}
    </div>

    <div class="catalog animate-rise is-delay-2" style="margin-top:18px">
      ${flows.map((flow) => renderFlowCard(flow, checkpoints)).join("")}
    </div>

    ${renderBindingModes()}`;
  },

  mount() {
    return null;
  },
};

function catalogStat(label, value, iconName, tone, meta) {
  return `<div class="stat tone-${tone}">
    <span class="stat__head"><span>${esc(label)}</span><span class="stat__icon">${icon(iconName)}</span></span>
    <span class="stat__value">${value}</span>
    <span class="stat__meta">${esc(meta)}</span>
  </div>`;
}

function renderFlowCard(flow, libraryEntries) {
  const stages = flow.stages || [];
  const checkpoints = flow.checkpoints || [];
  const checkerCount = checkpoints.filter((checkpoint) => checkpoint.checker).length;
  return `<article class="catalog-card">
    <div class="catalog-card__head">
      <div class="catalog-card__titles">
        <div>
          <div class="catalog-card__name">${esc(flow.title || flow.flow_id)}</div>
          <div class="catalog-card__id">${esc(flow.flow_id)}</div>
        </div>
        <span class="chip ${flow.custom_checks_supported ? "chip--violet" : "chip--sm"}">
          ${flow.custom_checks_supported ? `${icon("library")}library bindable` : `${icon("sparkle")}semantic only`}
        </span>
      </div>
      <p class="catalog-card__summary">${esc(flow.summary || "")}</p>
      <div class="catalog-card__badges">
        <span class="chip chip--mono chip--sm">${stages.length} stages</span>
        <span class="chip chip--mono chip--sm">${checkpoints.length} checkpoints</span>
        <span class="chip chip--mono chip--sm">${checkerCount} checker${checkerCount === 1 ? "" : "s"}</span>
        <span class="chip chip--mono chip--sm">${checkpoints.length - checkerCount} judged</span>
        ${stages[stages.length - 1]?.completes_flow || stages.length ? `<span class="chip chip--sm">${icon("flag")}terminal stage ${esc(stages[stages.length - 1]?.title || "—")}</span>` : ""}
      </div>
    </div>

    <div class="catalog-section">
      <div class="catalog-section__title"><span>Stages</span><span class="mono">ordered · frozen at init</span></div>
      ${stages
        .map(
          (stage, index) => `<div class="catalog-stage">
            <span class="catalog-stage__num">${String(index + 1).padStart(2, "0")}</span>
            <div class="catalog-stage__body">
              <div class="catalog-stage__name">${esc(stage.title)}<span class="mono">${esc(stage.stage_id)}</span></div>
              <div class="catalog-stage__goal">${esc(stage.goal || "")}</div>
              ${stage.instructions ? `<div class="catalog-stage__goal" style="color:var(--faint)">${esc(truncate(stage.instructions, 220))}</div>` : ""}
              ${(stage.expected_outputs || []).length
                ? `<div class="catalog-stage__outputs">${stage.expected_outputs.map((output) => `<span class="chip chip--mono chip--sm">${esc(output)}</span>`).join("")}</div>`
                : ""}
            </div>
          </div>`,
        )
        .join("")}
    </div>

    <div class="catalog-section">
      <div class="catalog-section__title"><span>Checkpoints</span><span class="mono">exit gates</span></div>
      ${checkpoints.map((checkpoint) => renderCheckpoint(flow, checkpoint)).join("")}
    </div>

    <div class="catalog-section">
      <div class="catalog-section__title"><span>Library availability</span><span class="mono">checkpoints in this project</span></div>
      ${
        flow.custom_checks_supported
          ? (() => {
              const syntacticEntries = (libraryEntries || []).filter((entry) => entry.kind === "syntactic" && entry.executor?.mode !== "cli");
              const semanticEntries = (libraryEntries || []).filter((entry) => entry.kind === "semantic");
              return `<div class="check-note">${icon("library")}<span><strong>Bindable check executors.</strong> Each syntactic checkpoint can bind the <span class="mono">built-in script</span> or a frozen <span class="mono">library checkpoint</span>. The semantic verification gate can bind a semantic library entry. The chosen executor is frozen into the plan.</span></div>
                <div class="catalog-library">
                  <div class="catalog-library__row">
                    <span class="catalog-library__label">${icon("shield")}Syntactic entries</span>
                    <span class="catalog-library__list">${
                      syntacticEntries.length
                        ? syntacticEntries.map((entry) => `<a class="chip chip--sm chip--cyan" href="#/checkpoints?open=${encodeURIComponent(entry.checkpoint_id)}">${esc(entry.title)} · <span class="mono">${esc(entry.checkpoint_id)}</span></a>`).join("")
                        : '<span class="text-faint" style="font-size:11px">No user entries; the built-in plan checkers remain available as built-in script bindings.</span>'
                    }</span>
                  </div>
                  <div class="catalog-library__row">
                    <span class="catalog-library__label">${icon("sparkle")}Semantic entries</span>
                    <span class="catalog-library__list">${
                      semanticEntries.length
                        ? semanticEntries.map((entry) => `<a class="chip chip--sm chip--violet" href="#/checkpoints?open=${encodeURIComponent(entry.checkpoint_id)}">${esc(entry.title)} · <span class="mono">${esc(entry.checkpoint_id)}</span></a>`).join("")
                        : '<span class="text-faint" style="font-size:11px">No semantic library entries yet.</span>'
                    }</span>
                  </div>
                </div>`;
            })()
          : `<div class="check-note">${icon("info")}<span><strong>Not bindable in this slice.</strong> WishVerifyFlow keeps its two semantic judgments; non-default check bindings are rejected for it. The gates are judged by an agent or human and recorded with that authority.</span></div>`
      }
    </div>
  </article>`;
}

function renderCheckpoint(flow, checkpoint) {
  const checker = checkpoint.checker;
  const semantic = checkpoint.kind === "semantic";
  return `<div class="checkpoint-block">
    <div class="checkpoint-block__head">
      <span class="checkpoint-block__name">${esc(checkpoint.title || checkpoint.checkpoint_id)}</span>
      <span class="checkpoint-block__stage">${esc(checkpoint.stage_id)}</span>
      <span class="checkpoint-row__kind ${semantic ? "is-semantic" : ""}">${esc(checkpoint.kind)}</span>
      ${semantic
        ? `<span class="chip chip--violet chip--sm">${icon("sparkle")}judged${checkpoint.agent_may_judge ? " · agent" : ""}${checkpoint.human_may_judge ? " · human" : ""}</span>`
        : `<span class="chip chip--green chip--sm">${icon("shield")}deterministic default</span>`}
    </div>
    <ul class="checkpoint-block__criteria">
      ${(checkpoint.criteria || []).map((criterion) => `<li>${esc(criterion)}</li>`).join("")}
    </ul>
    ${
      checker
        ? `<div class="checkpoint-block__checker">
            <div class="checkpoint-block__checker-row">
              ${icon("terminal")}<span>${esc(checker.command_uri || "built-in checker")}</span>
              <span class="sep-dot"></span><span>${esc(checker.establishes?.join(", ") || "no declared establishes")}</span>
              <span class="sep-dot"></span><span>modes ${esc((checker.modes || ["default"]).join(" / "))}</span>
            </div>
            <div class="rules">
              ${((checker.rules) || [])
                .map(
                  (rule) => `<div class="rule">
                    <div class="rule__id">${icon("diamond")}${esc(rule.rule_id)}</div>
                    <div class="rule__condition">${esc(rule.condition)}</div>
                    ${(rule.reads || []).length ? `<div class="rule__reads">${rule.reads.map((uri) => `<span class="mono">${esc(uri)}</span>`).join("")}</div>` : ""}
                  </div>`,
                )
                .join("")}
            </div>
          </div>`
        : `<div class="checkpoint-block__checker" style="border-style:dashed">
            <div class="checkpoint-block__checker-row">${icon("sparkle")}<span>No checker command: this checkpoint is judged. The assessment records authority, assessor, outcome and rationale, with evidence URIs.</span></div>
          </div>`
    }
  </div>`;
}

function renderBindingModes() {
  const modes = [
    {
      tone: "green",
      iconName: "shield",
      name: "Built-in script",
      tag: "script authority",
      text: "Runs the repository's deterministic checker inside xdesign. The plan bytes match the default plan. The assessment is recorded with script authority and the checker id as assessor.",
    },
    {
      tone: "cyan",
      iconName: "library",
      name: "Library checkpoint",
      tag: "frozen definition",
      text: "Freezes a project library entry's rules/criteria, inputs and declared executor into the plan. Later library edits never change an existing run; syntactic script entries keep script authority.",
    },
  ];
  return `<section class="panel" style="margin-top:18px">
    <div class="panel__head">
      <div class="panel__title">${icon("sliders")}How check bindings are frozen</div>
      <div class="panel__subtitle">one plan, one reconstructable executor</div>
    </div>
    <div class="panel__body" style="display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:14px" data-binding-modes>
      ${modes
        .map(
          (mode) => `<div class="card" style="padding:14px 15px;border-color:var(--tone-line)">
            <div class="row" style="gap:8px;margin-bottom:8px">
              <span class="agent-card__icon tone-${mode.tone}" style="width:28px;height:28px">${icon(mode.iconName)}</span>
              <span class="mono" style="font-size:11.5px;color:var(--ink-soft)">${mode.name}</span>
              <span class="chip chip--sm tone-${mode.tone}">${mode.tag}</span>
            </div>
            <div style="color:var(--muted);font-size:11.5px;line-height:1.6">${mode.text}</div>
          </div>`,
        )
        .join("")}
    </div>
    <div class="side-panel__note" style="padding:0 20px 18px">
      Stage work is always performed by the working agent; the check executor selects only who verifies the checkpoint. A non-default binding changes the frozen plan, and <span class="mono">xdesign next</span>, <span class="mono">xdesign check</span> and this page reconstruct the executor from the store without a separate configuration file.
    </div>
  </section>`;
}
