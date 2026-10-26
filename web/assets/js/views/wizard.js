// xdesign · WishFlow Studio — New wish wizard.
//
// Steps: Identity → Flow → Checks → Review. For SimpleFlow each syntactic gate
// can bind either the deterministic default checker or an agent CLI execution;
// the chosen bindings are frozen into the plan by `xdesign init`.

import { api, ApiError } from "../api.js";
import { esc, icon, suggestWishId, flowChip, parseHash, relativeTime } from "../util.js";
import { toastOk, toastError, toastWarn } from "../components.js";

const STEPS = [
  { key: "identity", label: "Identity", title: "Name the wish", sub: "A stable wish id, a short title and the intent behind it." },
  { key: "flow", label: "Flow", title: "Choose a flow", sub: "The flow decides the stages, the gates and the checks that run." },
  { key: "checks", label: "Checks", title: "Bind the checks", sub: "Stage work is always performed by the working agent; each binding selects only who verifies the checkpoint." },
  { key: "review", label: "Review", title: "Review and create", sub: "Confirm the wish and the frozen per-node bindings before creating the plan." },
];

const WISH_ID_RE = /^[A-Za-z0-9][A-Za-z0-9_-]{0,127}$/;
const KNOWN_SIMPLE_CHECKPOINTS = [
  { checkpoint_id: "simpleRequirementsAccepted", title: "simpleRequirementsCheck", stage: "requirements" },
  { checkpoint_id: "simpleDesignAccepted", title: "simpleDesignCheck", stage: "design" },
  { checkpoint_id: "simpleImplementationAccepted", title: "simpleImplementationCheck", stage: "implement" },
  { checkpoint_id: "simpleRecordAccepted", title: "simpleRecordCheck", stage: "verify" },
];

let draft = null;

function flowFor(app, flowId) {
  return (app.state.flows || []).find((flow) => flow.flow_id === flowId) || null;
}

function agentFor(app, cli) {
  return (app.state.agents || []).find((agent) => agent.cli === cli) || null;
}

function checkNodeInfo(app, flow) {
  const syntactic = (flow && flow.checkpoints ? flow.checkpoints : []).filter((checkpoint) => checkpoint.kind === "syntactic");
  if (syntactic.length > 0) {
    const ordered = [];
    for (const stage of flow.stages || []) {
      for (const checkpoint of syntactic) {
        if (checkpoint.stage_id === stage.stage_id) ordered.push(checkpoint);
      }
    }
    return ordered.map((checkpoint) => ({
      checkpoint_id: checkpoint.checkpoint_id,
      title: checkpoint.title,
      stage_id: checkpoint.stage_id,
      checker: checkpoint.checker,
    }));
  }
  return KNOWN_SIMPLE_CHECKPOINTS.map((checkpoint) => ({
    checkpoint_id: checkpoint.checkpoint_id,
    title: checkpoint.title,
    stage_id: checkpoint.stage,
    checker: { checker_id: checkpoint.title, command_uri: "", rules: [], establishes: [] },
  }));
}

function defaultCli(app) {
  const agents = app.state.agents || [];
  const available = agents.find((agent) => agent.available);
  return (available || agents[0] || {}).cli || "codex";
}

function defaultModelFor(app, cli) {
  const agent = agentFor(app, cli);
  return (agent && agent.default_model) || (agent && agent.models && agent.models[0] && agent.models[0].id) || "";
}

function ensureDraft(app) {
  const flow = flowFor(app, "simple-flow") || { flow_id: "simple-flow", stages: [], checkpoints: [] };
  if (!draft) {
    const cli = defaultCli(app);
    const checks = {};
    for (const node of checkNodeInfo(app, flow)) {
      checks[node.checkpoint_id] = {
        checkpoint_id: node.checkpoint_id,
        mode: "default",
        cli,
        model: defaultModelFor(app, cli),
        modelOverride: "",
      };
    }
    draft = {
      step: 0,
      wishId: suggestWishId(),
      title: "",
      description: "",
      flowId: "simple-flow",
      checks,
      errors: {},
      createError: null,
      submitting: false,
      touched: {},
    };
  }
  // Keep check entries in sync if the catalog changed.
  const nodes = checkNodeInfo(app, flow);
  for (const node of nodes) {
    if (!draft.checks[node.checkpoint_id]) {
      const cli = defaultCli(app);
      draft.checks[node.checkpoint_id] = {
        checkpoint_id: node.checkpoint_id,
        mode: "default",
        cli,
        model: defaultModelFor(app, cli),
        modelOverride: "",
        libraryId: "",
      };
    }
  }
  return draft;
}

export function resetWizardDraft() {
  draft = null;
}

/* ------------------------------------------------------------------ */
/* Validation                                                         */
/* ------------------------------------------------------------------ */

function validateIdentity(current) {
  const errors = {};
  const wishId = current.wishId.trim();
  const title = current.title.trim();
  if (!wishId) errors.wishId = "A wish id is required.";
  else if (!WISH_ID_RE.test(wishId)) errors.wishId = "Use 1–128 letters, digits, '-' or '_', starting with a letter or digit.";
  if (!title) errors.title = "A title is required.";
  else if (title.length > 200) errors.title = `The title is ${title.length} characters; the limit is 200.`;
  if ((current.description || "").length > 4000) errors.description = `The description is ${current.description.length} characters; the limit is 4000.`;
  return errors;
}

function libraryEntries(app, kind) {
  return (app.state.checkpoints || []).filter((entry) => !kind || entry.kind === kind);
}

function findLibraryEntry(app, libraryId) {
  if (!libraryId) return null;
  return (app.state.checkpoints || []).find((entry) => entry.checkpoint_id === libraryId) || null;
}

function selectedLibraryDetail(app, libraryId) {
  return (app.state.checkpointDetailById && app.state.checkpointDetailById.get(libraryId)) || null;
}

function renderCheckCard(app, current, flow, node, index, agents, { libraryError = null } = {}) {
  const binding = current.checks[node.checkpoint_id] || {
    mode: "default",
    cli: defaultCli(app),
    model: defaultModelFor(app, defaultCli(app)),
    modelOverride: "",
    libraryId: "",
  };
  const mode = binding.mode || "default";
  const isCli = mode === "cli";
  const isLibrary = mode === "library";
  const agent = agentFor(app, binding.cli);
  const available = Boolean(agent && agent.available);
  const checker = node.checker || {};
  const ruleCount = (checker.rules || []).length;
  const stage = (flow.stages || []).find((entry) => entry.stage_id === node.stage_id);
  const models = (agent && agent.models) || [];
  const effectiveModel = (binding.modelOverride || "").trim() || binding.model;
  const libraryChoices = libraryEntries(app, "syntactic");
  const entry = findLibraryEntry(app, binding.libraryId);
  const entryDetail = selectedLibraryDetail(app, binding.libraryId);
  const entryRules = entryDetail && entryDetail.rules ? entryDetail.rules : [];
  const previewDefinition = entryDetail || entry;
  const effectiveRuleCount = isLibrary ? (previewDefinition ? previewDefinition.rule_count ?? (entryRules || []).length : 0) : ruleCount;
  const error = current.errors[node.checkpoint_id];
  const modeChip = isCli
    ? `<span class="chip chip--violet chip--sm">${icon("cpu")}Agent CLI</span>`
    : isLibrary
      ? `<span class="chip chip--cyan chip--sm">${icon("library")}Library checkpoint</span>`
      : `<span class="chip chip--green chip--sm">${icon("shield")}Built-in script</span>`;

  const executorCell = `<div class="check-card__control">
      <span class="check-card__control-label">Check executor</span>
      <div class="segmented" role="group" aria-label="Check executor for ${esc(node.checkpoint_id)}">
        <button class="segmented__opt ${mode === "default" ? "is-active" : ""}" type="button" data-check-mode="${esc(node.checkpoint_id)}" data-mode="default" aria-pressed="${mode === "default"}">${icon("shield")}Built-in script</button>
        <button class="segmented__opt ${isCli ? "is-active" : ""}" type="button" data-check-mode="${esc(node.checkpoint_id)}" data-mode="cli" aria-pressed="${isCli}">${icon("cpu")}Agent CLI</button>
        <button class="segmented__opt ${isLibrary ? "is-active" : ""}" type="button" data-check-mode="${esc(node.checkpoint_id)}" data-mode="library" aria-pressed="${isLibrary}" ${libraryError ? "disabled" : ""}>${icon("library")}Library</button>
      </div>
      <span class="check-card__hint">${mode === "default" ? "Repository checker · script authority." : isCli ? "Separate CLI run · agent authority." : "Frozen library rules and executor."}</span>
    </div>`;

  const primaryCell = isCli
    ? `<div class="check-card__control">
        <span class="check-card__control-label">CLI</span>
        <select class="select" data-check-cli="${esc(node.checkpoint_id)}" aria-label="Agent CLI for ${esc(node.checkpoint_id)}">
          ${agents.map((entry2) => `<option value="${esc(entry2.cli)}" ${entry2.cli === binding.cli ? "selected" : ""}>${esc(entry2.cli)}${entry2.available ? "" : " · off"}</option>`).join("")}
        </select>
        <span class="check-card__hint">${available ? esc(agent.version || "detected") : "not detected on this machine"}</span>
      </div>`
    : isLibrary
      ? `<div class="check-card__control">
          <span class="check-card__control-label">Library checkpoint</span>
          <select class="select" data-check-library="${esc(node.checkpoint_id)}" aria-label="Library checkpoint for ${esc(node.checkpoint_id)}" ${libraryError ? "disabled" : ""}>
            <option value="">Select a syntactic entry…</option>
            ${libraryChoices
              .map((candidate) => {
                const rules = candidate.rule_count ?? 0;
                const executor = candidate.executor
                  ? candidate.executor.mode === "cli"
                    ? `agent CLI ${candidate.executor.cli}:${candidate.executor.model}`
                    : `script ${candidate.executor.command_uri}`
                  : "judged";
                return `<option value="${esc(candidate.checkpoint_id)}" ${candidate.checkpoint_id === binding.libraryId ? "selected" : ""}>${esc(candidate.title)} · ${esc(candidate.checkpoint_id)} · ${rules} rule${rules === 1 ? "" : "s"} · ${esc(executor)}</option>`;
              })
              .join("")}
          </select>
          <span class="check-card__hint">${libraryChoices.length} syntactic entr${libraryChoices.length === 1 ? "y" : "ies"} available</span>
        </div>`
      : `<div class="check-card__control">
          <span class="check-card__control-label">Built-in checker</span>
          <div class="check-card__static" title="${esc(checker.command_uri || "")}">${icon("terminal")}${esc(checker.checker_id || node.title)}</div>
          <span class="check-card__hint">${esc(truncateMiddle(checker.command_uri || "built-in checker", 34))}</span>
        </div>`;

  const secondaryCell = isCli
    ? `<div class="check-card__control">
        <span class="check-card__control-label">Model</span>
        <select class="select" data-check-model="${esc(node.checkpoint_id)}" aria-label="Model for ${esc(node.checkpoint_id)}">
          ${models.map((model) => `<option value="${esc(model.id)}" ${model.id === binding.model ? "selected" : ""}>${esc(model.label || model.id)}</option>`).join("")}
          ${models.length === 0 ? `<option value="${esc(binding.model)}" selected>${esc(binding.model || "default")}</option>` : ""}
        </select>
        <span class="check-card__hint">from <span class="mono">/api/flows</span></span>
      </div>`
    : isLibrary
      ? `<div class="check-card__control">
          <span class="check-card__control-label">Entry executor</span>
          <div class="check-card__static" title="${esc(entry ? (entry.executor && entry.executor.mode === "cli" ? `${entry.executor.cli}:${entry.executor.model}` : (entry.executor && entry.executor.command_uri) || "judged") : "no entry selected")}">
            ${icon("terminal")}${esc(entry ? (entry.executor && entry.executor.mode === "cli" ? `${entry.executor.cli}:${entry.executor.model}` : (entry.executor && entry.executor.command_uri ? entry.executor.command_uri.split("/").pop() : "judged")) : "no entry selected")}
          </div>
          <span class="check-card__hint" title="${esc(entry && entry.executor && entry.executor.command_uri ? entry.executor.command_uri : "")}">${entry ? (entry.executor && entry.executor.mode === "cli" ? "agent execution" : "script execution") : "choose an entry"} · ${entry ? (entry.builtin ? "built-in" : "user") : ""}</span>
        </div>`
      : `<div class="check-card__control">
          <span class="check-card__control-label">Authority</span>
          <div class="check-card__static">${icon("shield")}script authority</div>
          <span class="check-card__hint">recorded assessor ${esc(checker.checker_id || node.title)}</span>
        </div>`;

  const tertiaryCell = isCli
    ? `<div class="check-card__control">
        <span class="check-card__control-label">Model override</span>
        <input class="input input--mono" data-check-model-override="${esc(node.checkpoint_id)}" value="${esc(binding.modelOverride || "")}" placeholder="optional model id" spellcheck="false" autocomplete="off" aria-label="Free-text model override for ${esc(node.checkpoint_id)}" />
        <span class="check-card__hint">effective <span class="mono" data-effective-model="${esc(node.checkpoint_id)}">${esc(effectiveModel || "—")}</span></span>
      </div>`
    : isLibrary
      ? `<div class="check-card__control">
          <span class="check-card__control-label">Version</span>
          <div class="check-card__static">${icon("git-commit")}${entry ? `v${esc(entry.version)} · ${effectiveRuleCount} rule${effectiveRuleCount === 1 ? "" : "s"}` : "—"}</div>
          <span class="check-card__hint">${entry ? `updated ${esc(relativeTime(entry.updated_at))}` : "select an entry to preview rules"}</span>
        </div>`
      : `<div class="check-card__control">
          <span class="check-card__control-label">Declared rules</span>
          <div class="check-card__static">${icon("diamond")}${ruleCount} rule${ruleCount === 1 ? "" : "s"}</div>
          <span class="check-card__hint">${esc((checker.establishes || []).slice(0, 2).join(", ") || "deterministic checker")}</span>
        </div>`;

  const preview = isLibrary
    ? `<div class="check-card__preview">
        <div class="check-card__preview-head"><span>${icon("library")}rules frozen into the plan</span>${entry ? `<span class="check-card__preview-title">${esc(entry.title)}</span>` : ""}</div>
        ${
          !binding.libraryId
            ? '<span class="check-card__preview-empty">Select a library checkpoint to preview its rules.</span>'
            : entryRules.length
              ? entryRules.map((rule) => `<div class="check-card__preview-rule"><span class="mono">${esc(rule.rule_id)}</span><span>${esc(rule.condition)}</span></div>`).join("")
              : '<span class="check-card__preview-empty">Loading library rules…</span>'
        }
      </div>`
    : "";

  return `<div class="check-card ${isCli ? "is-cli" : ""} ${isLibrary ? "is-library" : ""}" data-check-card="${esc(node.checkpoint_id)}" data-executor="${esc(mode)}">
    <div class="check-card__head">
      <span class="check-card__order">${String(index + 1).padStart(2, "0")}</span>
      <div class="check-card__titles">
        <div class="check-card__title">
          <span class="check-card__name">${esc(isLibrary ? entry ? entry.title : "Library checkpoint" : checker.checker_id || node.title)}</span>
          <span class="check-card__id">${esc(node.checkpoint_id)}</span>
          <span class="chip chip--sm">${esc(stage ? stage.title : node.stage_id)}</span>
          ${modeChip}
        </div>
        <div class="check-card__meta">
          <span>${effectiveRuleCount} rule${effectiveRuleCount === 1 ? "" : "s"}</span>
          <span class="sep-dot"></span>
          <span class="mono">${esc(isCli ? "agent CLI execution" : isLibrary ? (entry ? entry.checkpoint_id : "library entry pending") : checker.command_uri || "built-in checker")}</span>
        </div>
      </div>
    </div>
    <div class="check-card__body">
      ${executorCell}
      ${primaryCell}
      ${secondaryCell}
      ${tertiaryCell}
      ${preview}
    </div>
    <div class="field__error" data-check-error="${esc(node.checkpoint_id)}" style="padding:0 15px 12px 53px" ${error ? "" : "hidden"}>${error ? `${icon("alert-circle")}${esc(error)}` : ""}</div>
  </div>`;
}

function renderSemanticCard(app, current, flow, { libraryError = null } = {}) {
  const gate = (flow.checkpoints || []).find((checkpoint) => checkpoint.checkpoint_id === "verificationAccepted");
  if (!gate) return "";
  const state = current.semantic || { mode: "default", libraryId: "" };
  const isLibrary = state.mode === "library";
  const choices = libraryEntries(app, "semantic");
  const entry = findLibraryEntry(app, state.libraryId);
  const detail = selectedLibraryDetail(app, state.libraryId);
  const criteria = isLibrary ? (detail && detail.criteria) || [] : gate.criteria || [];
  const error = current.errors.verificationAccepted;
  const judges = isLibrary && entry ? "library-defined" : "agent + human";
  return `<div class="semantic-card" data-semantic-card>
    <div class="semantic-card__head">
      <span class="semantic-card__order">${icon("sparkle")}</span>
      <div class="check-card__titles">
        <div class="check-card__title">
          <span class="check-card__name">verificationAccepted</span>
          <span class="check-card__id">SimpleFlow · semantic gate</span>
          <span class="chip chip--violet chip--sm">${icon("sparkle")}semantic judgment</span>
          <span class="chip chip--sm">${esc(gate.stage_id)}</span>
          ${isLibrary ? `<span class="chip chip--cyan chip--sm">${icon("library")}Library checkpoint</span>` : `<span class="chip chip--green chip--sm">${icon("shield")}Default criteria</span>`}
        </div>
        <div class="check-card__meta">
          <span>${criteria.length} ${isLibrary ? "library" : "flow"} criteria</span>
          <span class="sep-dot"></span>
          <span>${esc(judges)}</span>
          <span class="sep-dot"></span>
          <span class="mono">${entry ? `v${esc(entry.version)} · ${esc(entry.checkpoint_id)}` : "the flow's declared semantic gate"}</span>
        </div>
      </div>
    </div>
    <div class="semantic-card__body">
      <div class="check-card__control">
        <span class="check-card__control-label">Check executor</span>
        <div class="segmented" role="group" aria-label="Check executor for the verification gate">
          <button class="segmented__opt ${!isLibrary ? "is-active" : ""}" type="button" data-semantic-mode="default" aria-pressed="${!isLibrary}">${icon("shield")}Default criteria</button>
          <button class="segmented__opt ${isLibrary ? "is-active" : ""}" type="button" data-semantic-mode="library" aria-pressed="${isLibrary}" ${libraryError ? "disabled" : ""}>${icon("library")}Library</button>
        </div>
        <span class="check-card__hint">${isLibrary ? "A permitted judge applies the frozen semantic criteria." : "A permitted judge applies the flow's declared criteria."}</span>
      </div>
      <div class="check-card__control">
        <span class="check-card__control-label">Semantic checkpoint</span>
        ${
          isLibrary
            ? `<select class="select" data-semantic-library aria-label="Semantic library checkpoint" ${libraryError ? "disabled" : ""}>
                <option value="">Select a semantic entry…</option>
                ${choices.map((candidate) => `<option value="${esc(candidate.checkpoint_id)}" ${candidate.checkpoint_id === state.libraryId ? "selected" : ""}>${esc(candidate.title)} · ${esc(candidate.checkpoint_id)} · ${candidate.criteria_count || 0} criteria</option>`).join("")}
              </select>
              <span class="check-card__hint">${choices.length} semantic entr${choices.length === 1 ? "y" : "ies"} available</span>`
            : `<div class="check-card__static">${icon("shield")}flow criteria</div>
               <span class="check-card__hint">declared by SimpleFlow verification</span>`
        }
      </div>
      <div class="check-card__control">
        <span class="check-card__control-label">Judges</span>
        <div class="check-card__static">${icon("sparkle")}${esc(judges)}</div>
        <span class="check-card__hint">${isLibrary ? "from the frozen library entry" : `agent ${gate.agent_may_judge ? "permitted" : "not permitted"} · human ${gate.human_may_judge ? "permitted" : "not permitted"}`}</span>
      </div>
      <div class="check-card__control">
        <span class="check-card__control-label">Version</span>
        <div class="check-card__static">${icon("git-commit")}${entry ? `v${esc(entry.version)}` : "flow default"}</div>
        <span class="check-card__hint">${isLibrary ? "library frozen into the plan" : "no executor: the gate is judged"}</span>
      </div>
      <div class="semantic-card__criteria">
        <div class="check-card__preview-head">${icon("sparkle")}Criteria frozen into the plan</div>
        ${
          criteria.length
            ? `<ul class="semantic-card__criteria">${criteria.map((criterion) => `<li>${esc(criterion)}</li>`).join("")}</ul>`
            : `<span class="check-card__preview-empty">${isLibrary && state.libraryId ? "Loading library criteria…" : "Select a semantic library checkpoint to preview its criteria."}</span>`
        }
      </div>
    </div>
    <div class="field__error" data-check-error="verificationAccepted" style="padding:0 15px 12px 53px" ${error ? "" : "hidden"}>${error ? `${icon("alert-circle")}${esc(error)}` : ""}</div>
  </div>`;
}

function truncateMiddle(value, max) {
  const text = String(value || "");
  if (text.length <= max) return text;
  const half = Math.floor((max - 1) / 2);
  return `${text.slice(0, half)}…${text.slice(text.length - half)}`;
}

function renderReview(app, current) {
  const flow = flowFor(app, current.flowId);
  const sample = app.state.sampleMode;
  const nodes = flow && flow.flow_id === "simple-flow" ? checkNodeInfo(app, flow) : [];
  const semanticState = current.semantic || { mode: "default", libraryId: "" };
  const semanticEntry = findLibraryEntry(app, semanticState.libraryId);
  return `<div class="review">
    <div class="review__block">
      <div class="review__block-head">
        <span class="review__block-title">Wish</span>
        ${flowChip(current.flowId)}
      </div>
      <div class="review__wish-title">${esc(current.title.trim() || "Untitled wish")}</div>
      <div class="review__wish-id">${esc(current.wishId.trim() || "wish-id-pending")}</div>
      ${current.description.trim() ? `<div class="review__wish-desc">${esc(current.description.trim())}</div>` : '<div class="review__wish-desc text-faint">No description.</div>'}
      <div class="review__meta">
        ${flow ? `<span class="chip chip--mono chip--sm">${(flow.stages || []).length} stages</span>` : ""}
        ${flow ? `<span class="chip chip--mono chip--sm">${(flow.checkpoints || []).length} checkpoints</span>` : ""}
        <span class="chip chip--mono chip--sm">source · store</span>
      </div>
    </div>
    <div class="review__block">
      <div class="review__block-head">
        <span class="review__block-title">Check executors</span>
        <span class="chip chip--sm">${flow && flow.flow_id === "simple-flow" ? "frozen into plan" : "semantic judgments"}</span>
      </div>
      ${
        nodes.length > 0
          ? `<div class="review__bindings">${nodes
              .map((node) => {
                const binding = current.checks[node.checkpoint_id] || { mode: "default" };
                const model = (binding.modelOverride || "").trim() || binding.model;
                const entry = findLibraryEntry(app, binding.libraryId);
                const isCli = binding.mode === "cli";
                const isLibrary = binding.mode === "library";
                return `<div class="review-binding ${isCli ? "is-cli" : ""} ${isLibrary ? "is-library" : ""}">
                  <span class="review-binding__name">${esc(node.checker.checker_id || node.title)}</span>
                  <span class="review-binding__mode">
                    ${
                      isCli
                        ? `${icon("cpu")}<span class="badge chip--violet badge--mono">Agent CLI · ${esc(binding.cli)}:${esc(model)}</span>`
                        : isLibrary
                          ? `${icon("library")}<span class="badge chip--cyan badge--mono">Library · ${esc(entry ? entry.title : binding.libraryId || "not selected")}</span>`
                          : `${icon("shield")}<span class="badge badge--soft badge--mono">Built-in script</span>`
                    }
                  </span>
                </div>`;
              })
              .join("")}
              <div class="review-binding ${semanticState.mode === "library" ? "is-library" : ""}">
                <span class="review-binding__name">verificationAccepted <span class="text-faint">· semantic</span></span>
                <span class="review-binding__mode">
                  ${semanticState.mode === "library" ? `${icon("library")}<span class="badge chip--cyan badge--mono">Library · ${esc(semanticEntry ? semanticEntry.title : semanticState.libraryId || "not selected")}</span>` : `${icon("sparkle")}<span class="badge badge--soft badge--mono">Default criteria</span>`}
                </span>
              </div></div>`
          : `<div class="review__bindings">${(flow ? flow.checkpoints : [])
              .map((checkpoint) => `<div class="review-binding"><span class="review-binding__name">${esc(checkpoint.title)}</span><span class="review-binding__mode">${icon("sparkle")}<span class="badge badge--soft badge--mono">judged at ${esc(checkpoint.stage_id)}</span></span></div>`)
              .join("")}</div>`
      }
    </div>
    <div class="review__notice">${icon("info")}<span><strong>What creation does.</strong> The wizard writes the wish source and, when a binding is not the default, a binding configuration. <span class="mono">xdesign init</span> then creates the store item and freezes the plan — including these check executors — before any stage work begins. <strong>Stage work is never started automatically:</strong> hand the work order to a working agent from the wish detail page.</span></div>
    ${sample ? `<div class="review__notice">${icon("alert-triangle")}<span><strong>Sample mode.</strong> Creation is disabled for the labelled demo dataset. Start <span class="mono">xdesign web</span> against a real store to create this wish.</span></div>` : ""}
    <div class="form-error" data-create-error ${current.createError ? "" : "hidden"}>${icon("alert-circle")}<span>${esc(current.createError || "")}</span></div>
  </div>`;
}

function renderStepFoot(app, current) {
  const isFirst = current.step === 0;
  const isLast = current.step === STEPS.length - 1;
  const sample = app.state.sampleMode;
  return `<div class="wizard-card__foot">
    <div class="row">
      ${isFirst ? "" : `<button class="btn btn--ghost" type="button" data-step="${current.step - 1}">${icon("chevron-left")}Back</button>`}
    </div>
    <div class="row">
      ${isLast ? "" : `<button class="btn btn--primary" type="button" data-next-step>${current.step === 2 ? "Review" : "Continue"}${icon("chevron")}</button>`}
      ${
        isLast
          ? `<button class="btn btn--primary btn--lg ${current.submitting ? "is-busy" : ""}" type="button" data-create-wish ${sample || current.submitting ? "disabled" : ""} title="${sample ? "Writes are disabled in sample mode" : ""}">
              ${current.submitting ? "" : icon("sparkle")}${current.submitting ? "Creating…" : "Create wish"}
            </button>`
          : ""
      }
    </div>
  </div>`;
}

function renderSummary(app, current) {
  const flow = flowFor(app, current.flowId);
  const nodes = flow && flow.flow_id === "simple-flow" ? checkNodeInfo(app, flow) : [];
  const cliCount = nodes.filter((node) => current.checks[node.checkpoint_id] && current.checks[node.checkpoint_id].mode === "cli").length;
  const libraryCount = nodes.filter((node) => current.checks[node.checkpoint_id] && current.checks[node.checkpoint_id].mode === "library").length + (current.semantic && current.semantic.mode === "library" ? 1 : 0);
  return `<div class="card summary-card">
    <div class="summary-card__title">${icon("sparkle")}Draft summary</div>
    <div class="summary-row"><span class="summary-row__key">Wish id</span><span class="summary-row__value mono">${esc(current.wishId || "—")}</span></div>
    <div class="summary-row"><span class="summary-row__key">Title</span><span class="summary-row__value">${esc(current.title || "—")}</span></div>
    <div class="summary-row"><span class="summary-row__key">Flow</span><span class="summary-row__value mono">${esc(current.flowId)}</span></div>
    <div class="summary-row"><span class="summary-row__key">Gates</span><span class="summary-row__value mono">${nodes.length || (flow ? flow.checkpoints.length : 0)}</span></div>
    <div class="summary-row"><span class="summary-row__key">Agent CLI</span><span class="summary-row__value mono">${cliCount} of ${nodes.length || 0}</span></div>
    <div class="summary-row"><span class="summary-row__key">Library</span><span class="summary-row__value mono">${libraryCount} bound</span></div>
    <div class="summary-progress" aria-hidden="true">
      ${STEPS.map((step, index) => `<i class="${index < current.step ? "is-done" : index === current.step ? "is-active" : ""}"></i>`).join("")}
      <span>step ${current.step + 1}/${STEPS.length}</span>
    </div>
  </div>`;
}

function renderWhatHappens(app) {
  return `<div class="card summary-card">
    <div class="summary-card__title">${icon("layers")}What happens on create</div>
    <div class="work-result__list">
      <div style="padding-left:0;color:var(--muted);font-size:11.5px;line-height:1.6">1 — Write the wish source marker and the optional binding configuration.</div>
      <div style="padding-left:0;color:var(--muted);font-size:11.5px;line-height:1.6">2 — Run <span class="mono" style="color:var(--ink-soft)">xdesign init</span> to create the store item at revision 1.</div>
      <div style="padding-left:0;color:var(--muted);font-size:11.5px;line-height:1.6">3 — Freeze the plan, including every check binding, then open the wish detail.</div>
    </div>
  </div>`;
}

/* ------------------------------------------------------------------ */
/* Behaviour                                                          */
/* ------------------------------------------------------------------ */

function focusFirstError(root) {
  const invalid = root.querySelector(".is-invalid");
  if (invalid) {
    invalid.focus({ preventScroll: true });
    invalid.scrollIntoView({ block: "center", behavior: "smooth" });
  }
}

/* ------------------------------------------------------------------ */
/* Validation (check executors)                                       */
/* ------------------------------------------------------------------ */

function validateChecks(app) {
  const errors = {};
  const flow = flowFor(app, draft.flowId);
  if (!flow || flow.flow_id !== "simple-flow") return errors;
  for (const node of checkNodeInfo(app, flow)) {
    const binding = draft.checks[node.checkpoint_id];
    if (!binding) continue;
    if (binding.mode === "cli") {
      if (!["codex", "kimi"].includes(binding.cli)) {
        errors[node.checkpoint_id] = "Choose codex or kimi for an Agent CLI executor.";
        continue;
      }
      const model = (binding.modelOverride || "").trim() || binding.model;
      if (!model) errors[node.checkpoint_id] = "An Agent CLI executor needs a non-empty model id.";
      else if (model.length > 200) errors[node.checkpoint_id] = "The model id must be 200 characters or fewer.";
      continue;
    }
    if (binding.mode === "library") {
      const entry = findLibraryEntry(app, binding.libraryId);
      if (!binding.libraryId) errors[node.checkpoint_id] = "Select a library checkpoint for this gate.";
      else if (!entry) errors[node.checkpoint_id] = `Library checkpoint ${binding.libraryId} is no longer available.`;
      else if (entry.kind !== "syntactic") errors[node.checkpoint_id] = "This gate needs a syntactic library checkpoint.";
    }
  }
  if (draft.semantic && draft.semantic.mode === "library") {
    const entry = findLibraryEntry(app, draft.semantic.libraryId);
    if (!draft.semantic.libraryId) errors.verificationAccepted = "Select a semantic library checkpoint for the verification gate.";
    else if (!entry) errors.verificationAccepted = `Library checkpoint ${draft.semantic.libraryId} is no longer available.`;
    else if (entry.kind !== "semantic") errors.verificationAccepted = "The verification gate needs a semantic library checkpoint.";
  }
  return errors;
}

/* ------------------------------------------------------------------ */
/* View                                                               */
/* ------------------------------------------------------------------ */

export const wizardView = {
  title: "New wish",
  poll: false,

  crumb() {
    return "New wish";
  },

  skeleton() {
    return `<div class="skeleton skeleton--line" style="width:220px;height:28px"></div>
      <div class="skeleton" style="height:90px;margin-top:18px"></div>
      <div class="skeleton skeleton--panel" style="height:460px"></div>`;
  },

  async load(app) {
    if (!app.state.flows.length) await app.loadMeta();
    await app.loadCheckpoints();
    const { params } = parseHash(location.hash);
    const stepParam = Number(params.get("step"));
    if (Number.isInteger(stepParam) && stepParam >= 0 && stepParam < STEPS.length) {
      ensureDraft(app).step = stepParam;
    }
    const current = ensureDraft(app);
    const selected = [
      ...Object.values(current.checks || {})
        .filter((binding) => binding.mode === "library" && binding.libraryId)
        .map((binding) => binding.libraryId),
      ...(current.semantic && current.semantic.mode === "library" && current.semantic.libraryId ? [current.semantic.libraryId] : []),
    ];
    await Promise.all(selected.map((id) => app.loadCheckpointDetail(id)));
    return { flows: app.state.flows };
  },

  render(app) {
    const current = ensureDraft(app);
    return renderWizard(app, current);
  },

  mount(app, root) {
    const current = ensureDraft(app);

    const rerender = () => {
      root.innerHTML = renderWizard(app, current);
      bind();
    };

    const goStep = (step) => {
      if (step < 0 || step >= STEPS.length) return;
      if (step > current.step) {
        // Forward navigation validates the steps being left.
        if (current.step === 0) {
          const errors = validateIdentity(current);
          current.errors = errors;
          if (Object.keys(errors).length > 0) {
            rerender();
            focusFirstError(root);
            return;
          }
        }
        if (current.step === 2) {
          const errors = validateChecks(app);
          current.errors = errors;
          if (Object.keys(errors).length > 0) {
            rerender();
            const firstKey = Object.keys(errors)[0];
            const errorEl = root.querySelector(`[data-check-error="${CSS.escape(firstKey)}"]`);
            if (errorEl) errorEl.scrollIntoView({ block: "center", behavior: "smooth" });
            return;
          }
        }
      }
      current.step = step;
      const url = `#/new${step > 0 ? `?step=${step}` : ""}`;
      if (location.hash !== url) history.replaceState(null, "", url);
      rerender();
    };

    const bind = () => {
      // ---- stepper ----
      root.querySelectorAll("[data-step]").forEach((button) => {
        button.addEventListener("click", () => {
          const target = Number(button.dataset.step);
          if (Number.isNaN(target)) return;
          if (target < current.step) goStep(target);
        });
      });

      // ---- identity inputs ----
      const wishIdInput = root.querySelector("[data-field-wish-id]");
      if (wishIdInput) {
        wishIdInput.addEventListener("input", () => {
          current.wishId = wishIdInput.value;
          current.errors = { ...current.errors, wishId: undefined };
          const errorEl = root.querySelector('[data-error-for="wishId"]');
          if (errorEl) errorEl.innerHTML = "";
          wishIdInput.classList.remove("is-invalid");
        });
      }
      const suggest = root.querySelector("[data-suggest-id]");
      if (suggest) {
        suggest.addEventListener("click", () => {
          current.wishId = suggestWishId();
          rerender();
          const input = root.querySelector("[data-field-wish-id]");
          if (input) input.focus();
        });
      }
      const titleInput = root.querySelector("[data-field-title]");
      if (titleInput) {
        titleInput.addEventListener("input", () => {
          current.title = titleInput.value;
          const count = root.querySelector("[data-title-count]");
          if (count) count.textContent = `${titleInput.value.length}/200`;
          const errorEl = root.querySelector('[data-error-for="title"]');
          if (errorEl) errorEl.innerHTML = "";
          titleInput.classList.remove("is-invalid");
        });
      }
      const descriptionInput = root.querySelector("[data-field-description]");
      if (descriptionInput) {
        descriptionInput.addEventListener("input", () => {
          current.description = descriptionInput.value;
          const count = root.querySelector("[data-description-count]");
          if (count) {
            count.textContent = `${descriptionInput.value.length}/4000`;
            count.classList.toggle("is-over", descriptionInput.value.length > 4000);
          }
          const errorEl = root.querySelector('[data-error-for="description"]');
          if (errorEl) errorEl.innerHTML = "";
          descriptionInput.classList.remove("is-invalid");
        });
      }

      const nextButton = root.querySelector("[data-next-step]");
      if (nextButton) nextButton.addEventListener("click", () => goStep(current.step + 1));

      // ---- flow options ----
      root.querySelectorAll("[data-flow-option]").forEach((option) => {
        option.addEventListener("click", () => {
          const flowId = option.dataset.flowOption;
          if (current.flowId === flowId) return;
          current.flowId = flowId;
          current.errors = {};
          rerender();
        });
      });

      // ---- check executors ----
      root.querySelectorAll("[data-check-mode]").forEach((button) => {
        button.addEventListener("click", () => {
          const checkpointId = button.dataset.checkMode;
          const mode = button.dataset.mode;
          const binding = current.checks[checkpointId];
          if (!binding || binding.mode === mode) return;
          binding.mode = mode;
          current.errors = { ...current.errors, [checkpointId]: undefined };
          rerender();
        });
      });
      root.querySelectorAll("[data-check-library]").forEach((select) => {
        select.addEventListener("change", async () => {
          const checkpointId = select.dataset.checkLibrary;
          const binding = current.checks[checkpointId];
          if (!binding) return;
          binding.libraryId = select.value;
          current.errors = { ...current.errors, [checkpointId]: undefined };
          if (select.value) await app.loadCheckpointDetail(select.value);
          rerender();
        });
      });
      root.querySelectorAll("[data-semantic-mode]").forEach((button) => {
        button.addEventListener("click", () => {
          const mode = button.dataset.semanticMode;
          if (!current.semantic) current.semantic = { mode: "default", libraryId: "" };
          if (current.semantic.mode === mode) return;
          current.semantic.mode = mode;
          current.errors = { ...current.errors, verificationAccepted: undefined };
          rerender();
        });
      });
      root.querySelectorAll("[data-semantic-library]").forEach((select) => {
        select.addEventListener("change", async () => {
          if (!current.semantic) current.semantic = { mode: "library", libraryId: "" };
          current.semantic.libraryId = select.value;
          current.errors = { ...current.errors, verificationAccepted: undefined };
          if (select.value) await app.loadCheckpointDetail(select.value);
          rerender();
        });
      });
      root.querySelectorAll("[data-check-cli]").forEach((select) => {
        select.addEventListener("change", () => {
          const checkpointId = select.dataset.checkCli;
          const binding = current.checks[checkpointId];
          if (!binding) return;
          binding.cli = select.value;
          const agent = agentFor(app, binding.cli);
          binding.model = defaultModelFor(app, binding.cli);
          binding.modelOverride = "";
          if (agent && !agent.available) {
            toastWarn(`${agent.cli} is not detected`, "The binding is recorded, but the agent CLI must be available when the check runs.");
          }
          rerender();
        });
      });
      const updateEffectiveModel = (checkpointId) => {
        const binding = current.checks[checkpointId];
        const hint = root.querySelector(`[data-effective-model="${CSS.escape(checkpointId)}"]`);
        if (binding && hint) hint.textContent = (binding.modelOverride || "").trim() || binding.model || "—";
      };
      root.querySelectorAll("[data-check-model]").forEach((select) => {
        select.addEventListener("change", () => {
          const checkpointId = select.dataset.checkModel;
          const binding = current.checks[checkpointId];
          if (binding) binding.model = select.value;
          updateEffectiveModel(checkpointId);
        });
      });
      root.querySelectorAll("[data-check-model-override]").forEach((input) => {
        input.addEventListener("input", () => {
          const checkpointId = input.dataset.checkModelOverride;
          const binding = current.checks[checkpointId];
          if (binding) binding.modelOverride = input.value;
          updateEffectiveModel(checkpointId);
          const errorEl = root.querySelector(`[data-check-error="${CSS.escape(checkpointId)}"]`);
          if (errorEl) errorEl.hidden = true;
        });
      });

      // ---- create ----
      const createButton = root.querySelector("[data-create-wish]");
      if (createButton) {
        createButton.addEventListener("click", () => submit(app, root, current, rerender));
      }

      focusFirstError(root);

      // Hydrate library details chosen earlier (list endpoints return summaries only).
      if (current.step === 2) {
        const selectedLibraryIds = [
          ...Object.values(current.checks || {})
            .filter((binding) => binding.mode === "library" && binding.libraryId)
            .map((binding) => binding.libraryId),
          ...(current.semantic && current.semantic.mode === "library" && current.semantic.libraryId ? [current.semantic.libraryId] : []),
        ].filter(Boolean);
        const missing = selectedLibraryIds.filter((id) => !app.state.checkpointDetailById.has(id));
        if (missing.length > 0) {
          Promise.all(missing.map((id) => app.loadCheckpointDetail(id))).then(() => {
            if (current.step === 2) rerender();
          });
        }
      }
    };

    bind();
    return () => {
      /* Draft intentionally survives navigation within the session. */
    };
  },
};

/* ------------------------------------------------------------------ */
/* Render                                                             */
/* ------------------------------------------------------------------ */

function renderWizard(app, current) {
  const step = STEPS[current.step];
  const sample = app.state.sampleMode;
  return `<div class="page-head animate-rise">
    <div class="page-head__titles">
      <span class="eyebrow">Start a wish</span>
      <h1 class="h1">New wish</h1>
      <p class="lede">Write the wish source, choose its flow, and freeze the check executors into the plan. The wizard never runs a check and never starts stage work; it creates a run waiting at the first stage.</p>
    </div>
    <div class="page-head__actions">
      <a class="btn btn--ghost" href="#/wishes">${icon("chevron-left")}Back to wishes</a>
    </div>
  </div>
  ${sample ? renderSampleNotice() : ""}
  <div class="wizard animate-rise is-delay-1">
    <div class="wizard__main">
      <div class="panel panel--pad" style="padding:15px 20px">
        ${renderStepper(current.step)}
      </div>
      <div class="wizard-card">
        <div class="wizard-card__head">
          <span class="wizard-card__num">${String(current.step + 1).padStart(2, "0")}</span>
          <div>
            <div class="wizard-card__title">${esc(step.title)}</div>
            <div class="wizard-card__sub">${esc(step.sub)}</div>
          </div>
        </div>
        <div class="wizard-card__body">
          ${renderStepBody(app, current)}
        </div>
        ${renderStepFoot(app, current)}
      </div>
    </div>
    <aside class="wizard__aside">
      ${renderSummary(app, current)}
      ${renderWhatHappens(app)}
    </aside>
  </div>`;
}

function renderStepper(activeStep) {
  return `<div class="stepper" role="list" aria-label="Wizard steps">${STEPS.map((step, index) => {
    const state = index === activeStep ? "is-active" : index < activeStep ? "is-done" : "";
    const clickable = index < activeStep;
    return `<${clickable ? "button" : "div"} class="stepper__step ${state}" ${clickable ? `type="button" data-step="${index}"` : 'role="listitem"'} ${index === activeStep ? 'aria-current="step"' : ""}>
      <span class="stepper__index">${index < activeStep ? icon("check") : String(index + 1).padStart(2, "0")}</span>
      <span class="stepper__label">${esc(step.label)}</span>
    </${clickable ? "button" : "div"}>${index < STEPS.length - 1 ? '<span class="stepper__line" aria-hidden="true"></span>' : ""}`;
  }).join("")}</div>`;
}

function renderSampleNotice() {
  return `<div class="banner banner--sample" style="margin:0 0 16px;width:100%">
    <span class="banner__icon">${icon("info")}</span>
    <div class="banner__text"><strong>Sample mode</strong><span>The wizard is fully interactive for exploration, but <span class="mono">Create wish</span> is disabled because this labelled dataset cannot write to a store.</span></div>
  </div>`;
}

function renderStepBody(app, current) {
  switch (STEPS[current.step].key) {
    case "identity":
      return renderIdentity(current);
    case "flow":
      return renderFlowStep(app, current);
    case "checks":
      return renderChecksStep(app, current);
    case "review":
      return renderReview(app, current);
    default:
      return "";
  }
}

function renderIdentity(current) {
  return `<div class="form-grid">
    <div class="field">
      <div class="field__row">
        <label class="label" for="wish-id">Wish id <span class="req">*</span></label>
        <span class="char-count">identifier</span>
      </div>
      <div class="field__inline">
        <input class="input input--mono" id="wish-id" data-field-wish-id value="${esc(current.wishId)}" placeholder="wish-web-0001" autocomplete="off" spellcheck="false" aria-describedby="wish-id-hint" />
        <button class="btn btn--sm" type="button" data-suggest-id title="Suggest an id for today">${icon("sparkle")}Suggest</button>
      </div>
      <div class="field__hint" id="wish-id-hint">Stable identifier used by every revision and evidence URI. Pattern: letters, digits, <span class="mono">-</span> and <span class="mono">_</span>.</div>
      <div class="field__error" data-error-for="wishId">${current.errors.wishId ? `${icon("alert-circle")}${esc(current.errors.wishId)}` : ""}</div>
    </div>
    <div class="field">
      <div class="field__row">
        <label class="label" for="wish-title">Title <span class="req">*</span></label>
        <span class="char-count" data-title-count>${current.title.length}/200</span>
      </div>
      <input class="input ${current.errors.title ? "is-invalid" : ""}" id="wish-title" data-field-title value="${esc(current.title)}" placeholder="Add energy budget check" maxlength="200" autocomplete="off" />
      <div class="field__hint">One line that names the change, not the whole intent.</div>
      <div class="field__error" data-error-for="title">${current.errors.title ? `${icon("alert-circle")}${esc(current.errors.title)}` : ""}</div>
    </div>
    <div class="field field--wide">
      <div class="field__row">
        <label class="label" for="wish-description">Description <span class="optional">optional</span></label>
        <span class="char-count" data-description-count>${current.description.length}/4000</span>
      </div>
      <textarea class="textarea" id="wish-description" data-field-description rows="4" placeholder="Introduce a deterministic energy budget gate that fails when the declared budget is exceeded.">${esc(current.description)}</textarea>
      <div class="field__hint">Stored in the wish source marker comment and shown on the wish detail page.</div>
      <div class="field__error" data-error-for="description">${current.errors.description ? `${icon("alert-circle")}${esc(current.errors.description)}` : ""}</div>
    </div>
  </div>`;
}

function renderFlowStep(app, current) {
  const flows = app.state.flows || [];
  return `<div class="flow-picker">
    ${flows
      .map((flow) => {
        const selected = current.flowId === flow.flow_id;
        const stageCount = (flow.stages || []).length;
        const gateCount = (flow.checkpoints || []).filter((checkpoint) => checkpoint.kind === "syntactic").length;
        const semanticCount = (flow.checkpoints || []).filter((checkpoint) => checkpoint.kind === "semantic").length;
        return `<button class="flow-option ${selected ? "is-selected" : ""}" type="button" data-flow-option="${esc(flow.flow_id)}" role="radio" aria-checked="${selected}">
          <span class="flow-option__top">
            <span>
              <span class="flow-option__name">${esc(flow.title || flow.flow_id)}</span>
              <span class="flow-option__id" style="display:block">${esc(flow.flow_id)}</span>
            </span>
            <span class="flow-option__radio" aria-hidden="true"></span>
          </span>
          <span class="flow-option__summary">${esc(flow.summary || "")}</span>
          <span class="flow-option__stages" aria-hidden="true">${Array.from({ length: Math.max(stageCount, 2) }).map(() => "<i></i>").join("")}</span>
          <span class="flow-option__foot">
            <span>${stageCount} stages · ${gateCount} syntactic gates${semanticCount ? ` · ${semanticCount} judged` : ""}</span>
            ${flow.custom_checks_supported ? '<span class="chip chip--violet chip--sm">library bindable</span>' : '<span class="chip chip--sm">semantic only</span>'}
          </span>
        </button>`;
      })
      .join("")}
  </div>
  ${
    current.flowId === "wish-verify"
      ? `<div class="check-note">${icon("info")}<span><strong>WishVerifyFlow keeps its semantic judgments.</strong> Non-default check executors are rejected for this flow in the current slice, so the wizard creates it with the two judged checkpoints only.</span></div>`
      : `<div class="check-note">${icon("library")}<span><strong>SimpleFlow supports library and agent executors.</strong> Each syntactic gate can bind the built-in script, a separate Agent CLI run, or a frozen library checkpoint; the semantic verification gate can bind a semantic library entry. The choice is frozen into the plan.</span></div>`
  }`;
}

/* ------------------------------------------------------------------ */
/* Behaviour                                                          */
/* ------------------------------------------------------------------ */

async function submit(app, root, current, rerender) {
  if (current.submitting) return;
  if (!app.requireWritable()) return;

  const identityErrors = validateIdentity(current);
  const checkErrors = validateChecks(app);
  current.errors = { ...identityErrors, ...checkErrors };
  if (Object.keys(identityErrors).length > 0) {
    current.step = 0;
    rerender();
    return;
  }
  if (Object.keys(checkErrors).length > 0) {
    current.step = 2;
    rerender();
    return;
  }

  const flow = flowFor(app, current.flowId);
  const body = {
    wish_id: current.wishId.trim(),
    title: current.title.trim(),
    description: current.description.trim(),
    flow_id: current.flowId,
  };
  if (flow && flow.flow_id === "simple-flow") {
    body.checks = checkNodeInfo(app, flow).map((node) => {
      const binding = current.checks[node.checkpoint_id] || { mode: "default" };
      if (binding.mode === "cli") {
        return {
          checkpoint_id: node.checkpoint_id,
          mode: "cli",
          cli: binding.cli,
          model: (binding.modelOverride || "").trim() || binding.model,
        };
      }
      if (binding.mode === "library") {
        return { checkpoint_id: node.checkpoint_id, mode: "library", library_id: binding.libraryId };
      }
      return { checkpoint_id: node.checkpoint_id, mode: "default" };
    });
    if (current.semantic && current.semantic.mode === "library" && current.semantic.libraryId) {
      body.checks.push({ checkpoint_id: "verificationAccepted", mode: "library", library_id: current.semantic.libraryId });
    }
  }

  current.submitting = true;
  current.createError = null;
  rerender();

  try {
    const result = await api.create(body);
    current.submitting = false;
    toastOk(
      "Wish created",
      `${result.wish_id || body.wish_id} initialised with a frozen ${current.flowId} plan.`,
    );
    draft = null;
    await app.loadWishes();
    app.state.lastUpdated = Date.now();
    location.hash = `#/wishes/${encodeURIComponent(body.wish_id)}`;
  } catch (error) {
    current.submitting = false;
    if (error instanceof ApiError) {
      if (error.code === "already_exists") {
        current.errors = { ...current.errors, wishId: `The wish id “${body.wish_id}” already exists in the store or legacy registry.` };
        current.createError = "An item with this wish id already exists. Choose another id and try again.";
        current.step = 0;
      } else if (error.status === 400) {
        current.createError = error.message;
      } else {
        current.createError = `${error.message}${error.code === "init_failed" ? " The wish source was written but init failed; check the CLI output." : ""}`;
      }
    } else {
      current.createError = error.message || "Creation failed.";
    }
    rerender();
    toastError("Wish could not be created", current.createError);
  }
}

function renderChecksStep(app, current) {
  const flow = flowFor(app, current.flowId);
  if (!flow || flow.flow_id !== "simple-flow") {
    return `<div class="check-note">${icon("info")}<span><strong>No check bindings for ${esc(flow ? flow.title : current.flowId)}.</strong> This flow judges its checkpoints semantically at verification time; continue to review.</span></div>
    <div class="review__block">
      <div class="review__block-head"><span class="review__block-title">Judged checkpoints</span><span class="chip chip--violet chip--sm">semantic</span></div>
      <div class="review__bindings">
        ${(flow ? flow.checkpoints : [])
          .map((checkpoint) => `<div class="review-binding"><span class="review-binding__name">${esc(checkpoint.title)}</span><span class="review-binding__mode">${icon("sparkle")}${esc(checkpoint.stage_id)} · judged at verification</span></div>`)
          .join("")}
      </div>
    </div>`;
  }
  const nodes = checkNodeInfo(app, flow);
  const agents = app.state.agents || [];
  const libraryError = app.state.checkpointsError;
  return `<div class="check-note check-executor-note">${icon("info")}<span><strong>Stage work is always performed by the working agent.</strong> The <strong>check executor</strong> selects only who verifies a checkpoint: the repository's <strong>built-in script</strong> (deterministic, script authority), a separate <strong>Agent CLI</strong> run (agent authority <span class="mono">cli:model</span>), or a <strong>library checkpoint</strong> whose frozen rules and declared executor are used. The choice is frozen into the plan by <span class="mono">xdesign init</span>.</span></div>
  ${libraryError ? `<div class="form-inline-error" style="margin-top:10px">${icon("alert-circle")}<span>Checkpoint library unavailable (${esc(libraryError.message || "request failed")}). Library mode stays disabled until the service returns.</span></div>` : ""}
  <div class="check-cards">
    ${nodes.map((node, index) => renderCheckCard(app, current, flow, node, index, agents, { libraryError })).join("")}
  </div>
  ${renderSemanticCard(app, current, flow, { libraryError })}
  <div class="check-note">${icon("info")}<span><strong>Built-in script bindings are deterministic</strong> and recorded with script authority. <strong>Agent CLI bindings</strong> run a separate codex/kimi review of the same declared rules and record agent authority. <strong>Library bindings</strong> freeze the library entry's rules, inputs and executor; later library edits never change a frozen plan.</span></div>`;
}
