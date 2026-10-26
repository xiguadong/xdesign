// xdesign · WishFlow Studio — Wish detail: frozen plan lifecycle board, stage
// assessment cards, event timeline, on-demand current work and the web check
// form. Read-only unless the operator explicitly runs a lifecycle action.

import { api, ApiError } from "../api.js";
import {
  esc,
  icon,
  relativeTime,
  absoluteTime,
  statusChip,
  flowChip,
  revChip,
  outcomeChip,
  outcomeTone,
  shortHash,
  truncate,
  copyText,
  prettyJson,
  parseHash,
} from "../util.js";
import {
  openMaterialViewer,
  toastOk,
  toastError,
  toastWarn,
  toastInfo,
  librarySummaryChip,
  executorSummary,
  rawJsonBlock,
  loadingBlock,
  emptyState,
  errorState,
} from "../components.js";
import { getSampleDetail, getSampleWork, getSampleCheckResponse } from "../sample-data.mjs";

let viewDetail = null;

const EVENT_ICONS = {
  init: "flag",
  activity: "git-commit",
  assessment: "shield",
  advance: "arrow-right",
};

let detailState = null; // work + check responses survive surgical polling

export const wishDetailView = {
  title: "Wish detail",
  poll: true,

  crumb(app, param) {
    return param ? `Wish · ${param}` : "Wish detail";
  },

  skeleton() {
    return `<div class="skeleton" style="height:140px"></div>
      <div class="detail-layout" style="margin-top:16px">
        <div class="skeleton" style="height:420px"></div>
        <div class="skeleton" style="height:420px"></div>
      </div>`;
  },

  async load(app, param) {
    const wishId = param;
    if (!wishId) throw new Error("No wish id was provided in the URL.");
    if (app.state.sampleMode) {
      const detail = getSampleDetail(wishId);
      if (!detail) {
        return { notFound: true, wishId };
      }
      detailState = { work: null, workError: null, checkResult: null, checkError: null };
      return { detail, wishId };
    }
    try {
      const detail = await api.wish(wishId);
      detailState = { work: null, workError: null, checkResult: null, checkError: null };
      return { detail, wishId };
    } catch (error) {
      if (error instanceof ApiError && error.status === 404) {
        return { notFound: true, wishId };
      }
      throw error;
    }
  },

  async refresh(app, data) {
    if (data.notFound) return data;
    if (app.state.sampleMode) {
      const detail = getSampleDetail(data.wishId);
      return { ...data, detail };
    }
    const detail = await api.wish(data.wishId);
    return { ...data, detail };
  },

  async pollRefresh(app, data, param, root) {
    if (!root || data.notFound) return data;
    let detail;
    if (app.state.sampleMode) {
      detail = getSampleDetail(data.wishId);
    } else {
      try {
        detail = await api.wish(data.wishId);
      } catch (error) {
        if (error instanceof ApiError && error.status === 404) {
          // The item disappeared from the store; show the not-found state.
          root.innerHTML = renderNotFound(data.wishId);
          return { ...data, notFound: true };
        }
        throw error;
      }
    }
    if (!detail) return data;
    if (signature(detail) === signature(data.detail)) return data;

    const statusChanged = detail.status !== data.detail.status;
    if (statusChanged) {
      // Delivery transitions reshape the action side panels: re-render fully.
      const nextData = { detail, wishId: data.wishId };
      root.innerHTML = wishDetailView.render(app, nextData, param);
      wishDetailView.mount(app, root, nextData, param);
      if (detail.status === "delivered") toastOk("Wish delivered", `${detail.wish_id} completed its run with retained evidence.`);
      return nextData;
    }

    const head = root.querySelector("[data-detail-head]");
    const main = root.querySelector("[data-detail-main]");
    viewDetail = detail;
    if (head) head.innerHTML = renderHead(app, detail, app.state.flows);
    if (main) main.innerHTML = renderMain(app, detail, app.state.flows);
    const handoffSlot = root.querySelector("[data-handoff]");
    if (handoffSlot) handoffSlot.outerHTML = renderHandoff(app, detail, app.state.flows);
    return { detail, wishId: data.wishId };
  },

  render(app, data, param) {
    if (data.notFound) return renderNotFound(data.wishId);
    const detail = data.detail;
    if (!detail) return renderNotFound(param);
    const flows = app.state.flows;
    return `${renderSampleNote(app)}
      <div class="animate-rise" data-detail-head>${renderHead(app, detail, flows)}</div>
      <div class="detail-layout animate-rise is-delay-1">
        <div class="detail-main" data-detail-main>${renderMain(app, detail, flows)}</div>
        <div class="detail-side">${renderSide(app, detail, flows)}</div>
      </div>`;
  },

  mount(app, root, data, param) {
    if (data.notFound) return null;
    const wishId = data.wishId;
    viewDetail = data.detail;
    const state = (detailState = detailState || { work: null, workError: null, checkResult: null, checkError: null });

    // ---- delegated read-only actions (survive surgical polling) ----
    const onClick = (event) => {
      const evidence = event.target.closest("[data-evidence]");
      if (evidence) {
        event.preventDefault();
        openMaterialViewer(app, wishId, evidence.dataset.evidence);
        return;
      }
      const rawToggle = event.target.closest("[data-raw-toggle]");
      if (rawToggle) {
        const target = rawToggle.dataset.rawTarget;
        const panel = target ? root.querySelector(`#${CSS.escape(target)}`) : rawToggle.nextElementSibling;
        if (panel) {
          const open = panel.hidden;
          panel.hidden = !open;
          rawToggle.setAttribute("aria-expanded", String(open));
        }
        return;
      }
      const copyId = event.target.closest("[data-copy-wish-id]");
      if (copyId) {
        copyText(wishId).then((ok) => (ok ? toastOk("Wish id copied", wishId) : toastError("Copy failed")));
        return;
      }
      const refresh = event.target.closest("[data-refresh-view]");
      if (refresh) {
        app.refreshCurrent({ userInitiated: true });
        return;
      }
      const copyHandoff = event.target.closest("[data-copy-handoff]");
      if (copyHandoff) {
        event.preventDefault();
        const kind = copyHandoff.dataset.copyHandoff;
        const texts = handoffTexts(viewDetail, app.state.flows);
        const text = kind === "next" ? texts.next : kind === "check" ? texts.check : texts.prompt;
        copyText(text).then((ok) => {
          if (!ok) {
            toastError("Copy failed", "The browser blocked clipboard access.");
            return;
          }
          if (kind === "prompt") toastOk("Handoff prompt copied", "Paste it into the working agent.");
          else toastOk("Command copied", text);
        });
        return;
      }
      const openLibrary = event.target.closest("[data-open-library]");
      if (openLibrary) {
        event.preventDefault();
        location.hash = `#/checkpoints?open=${encodeURIComponent(openLibrary.dataset.openLibrary)}`;
        return;
      }
      const scrollWork = event.target.closest("[data-scroll-work]");
      if (scrollWork) {
        const panel = root.querySelector("[data-work-panel]");
        if (panel) panel.scrollIntoView({ behavior: "smooth", block: "center" });
        return;
      }
      const scrollCheck = event.target.closest("[data-scroll-check]");
      if (scrollCheck) {
        const panel = root.querySelector("[data-check-panel]");
        if (panel) {
          panel.scrollIntoView({ behavior: "smooth", block: "center" });
          const first = panel.querySelector("input, textarea, select");
          if (first) first.focus({ preventScroll: true });
        }
        return;
      }
    };
    root.addEventListener("click", onClick);

    // Deep link: #/wishes/<id>?material=<uri> opens the material viewer.
    const { params } = parseHash(location.hash);
    const materialUri = params.get("material");
    if (materialUri) {
      const rest = new URLSearchParams(params);
      rest.delete("material");
      const query = rest.toString();
      history.replaceState(null, "", `#/wishes/${encodeURIComponent(wishId)}${query ? "?" + query : ""}`);
      openMaterialViewer(app, wishId, materialUri);
    }

    // ---- run check form ----
    const form = root.querySelector("[data-check-form]");
    let cleanupForm = null;
    if (form) {
      cleanupForm = bindCheckForm(app, root, form, wishId);
    }
    const workButton = root.querySelector("[data-get-work]");
    let cleanupWork = null;
    if (workButton) {
      cleanupWork = bindWorkButton(app, root, workButton, wishId);
    }

    return () => {
      root.removeEventListener("click", onClick);
      if (cleanupForm) cleanupForm();
      if (cleanupWork) cleanupWork();
    };
  },
};

/* ------------------------------------------------------------------ */
/* Signatures                                                         */
/* ------------------------------------------------------------------ */

function signature(detail) {
  if (!detail) return "";
  return `${detail.status}:${detail.current_stage}:${detail.head_revision}:${detail.plan?.plan_sha256}:${(detail.events || []).length}:${detail.run?.current_visit}:${detail.run?.status}`;
}

/* ------------------------------------------------------------------ */
/* Not found                                                          */
/* ------------------------------------------------------------------ */

function renderNotFound(wishId) {
  return `<div class="page-head">
    <div class="page-head__titles">
      <span class="eyebrow">Wish detail</span>
      <h1 class="h1">Wish not found</h1>
      <p class="lede">The store has no wish item or legacy trace with the id <span class="mono">${esc(wishId || "unknown")}</span>.</p>
    </div>
  </div>
  <div class="panel">${emptyState({
    title: "Nothing at that wish id",
    text: "Check the identifier, or browse the register to pick a wish that exists in this workspace.",
    actions: `<a class="btn btn--primary btn--sm" href="#/wishes">${icon("wishes")}Browse wishes</a><a class="btn btn--sm" href="#/new">${icon("plus")}New wish</a>`,
  })}</div>`;
}

/* ------------------------------------------------------------------ */
/* Header                                                             */
/* ------------------------------------------------------------------ */

function lastEvent(detail) {
  const events = getEvents(detail);
  for (let index = events.length - 1; index >= 0; index -= 1) {
    if (events[index].at) return events[index];
  }
  return null;
}

function renderHead(app, detail, flows) {
  const flow = (flows || []).find((entry) => entry.flow_id === detail.flow_id);
  const event = lastEvent(detail);
  const statusClass = detail.status === "blocked" ? "detail-head--blocked" : detail.status === "delivered" ? "detail-head--delivered" : "";
  const plan = detail.plan || {};
  return `<header class="detail-head ${statusClass}">
    <div class="detail-head__crumb">
      <a href="#/wishes">${icon("chevron-left")}Wishes</a>
      <span class="sep-dot"></span>
      <span class="mono">${esc(detail.wish_id)}</span>
      ${detail.source === "legacy" ? '<span class="chip chip--source">legacy trace</span>' : '<span class="chip chip--source">store item</span>'}
    </div>
    <div class="detail-head__row">
      <div style="min-width:0">
        <div class="detail-head__title">
          <h1>${esc(detail.title || detail.wish_id)}</h1>
          ${statusChip(detail.status)}
          ${detail.status === "blocked" ? `<span class="chip chip--rose chip--sm">${icon("alert-triangle")}correction required</span>` : ""}
          ${detail.status === "delivered" ? `<span class="chip chip--green chip--sm">${icon("check-circle")}evidence retained</span>` : ""}
        </div>
        ${detail.title && detail.title !== detail.wish_id ? `<div class="detail-head__id">${esc(detail.wish_id)}</div>` : ""}
        ${detail.description ? `<p class="detail-head__desc">${esc(detail.description)}</p>` : ""}
        <div class="detail-head__meta">
          ${flowChip(detail.flow_id)}
          ${revChip(detail.head_revision)}
          <span class="chip chip--sm">${icon("git-commit")}${esc(plan.plan_id || "plan")}</span>
          <span class="chip chip--sm" title="${esc(plan.plan_sha256 || "")}">${icon("lock")}sha ${esc(shortHash(plan.plan_sha256))}</span>
          <span class="chip chip--sm">${esc(detail.current_stage ? `stage ${stageTitle(flow, detail.current_stage)}` : detail.status === "delivered" ? "run completed" : "no active stage")}</span>
          <span class="chip chip--sm">${icon("clock")}${event ? `last event ${relativeTime(event.at)}` : "no events"}</span>
        </div>
      </div>
      <div class="detail-head__actions">
        <button class="icon-btn" type="button" data-refresh-view title="Refresh wish" aria-label="Refresh wish">${icon("refresh")}</button>
        <button class="btn btn--sm" type="button" data-copy-wish-id>${icon("copy")}Copy id</button>
        <button class="btn btn--sm${app.state.sampleMode ? " is-disabled" : ""}" type="button" data-scroll-check ${detail.status === "delivered" ? "disabled" : ""} title="${detail.status === "delivered" ? "The run is delivered" : app.state.sampleMode ? "Writes are disabled in sample mode" : "Run a web check"}">${icon("play")}Run check</button>
        <button class="btn btn--sm" type="button" data-raw-toggle data-raw-target="detail-raw">${icon("code")}Raw JSON</button>
      </div>
    </div>
    ${renderWarnings(detail)}
    <pre class="raw-json" id="detail-raw" hidden>${esc(prettyJson(detail))}</pre>
  </header>`;
}

/* ------------------------------------------------------------------ */
/* Main column                                                        */
/* ------------------------------------------------------------------ */

function renderMain(app, detail, flows) {
  const flow = (flows || []).find((entry) => entry.flow_id === detail.flow_id);
  return `${detail.status === "delivered" ? renderDelivery(detail, flow) : ""}
    ${renderBoard(detail, flow)}
    ${renderStageCards(detail, flow)}
    ${renderTimeline(app, detail, flow)}`;
}

/* ------------------------------------------------------------------ */
/* Delivery panel                                                     */
/* ------------------------------------------------------------------ */

function renderDelivery(detail, flow) {
  const events = getEvents(detail);
  const assessments = (detail.run?.visits || []).flatMap((visit) => visit.assessments || []);
  const passed = assessments.filter((assessment) => assessment.outcome === "passed").length;
  const plan = detail.plan || {};
  const completedEvent = [...events].reverse().find((event) => event.kind === "advance" && /completed/i.test(event.title || ""));
  return `<section class="delivery">
    <div class="delivery__head">
      <span class="delivery__seal">${icon("check-circle")}</span>
      <div>
        <div class="delivery__title">Delivered — evidence retained</div>
        <div class="delivery__sub">${esc(flow ? flow.title : detail.flow_id)} completed its frozen plan${completedEvent && completedEvent.at ? ` · ${relativeTime(completedEvent.at)}` : ""}</div>
      </div>
    </div>
    <div class="delivery__stats">
      <div class="delivery__stat"><div class="delivery__stat-value">${esc(detail.head_revision ?? "—")}</div><div class="delivery__stat-label">head revision</div></div>
      <div class="delivery__stat"><div class="delivery__stat-value">${(detail.run?.visits || []).length}</div><div class="delivery__stat-label">stage visits</div></div>
      <div class="delivery__stat"><div class="delivery__stat-value">${passed}/${assessments.length}</div><div class="delivery__stat-label">assessments passed</div></div>
      <div class="delivery__stat"><div class="delivery__stat-value">${esc(shortHash(plan.plan_sha256))}</div><div class="delivery__stat-label">plan digest</div></div>
    </div>
    <div class="delivery__actions">
      <button class="btn btn--sm" type="button" data-raw-toggle data-raw-target="delivery-raw">${icon("code")}View raw run JSON</button>
      <a class="btn btn--ghost btn--sm" href="#/wishes">${icon("wishes")}Back to wishes</a>
      <span class="chip chip--green chip--sm">${icon("lock")}plan frozen at init</span>
    </div>
    <pre class="raw-json" id="delivery-raw" hidden>${esc(prettyJson(detail))}</pre>
  </section>`;
}

/* ------------------------------------------------------------------ */
/* Lifecycle board                                                    */
/* ------------------------------------------------------------------ */

function stageTitle(flow, stageId) {
  if (!flow) return stageId;
  return (flow.stages.find((stage) => stage.stage_id === stageId) || {}).title || stageId;
}

function visitForStage(detail, stageId) {
  const visits = detail.run?.visits || [];
  for (let index = visits.length - 1; index >= 0; index -= 1) {
    if (visits[index].stage_id === stageId) return visits[index];
  }
  return null;
}

function latestAssessment(detail, checkpointId) {
  const visits = detail.run?.visits || [];
  for (let index = visits.length - 1; index >= 0; index -= 1) {
    const assessments = visits[index].assessments || [];
    for (let inner = assessments.length - 1; inner >= 0; inner -= 1) {
      if (assessments[inner].checkpoint_id === checkpointId) return assessments[inner];
    }
  }
  return null;
}

function stageState(detail, stageId) {
  const visit = visitForStage(detail, stageId);
  if (detail.status === "delivered") return "done";
  if (visit && visit.status === "blocked") return "blocked";
  if (detail.current_stage === stageId) return detail.status === "blocked" ? "blocked" : "current";
  if (visit && visit.status === "completed") return "done";
  return "idle";
}

function boardGateState(assessment) {
  if (!assessment) return "none";
  return assessment.outcome || "unevaluated";
}

function planCheckpoint(detail, checkpointId) {
  return (detail.plan?.checkpoints || []).find((entry) => entry.checkpoint_id === checkpointId) || null;
}

function bindingFor(detail, checkpointId) {
  const checkpoint = planCheckpoint(detail, checkpointId);
  if (!checkpoint) return null;
  // Syntactic bindings live under checker.binding (SPEC §5.4/§9.4); a semantic
  // library binding lives on the checkpoint itself (checker is null). Default
  // semantic gates carry no binding object at all.
  if (checkpoint.checker && checkpoint.checker.binding) return checkpoint.checker.binding;
  if (checkpoint.binding) return checkpoint.binding;
  return null;
}

function libraryFor(detail, checkpointId) {
  const checkpoint = planCheckpoint(detail, checkpointId);
  return checkpoint && checkpoint.library ? checkpoint.library : null;
}

function bindingLabel(binding, checkpoint = null) {
  const semantic = Boolean(checkpoint && checkpoint.kind === "semantic");
  if (!binding || !binding.mode) return semantic ? "judged" : "built-in script";
  if (binding.mode === "cli") return `${binding.cli}:${binding.model}`;
  if (binding.mode === "library") return `library · ${binding.library_id || "unresolved"}`;
  if (binding.mode === "command") return truncate(binding.command_uri || "command", 22);
  return semantic ? "judged" : "built-in script";
}

function renderWarnings(detail) {
  const warnings = Array.isArray(detail.warnings) ? detail.warnings : [];
  if (warnings.length === 0) return "";
  return `<div class="detail-warnings">${icon("alert-triangle")}
    <div>
      <strong>${warnings.length} frozen library binding${warnings.length === 1 ? "" : "s"} no longer resolve${warnings.length === 1 ? "s" : ""} in the current project library.</strong>
      <ul>${warnings.map((warning) => `<li>${esc(typeof warning === "string" ? warning : warning.message || warning.library_id || "unresolved library id")}</li>`).join("")}</ul>
      <div class="text-faint" style="font-size:10.5px;margin-top:3px">The run keeps its frozen snapshot; only the display summary is missing.</div>
    </div>
  </div>`;
}

function renderLibraryBinding(detail, checkpoint) {
  const binding = bindingFor(detail, checkpoint.checkpoint_id);
  if (!binding || binding.mode !== "library") return "";
  const library = libraryFor(detail, checkpoint.checkpoint_id);
  const missing = !library;
  const rules = (checkpoint.checker && checkpoint.checker.rules) || [];
  return `<div class="binding-stack">
    <div class="library-binding ${missing ? "is-missing" : ""}">
      ${icon(missing ? "alert-triangle" : "library")}
      <span class="library-binding__title">${esc(library ? library.title : "Library checkpoint no longer resolves")}</span>
      <span class="library-binding__meta">${esc(binding.library_id || "")}${library ? ` · ${esc(library.kind)} · v${esc(library.version)}${library.builtin ? " · built-in" : ""}` : ""}</span>
      ${missing
        ? `<span class="library-binding__warn">frozen id is missing from the project library; the frozen rules below are still authoritative.</span>`
        : `<button class="library-binding__open" type="button" data-open-library="${esc(library.checkpoint_id)}">${icon("arrow-up-right")}open in library</button>`}
    </div>
    ${rules.length && checkpoint.kind === "syntactic"
      ? `<div>${rules.map((rule) => `<div class="check-card__preview-rule"><span class="mono">${esc(rule.rule_id)}</span><span>${esc(rule.condition)}</span></div>`).join("")}</div>`
      : ""}
  </div>`;
}

function renderBoard(detail, flow) {
  const stages = flow?.stages || [];
  const checkpoints = detail.plan?.checkpoints || detail.checkpoints || [];
  const columns = stages
    .map((stage, index) => {
      const state = stageState(detail, stage.stage_id);
      const stateLabel = state === "done" ? "done" : state === "current" ? "in progress" : state === "blocked" ? "blocked" : "queued";
      const stageCheckpoints = checkpoints.filter((checkpoint) => checkpoint.stage_id === stage.stage_id);
      const gateMarkup = stageCheckpoints.length
        ? `<div class="board__gates">${stageCheckpoints
            .map((checkpoint) => {
              const assessment = latestAssessment(detail, checkpoint.checkpoint_id);
              const gateState = boardGateState(assessment);
              const binding = bindingFor(detail, checkpoint.checkpoint_id);
              const library = libraryFor(detail, checkpoint.checkpoint_id);
              const isLibrary = binding && binding.mode === "library";
              const missing = isLibrary && !library;
              const bindingText = missing ? "library missing" : bindingLabel(binding, checkpoint);
              const bindingClass = isLibrary ? (missing ? "is-missing" : "is-library") : "";
              const title = `${checkpoint.title || checkpoint.checkpoint_id} · ${gateState}${isLibrary ? ` · library ${binding.library_id}${library ? ` (${library.title})` : " (unresolved)"}` : binding && binding.mode === "cli" ? ` · ${binding.cli}:${binding.model}` : binding ? "" : " · judged"}`;
              return `<div class="board__gate-slot">
                <span class="board__gate is-${gateState}${missing ? " is-warning" : ""} tone-${outcomeTone(assessment?.outcome)}" title="${esc(title)}">
                  <span class="diamond" aria-hidden="true"></span>
                  <span>${esc(gateState === "none" ? "not run" : gateState)}</span>
                </span>
                <span class="board__gate-label" title="${esc(checkpoint.checkpoint_id)}">${esc(truncate(checkpoint.title || checkpoint.checkpoint_id, 26))}</span>
                <span class="board__gate-binding ${bindingClass}" title="${esc(isLibrary ? `library checkpoint ${binding.library_id}${library ? ` · ${library.title} · v${library.version}` : " · unresolved"}` : bindingText)}">${esc(bindingText)}</span>
              </div>`;
            })
            .join("")}</div>`
        : "";
      return `<div class="board__col">
        <div class="board__stage" data-state="${state}">
          ${state === "current" ? '<span class="board__current-pulse" aria-hidden="true"></span>' : ""}
          <div class="board__stage-top">
            <span class="board__stage-num">stage ${String(index + 1).padStart(2, "0")}</span>
            <span class="board__stage-state">${stateLabel}</span>
          </div>
          <div class="board__stage-title">${esc(stage.title)}</div>
        </div>
        ${gateMarkup}
      </div>`;
    })
    .join('<span class="board__connector" aria-hidden="true">' + icon("chevron") + "</span>");

  const plan = detail.plan || {};
  const flowTitle = flow ? flow.title : detail.flow_id;
  return `<section class="panel board">
    <div class="board__flow">
      <div class="board__flow-name">${icon("git-commit")}<span>${esc(flowTitle)}</span><span class="mono">${esc(plan.plan_id || "flow-run")}</span></div>
      <div class="board__plan">
        <span class="chip chip--mono chip--sm">plan sha ${esc(shortHash(plan.plan_sha256))}</span>
        <span class="chip chip--sm">${icon("lock")}frozen at init</span>
      </div>
    </div>
    <div class="board__stages" role="list" aria-label="Lifecycle stages">${columns}</div>
    <div class="board__legend">
      <span class="is-passed"><i></i>passed</span>
      <span class="is-failed"><i></i>failed</span>
      <span class="is-unevaluated"><i></i>unevaluated</span>
      <span class="is-none"><i></i>not run</span>
      <span class="text-faint" style="margin-left:auto">diamonds are stage exit gates · binding shown below</span>
    </div>
  </section>`;
}

/* ------------------------------------------------------------------ */
/* Stage cards                                                        */
/* ------------------------------------------------------------------ */

function renderStageCards(detail, flow) {
  const stages = flow?.stages || [];
  const checkpoints = detail.plan?.checkpoints || [];
  return stages
    .map((stage, index) => {
      const state = stageState(detail, stage.stage_id);
      const visit = visitForStage(detail, stage.stage_id);
      const stageCheckpoints = checkpoints.filter((checkpoint) => checkpoint.stage_id === stage.stage_id);
      const stateLabel = state === "done" ? "completed" : state === "current" ? "in progress" : state === "blocked" ? "blocked" : "not visited";
      return `<section class="stage-card" data-state="${state}">
        <div class="stage-card__head">
          <div class="stage-card__title">
            <span class="stage-card__order">${String(index + 1).padStart(2, "0")}</span>
            <div>
              <div class="stage-card__name">${esc(stage.title)}</div>
              <div class="stage-card__meta">
                <span>${esc(stateLabel)}</span>
                ${visit ? `<span class="sep-dot"></span><span>revision ${esc(visit.artifact_revision)}</span><span class="sep-dot"></span><span>${esc(visit.status)}</span>` : ""}
                ${visit && visit.artifact_digests?.length ? `<span class="sep-dot"></span><span>${visit.artifact_digests.length} artifacts</span>` : ""}
              </div>
            </div>
          </div>
          <div class="row">
            ${stageCheckpoints.length ? `<span class="chip chip--sm">${stageCheckpoints.length} checkpoint${stageCheckpoints.length === 1 ? "" : "s"}</span>` : ""}
            ${visit && visit.artifact_snapshot_digest ? `<span class="chip chip--mono chip--sm" title="${esc(visit.artifact_snapshot_digest)}">snapshot ${esc(shortHash(visit.artifact_snapshot_digest))}</span>` : ""}
          </div>
        </div>
        <div class="stage-card__body">
          ${stageCheckpoints.length ? stageCheckpoints.map((checkpoint) => renderCheckpointRow(detail, checkpoint)).join("") : `<div class="text-faint" style="padding:14px 0;font-size:11.5px">No exit checkpoints declared for this stage.</div>`}
        </div>
      </section>`;
    })
    .join("");
}

function renderCheckpointRow(detail, checkpoint) {
  const assessment = latestAssessment(detail, checkpoint.checkpoint_id);
  const binding = bindingFor(detail, checkpoint.checkpoint_id);
  const isSemantic = checkpoint.kind === "semantic";
  const tone = assessment ? outcomeTone(assessment.outcome) : "slate";
  const visit = visitForStage(detail, checkpoint.stage_id);
  const artifacts = visit?.artifact_digests || [];
  return `<div class="checkpoint-row">
    <div class="checkpoint-row__head">
      <div class="checkpoint-row__title">
        <strong>${esc(checkpoint.title || checkpoint.checkpoint_id)}</strong>
        <span class="chip chip--mono chip--sm">${esc(checkpoint.checkpoint_id)}</span>
        <span class="checkpoint-row__kind ${isSemantic ? "is-semantic" : ""}">${esc(checkpoint.kind)}</span>
        ${
          binding && binding.mode === "library"
            ? `<span class="chip chip--cyan chip--sm">${icon("library")}library checkpoint</span>`
            : binding && binding.mode === "cli"
              ? `<span class="chip chip--violet chip--sm">${icon("cpu")}agent CLI · ${esc(binding.cli)}:${esc(binding.model)}</span>`
              : isSemantic
                ? `<span class="chip chip--violet chip--sm">${icon("sparkle")}default criteria · judged</span>`
                : `<span class="chip chip--green chip--sm">${icon("shield")}built-in script</span>`
        }
      </div>
      ${assessment ? outcomeChip(assessment.outcome) : `<span class="badge badge--soft">not run</span>`}
    </div>
    <ul class="checkpoint-row__criteria">
      ${(checkpoint.criteria || []).map((criterion) => `<li>${esc(criterion)}</li>`).join("")}
    </ul>
    ${renderLibraryBinding(detail, checkpoint)}
    ${
      assessment
        ? `<div class="assessment tone-${tone}">
            <div class="assessment__head">
              <span class="assessment__outcome"><span class="dot"></span>${esc(assessment.outcome)}</span>
              <span class="assessment__who">judged by <span class="mono">${esc(assessment.judged_by || "unknown")}</span> · assessor <span class="mono">${esc(assessment.assessor || "unknown")}</span> · revision <span class="mono">${esc(assessment.evaluated_revision ?? "—")}</span></span>
            </div>
            ${assessment.rationale ? `<div class="assessment__rationale">${esc(assessment.rationale)}</div>` : ""}
            ${(assessment.evidence_uris || []).length ? `<div class="assessment__evidence">${assessment.evidence_uris
                .map((uri) => `<button class="evidence-link" type="button" data-evidence="${esc(uri)}" title="Open ${esc(uri)}">${icon("file")}<span>${esc(truncate(uri, 74))}</span></button>`)
                .join("")}</div>` : ""}
          </div>`
        : `<div class="assessment__none">No assessment recorded for this checkpoint${detail.status === "delivered" ? "" : " yet"}.</div>`
    }
    ${artifacts.length ? `<div class="assessment__artifacts">${artifacts
        .map((artifact) => `<button class="artifact-chip" type="button" data-evidence="${esc(artifact.uri)}" title="${esc(artifact.uri)} · sha256 ${esc(artifact.sha256 || "")}">${icon("file")}<span>${esc(truncate(artifact.uri, 42))}</span><span class="digest">${esc(shortHash(artifact.sha256))}</span></button>`)
        .join("")}</div>` : ""}
  </div>`;
}

/* ------------------------------------------------------------------ */
/* Timeline                                                           */
/* ------------------------------------------------------------------ */

function getEvents(detail) {
  if (Array.isArray(detail.events) && detail.events.length) return detail.events;
  if (detail.run && Array.isArray(detail.run.events)) return detail.run.events;
  return [];
}

function renderTimeline(app, detail, flow) {
  const events = getEvents(detail);
  if (events.length === 0) {
    return `<section class="panel">
      <div class="panel__head"><div class="panel__title">${icon("clock")}Event timeline</div><div class="panel__subtitle">derived from run visits</div></div>
      ${emptyState({ title: "No events yet", text: "The run record has no visits. Fetching next work or running a check would create the first activity.", small: true })}
    </section>`;
  }
  const items = events
    .map((event) => {
      const tone =
        event.kind === "assessment" ? outcomeTone(event.outcome) : event.kind === "advance" ? "cyan" : event.kind === "init" ? "violet" : "slate";
      const title = event.title || event.kind;
      const stage = event.stage_id ? stageTitle(flow, event.stage_id) : null;
      const evidence = event.evidence_uris || [];
      return `<div class="timeline__item">
        <span class="timeline__icon tone-${tone}">${icon(EVENT_ICONS[event.kind] || "info")}</span>
        <div class="timeline__body">
          <div class="timeline__head">
            <span class="timeline__title">${esc(truncate(title, 96))}</span>
            ${stage ? `<span class="timeline__stage">${esc(stage)}</span>` : ""}
            ${event.kind === "assessment" && event.outcome ? outcomeChip(event.outcome, "chip--sm") : ""}
            ${event.kind === "advance" ? `<span class="chip chip--accent chip--sm">advance</span>` : ""}
          </div>
          ${event.detail ? `<div class="timeline__detail">${esc(truncate(event.detail, 190))}</div>` : ""}
          <div class="timeline__meta">
            <span>${esc(event.actor || "xdesign")}</span>
            <span class="sep">·</span>
            <span class="mono" title="${esc(event.at ? absoluteTime(event.at) : "")}">${event.at ? relativeTime(event.at) : "time not recorded"}</span>
          </div>
          ${evidence.length ? `<div class="timeline__evidence">${evidence
            .map((uri) => `<button class="evidence-link" type="button" data-evidence="${esc(uri)}">${icon("file")}<span>${esc(truncate(uri, 64))}</span></button>`)
            .join("")}</div>` : ""}
        </div>
      </div>`;
    })
    .join("");
  return `<section class="panel">
    <div class="panel__head">
      <div class="panel__title">${icon("clock")}Event timeline</div>
      <div class="panel__subtitle">${events.length} event${events.length === 1 ? "" : "s"} · visit order</div>
    </div>
    <div class="timeline">${items}</div>
  </section>`;
}

/* ------------------------------------------------------------------ */
/* Side column                                                        */
/* ------------------------------------------------------------------ */

/* ------------------------------------------------------------------ */
/* Start-work handoff (SPEC §9.5)                                     */
/* ------------------------------------------------------------------ */

function latestFailedAssessment(detail) {
  const visits = detail.run?.visits || [];
  for (let index = visits.length - 1; index >= 0; index -= 1) {
    const assessments = visits[index].assessments || [];
    for (let inner = assessments.length - 1; inner >= 0; inner -= 1) {
      if (assessments[inner].outcome === "failed") return assessments[inner];
    }
  }
  return null;
}

function handoffCorrections(detail, work) {
  if (detail.status !== "blocked") return [];
  const fromWork = (work && work.correction_reasons) || [];
  if (fromWork.length > 0) return fromWork;
  const failed = latestFailedAssessment(detail);
  if (failed) return [`${failed.checkpoint_id}: ${failed.rationale || "the recorded assessment failed."}`];
  return [];
}

function handoffTexts(detail, flows) {
  const flow = (flows || []).find((entry) => entry.flow_id === detail.flow_id);
  const workResult = detailState && detailState.work;
  const response = workResult && workResult.response ? workResult.response : null;
  const work = response && response.work ? response.work : null;
  const stageId = (work && work.stage_id) || detail.current_stage || (flow && flow.stages && flow.stages[0] && flow.stages[0].stage_id) || "first";
  const stage = flow && flow.stages ? flow.stages.find((entry) => entry.stage_id === stageId) : null;
  const goal = (work && work.goal) || (stage && stage.goal) || "Read the current work order with `xdesign next`.";
  const instructions = (work && work.instructions) || (stage && stage.instructions) || "";
  const outputs = (work && work.expected_outputs) || (stage && stage.expected_outputs) || [];
  const corrections = handoffCorrections(detail, work);
  const semantic = hasSemanticGate(detail);

  const lines = [];
  lines.push(`You are the working agent for xdesign wish ${detail.wish_id}${detail.title && detail.title !== detail.wish_id ? ` ("${detail.title}")` : ""}.`);
  lines.push(`Flow: ${flow ? flow.title : detail.flow_id} · current stage: ${stageTitle(flow, stageId)} (${stageId})`);
  lines.push("");
  lines.push(`Goal: ${goal}`);
  if (instructions) lines.push(`Instructions: ${instructions}`);
  if (outputs.length) lines.push(`Expected outputs: ${outputs.join(", ")}`);
  lines.push("");
  lines.push("Run these commands from the project root:");
  lines.push(`  xdesign next ${detail.wish_id}`);
  lines.push(`  xdesign check ${detail.wish_id} --result result.json`);
  lines.push("");
  lines.push("Write result.json from the latest `xdesign next` response. It must include:");
  lines.push("  - the current state_token from that response,");
  lines.push("  - the batch fields for this stage (activities, artifacts and digests),");
  lines.push(
    semantic
      ? "  - a judgment object {authority, identity, outcome, rationale} because this stage has a semantic gate."
      : "  - a judgment object only if this stage has a semantic gate (this one does not).",
  );
  if (work && response) {
    lines.push("");
    lines.push(
      `Loaded work order: stage ${work.stage_id}, visit ${work.visit_index ?? 0}, artifact revision ${work.artifact_revision ?? 0}, state_token ${response.state_token || "(see response)"}.`,
    );
  } else {
    lines.push("");
    lines.push("Current work has not been fetched in this console yet; run `xdesign next` to obtain the state_token before writing result.json.");
  }
  if (corrections.length > 0) {
    lines.push("");
    lines.push("Corrections required before a check can accept this stage:");
    for (const correction of corrections) lines.push(`  - ${correction}`);
  }
  lines.push("");
  lines.push("Do not start stage work from the web console: it displays work orders and submits results only.");

  return {
    prompt: lines.join("\n"),
    next: `xdesign next ${detail.wish_id}`,
    check: `xdesign check ${detail.wish_id} --result result.json`,
    corrections,
    workLoaded: Boolean(work),
  };
}

function renderHandoff(app, detail, flows) {
  const texts = handoffTexts(detail, flows);
  const sample = app.state.sampleMode;
  return `<section class="handoff" data-handoff aria-label="Start work handoff">
    <div class="handoff__head">
      <div class="handoff__title"><span class="icon-badge">${icon("handoff")}</span>Start work
        <span class="chip chip--accent chip--sm">operator handoff</span>
        ${texts.workLoaded ? `<span class="chip chip--green chip--sm">${icon("check")}work order loaded</span>` : ""}
      </div>
    </div>
    <div class="handoff__explain"><strong>Creating this wish froze the plan; it did not start an agent.</strong> Hand the work order to a working agent, let it produce the stage result, then submit <span class="mono">xdesign check</span>. The console only runs the CLI for the operator.</div>
    <div class="handoff__body">
      <div>
        <div class="handoff__block-title"><span>Agent handoff prompt</span>
          <button class="handoff__copy" type="button" data-copy-handoff="prompt" aria-label="Copy the handoff prompt">${icon("copy")}copy</button>
        </div>
        <pre class="handoff__prompt" data-handoff-prompt>${esc(texts.prompt)}</pre>
      </div>
      <div>
        <div class="handoff__block-title"><span>Commands</span></div>
        <div class="handoff__commands">
          <div class="handoff__command"><code data-handoff-command="next">${esc(texts.next)}</code><button class="handoff__copy" type="button" data-copy-handoff="next">${icon("copy")}copy</button></div>
          <div class="handoff__command"><code data-handoff-command="check">${esc(texts.check)}</code><button class="handoff__copy" type="button" data-copy-handoff="check">${icon("copy")}copy</button></div>
        </div>
      </div>
      ${
        texts.corrections.length
          ? `<div class="handoff__corrections"><strong>Corrections required</strong><ul>${texts.corrections.map((correction) => `<li>${esc(correction)}</li>`).join("")}</ul></div>`
          : ""
      }
      <div class="handoff__note">The result file must carry the current <span class="mono">state_token</span>${hasSemanticGate(detail) ? " and a judgment object for the semantic gate" : ""}. Stage work is never started from this console.</div>
      <div class="handoff__actions">
        <button class="btn btn--primary btn--sm" type="button" data-copy-handoff="prompt">${icon("copy")}Copy handoff prompt</button>
        ${
          texts.workLoaded
            ? `<button class="btn btn--ghost btn--sm" type="button" data-scroll-work>${icon("play")}Show current work</button>`
            : `<button class="btn btn--ghost btn--sm${sample ? " is-disabled" : ""}" type="button" data-get-work title="${sample ? "Writes are disabled in sample mode" : "Run xdesign next read-only and compose the prompt with the work order"}">${icon("play")}Load current work</button>`
        }
      </div>
    </div>
  </section>`;
}

function renderSide(app, detail, flows) {
  if (detail.status === "delivered") {
    return `${renderReceipt(detail)}${renderHistory(detail)}${renderRawDetail()}`;
  }
  return `${renderHandoff(app, detail, flows)}
    <div data-work-panel>${renderWorkPanel(app, detail)}</div>
    <div data-check-panel>${renderCheckPanel(app, detail)}</div>
    ${renderHistory(detail)}`;
}

function renderWorkPanel(app, detail) {
  const state = detailState || { work: null, workError: null };
  return `<section class="side-panel">
    <div class="side-panel__head">
      <div class="side-panel__title">${icon("play")}Current work</div>
      <button class="raw-toggle" type="button" data-raw-toggle data-raw-target="work-raw" title="Show work response JSON">${icon("code")}<span>JSON</span></button>
    </div>
    <div class="side-panel__body" data-work-body>
      ${state.work ? renderWorkResult(detail, state.work) : state.workError ? renderWorkError(state.workError) : renderWorkPrompt(app, detail)}
    </div>
    <pre class="raw-json" id="work-raw" hidden>${state.work ? esc(prettyJson(state.work)) : "{}"}</pre>
    <div class="side-panel__note">
      “Get next work” runs <span class="mono">xdesign next</span> read-only against the project root to show what the current stage expects. It never records an activity.
    </div>
  </section>`;
}

function renderWorkPrompt(app, detail) {
  const stage = detail.current_stage;
  return `<div class="work-empty">
    <p>Fetch the current stage instructions, inputs and expected outputs on demand. Work is computed from the frozen plan and the current artifact revision.</p>
    <div class="work-result__chips">
      <span class="chip chip--sm">${esc(stage ? `stage ${stage}` : "no active stage")}</span>
      <span class="chip chip--sm">${esc(detail.run?.status || "run")}</span>
    </div>
    <div class="side-actions">
      <button class="btn btn--primary btn--sm${app.state.sampleMode ? " is-disabled" : ""}" type="button" data-get-work title="${app.state.sampleMode ? "Writes are disabled in sample mode" : "Run xdesign next read-only"}">${icon("play")}Get next work</button>
    </div>
  </div>`;
}

function renderWorkResult(detail, payload) {
  const response = payload.response || payload;
  const work = response.work;
  if (response.completed || !work) {
    return `<div class="work-result">
      <div class="work-result__head"><span class="chip chip--green chip--sm">${icon("check-circle")}run completed</span>
      <span class="chip chip--mono chip--sm">exit ${esc(payload.exit_code ?? 0)}</span></div>
      <div class="work-result__goal">This run is delivered; there is no pending stage work.</div>
      ${renderWorkMeta(response, payload)}
    </div>`;
  }
  const corrections = work.correction_reasons || [];
  return `<div class="work-result">
    <div class="work-result__head">
      <span class="chip chip--accent chip--sm">${icon("layers")}${esc(work.stage_id)}</span>
      <span class="chip chip--mono chip--sm">visit ${esc(work.visit_index ?? 0)}</span>
      <span class="chip chip--mono chip--sm">rev ${esc(work.artifact_revision ?? 0)}</span>
      ${response.status ? `<span class="chip chip--sm">${esc(response.status)}</span>` : ""}
    </div>
    <div class="work-result__goal">${esc(work.goal || "No goal reported for this stage.")}</div>
    ${work.instructions ? `<div><div class="work-result__block-title">Instructions</div><div class="work-result__goal" style="color:var(--muted);font-size:11.5px">${esc(work.instructions)}</div></div>` : ""}
    ${(work.expected_outputs || []).length ? `<div><div class="work-result__block-title">Expected outputs</div><div class="work-result__chips">${work.expected_outputs.map((output) => `<span class="chip chip--mono chip--sm">${esc(output)}</span>`).join("")}</div></div>` : ""}
    ${(work.exit_checkpoints || []).length ? `<div><div class="work-result__block-title">Exit checkpoints</div><div class="work-result__chips">${work.exit_checkpoints.map((id) => `<span class="chip chip--sm">${icon("diamond")}${esc(id)}</span>`).join("")}</div></div>` : ""}
    ${(work.current_assessments || []).length ? `<div><div class="work-result__block-title">Current assessments</div>${work.current_assessments.map((assessment) => `<div class="row" style="margin-bottom:4px">${outcomeChip(assessment.outcome)}<span class="mono text-faint" style="font-size:9.5px">${esc(assessment.checkpoint_id || "")}</span><span class="text-faint" style="font-size:10.5px">${esc(assessment.rationale || "")}</span></div>`).join("")}</div>` : ""}
    ${corrections.length ? `<div><div class="work-result__block-title" style="color:var(--rose)">Correction reasons</div><ul class="work-result__list">${corrections.map((reason) => `<li class="correction">${esc(reason)}</li>`).join("")}</ul></div>` : ""}
    ${renderWorkMeta(response, payload)}
    <div class="side-actions">
      <button class="btn btn--sm" type="button" data-get-work>${icon("refresh")}Refresh work</button>
    </div>
  </div>`;
}

function renderWorkMeta(response, payload) {
  return `<div class="work-meta">
    <span><b>run</b> ${esc(response.run_id ? shortHash(response.run_id) : "—")}</span>
    <span><b>store rev</b> ${esc(response.store_revision ?? "—")}</span>
    <span><b>exit</b> ${esc(payload.exit_code ?? "—")}</span>
    <span><b>flow</b> ${esc(response.flow_id || "—")}</span>
  </div>`;
}

function renderWorkError(error) {
  return `<div class="work-empty">
    <div class="form-inline-error">${icon("alert-circle")}<span>${esc(error.message || "The next-work request failed.")}</span></div>
    ${error.detail ? `<pre class="material-pre" style="max-height:160px;font-size:10px">${esc(typeof error.detail === "string" ? error.detail : prettyJson(error.detail))}</pre>` : ""}
    <div class="side-actions">
      <button class="btn btn--sm" type="button" data-get-work>${icon("refresh")}Retry</button>
    </div>
  </div>`;
}

/* ---- run check form ------------------------------------------------ */

function hasSemanticGate(detail) {
  const checkpoints = detail.plan?.checkpoints || [];
  const stage = detail.current_stage;
  if (!stage) return false;
  return checkpoints.some((checkpoint) => checkpoint.kind === "semantic" && checkpoint.stage_id === stage);
}

function renderCheckPanel(app, detail) {
  const sample = app.state.sampleMode;
  const semantic = hasSemanticGate(detail);
  if (detail.status === "delivered") {
    return `<section class="side-panel">
      <div class="side-panel__head"><div class="side-panel__title">${icon("shield")}Run check</div></div>
      <div class="side-panel__body"><div class="work-empty"><p>This wish is delivered. The run completed and no further check can advance it.</p></div></div>
    </section>`;
  }
  const stage = detail.current_stage;
  return `<section class="side-panel">
    <div class="side-panel__head">
      <div class="side-panel__title">${icon("shield")}Run check</div>
      <span class="chip chip--mono chip--sm">${esc(stage || "first stage")}</span>
    </div>
    <div class="side-panel__body">
      <form class="form-stack" data-check-form autocomplete="off">
        <div class="field">
          <span class="label">Description</span>
          <input class="input" name="description" value="${esc(stage ? `Web check, stage ${stage}` : "Web check")}" placeholder="Web check, stage design" />
        </div>
        <div class="field">
          <span class="label">Performed by</span>
          <input class="input" name="performed_by" value="xdesign web operator" placeholder="xdesign web operator" />
        </div>
        <div class="field">
          <span class="label">Artifact URIs <span class="optional">one per line</span></span>
          <textarea class="textarea textarea--mono" name="artifacts" rows="3" placeholder="docs/design/flows/simple-flow/definition.md">${esc(guessArtifactUri(detail))}</textarea>
        </div>
        ${
          semantic
            ? `<div class="field">
                <span class="label">Judgment <span class="optional">semantic gate</span></span>
                <div class="form-row">
                  <select class="select" name="authority" aria-label="Judgment authority">
                    <option value="agent">authority · agent</option>
                    <option value="human">authority · human</option>
                  </select>
                  <input class="input" name="identity" value="operator" placeholder="identity" aria-label="Judgment identity" />
                </div>
              </div>
              <div class="field">
                <span class="label">Outcome</span>
                <select class="select" name="outcome" aria-label="Judgment outcome">
                  <option value="passed">passed</option>
                  <option value="failed">failed</option>
                  <option value="unevaluated">unevaluated</option>
                </select>
              </div>
              <div class="field">
                <span class="label">Rationale</span>
                <textarea class="textarea" name="rationale" rows="3">Reviewed the retained evidence set.</textarea>
              </div>`
            : `<div class="field__hint">This stage has no semantic gate; the check runs the declared deterministic checker only.</div>`
        }
        <div class="form-inline-error" data-check-error hidden></div>
        <div class="form-actions">
          <button class="btn btn--primary btn--sm" type="submit" data-check-submit ${sample ? "disabled" : ""} title="${sample ? "Writes are disabled in sample mode" : ""}">
            ${icon("send")}Submit check
          </button>
        </div>
        ${sample ? '<div class="field__hint">Sample mode: the check form is disabled because it would write an assessment.</div>' : ""}
      </form>
      <div data-check-response style="margin-top:14px"></div>
    </div>
  </section>`;
}

function guessArtifactUri(detail) {
  const stage = detail.current_stage;
  const flowId = detail.flow_id;
  if (stage === "requirements") return "mbse/model/requirements.sysml";
  if (stage === "design") return `docs/design/flows/${flowId}/definition.md`;
  if (stage === "implement") return "src/lib.rs";
  if (stage === "verify") return "mbse/model/verification.sysml";
  return "";
}

function bindCheckForm(app, root, form, wishId) {
  const submit = async (event) => {
    event.preventDefault();
    if (!app.requireWritable()) return;
    const submitButton = form.querySelector("[data-check-submit]");
    const errorBox = form.querySelector("[data-check-error]");
    if (errorBox) errorBox.hidden = true;

    const data = new FormData(form);
    const description = String(data.get("description") || "").trim();
    const performedBy = String(data.get("performed_by") || "").trim();
    const artifactLines = String(data.get("artifacts") || "")
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean);
    const body = {
      description: description || undefined,
      performed_by: performedBy || undefined,
      artifacts: artifactLines.map((uri) => ({ uri })),
    };
    const outcome = data.get("outcome");
    if (outcome) {
      body.judgment = {
        authority: data.get("authority") === "human" ? "human" : "agent",
        identity: String(data.get("identity") || "operator").trim() || "operator",
        outcome: ["passed", "failed", "unevaluated"].includes(outcome) ? outcome : "passed",
        rationale: String(data.get("rationale") || "").trim() || "No rationale provided.",
      };
    }

    if (submitButton) {
      submitButton.classList.add("is-busy");
      submitButton.disabled = true;
    }
    try {
      const result = app.state.sampleMode ? getSampleCheckResponse(wishId, body) : await api.check(wishId, body);
      renderCheckResponse(root, result);
      const exitCode = result.exit_code ?? 0;
      const response = result.response || {};
      if (exitCode === 0) {
        const advance = response.advance;
        toastOk("Check accepted", advance ? `Response: ${advance}` : "The stage was accepted by the checker.");
      } else if (exitCode === 1) {
        const reasons = response.correction_reasons || [];
        toastWarn("Stage remains blocked", reasons[0] || "The check did not accept the stage; see the correction reasons.");
      } else {
        toastInfo("Check completed with a non-zero exit code", `exit ${exitCode}`);
      }
      await app.loadWishes();
      app.refreshCurrent({ silent: true });
    } catch (error) {
      if (errorBox) {
        errorBox.hidden = false;
        errorBox.innerHTML = `${icon("alert-circle")}<span>${esc(error.message || "The check request failed.")}</span>`;
      }
      if (error instanceof ApiError) {
        if (error.code === "already_delivered") toastWarn("Already delivered", "This wish completed its run; there is no gate left to check.");
        else toastError("Check failed", error.message);
      } else {
        toastError("Check failed", error.message || "Unexpected error");
      }
    } finally {
      if (submitButton) {
        submitButton.classList.remove("is-busy");
        submitButton.disabled = app.state.sampleMode;
      }
    }
  };
  form.addEventListener("submit", submit);
  return () => form.removeEventListener("submit", submit);
}

function renderCheckResponse(root, result) {
  const container = root.querySelector("[data-check-response]");
  if (!container) return;
  const response = result.response || {};
  const assessed = response.assessed || {};
  const corrections = response.correction_reasons || [];
  const exitCode = result.exit_code ?? 0;
  container.innerHTML = `<div class="assessment tone-${exitCode === 0 ? "green" : exitCode === 1 ? "rose" : "amber"}">
      <div class="assessment__head">
        <span class="assessment__outcome"><span class="dot"></span>${esc(exitCode === 0 ? "accepted" : exitCode === 1 ? "blocked" : "error")}</span>
        <span class="assessment__who">exit <span class="mono">${esc(exitCode)}</span>${response.advance ? ` · advance <span class="mono">${esc(response.advance)}</span>` : ""}</span>
      </div>
      ${assessed.checkpoint_id ? `<div class="assessment__who" style="margin-top:4px">checkpoint <span class="mono">${esc(assessed.checkpoint_id)}</span>${assessed.outcome ? ` · outcome <span class="mono">${esc(assessed.outcome)}</span>` : ""}${assessed.judged_by ? ` · judged by <span class="mono">${esc(assessed.judged_by)}</span>` : ""}${assessed.assessor ? ` · <span class="mono">${esc(assessed.assessor)}</span>` : ""}</div>` : ""}
      ${assessed.rationale ? `<div class="assessment__rationale">${esc(assessed.rationale)}</div>` : ""}
      ${corrections.length ? `<ul class="work-result__list" style="margin-top:8px">${corrections.map((reason) => `<li class="correction">${esc(reason)}</li>`).join("")}</ul>` : ""}
      ${result.stderr ? `<div class="assessment__rationale" style="color:var(--muted)">stderr: ${esc(truncate(result.stderr, 220))}</div>` : ""}
    </div>
    <div class="raw-block">
      <button class="raw-toggle" type="button" data-raw-toggle>${icon("code")}<span>Response JSON</span></button>
      <pre class="raw-json" hidden>${esc(prettyJson(result))}</pre>
    </div>`;
}

/* ---- current work button ------------------------------------------- */

function refreshHandoff(app, root) {
  if (!root || !viewDetail) return;
  const slot = root.querySelector("[data-handoff]");
  if (slot) slot.outerHTML = renderHandoff(app, viewDetail, app.state.flows);
}

function bindWorkButton(app, root, button, wishId) {
  const onClick = async (event) => {
    const target = event.target.closest("[data-get-work]");
    if (!target) return;
    event.preventDefault();
    if (app.state.sampleMode) {
      toastWarn("Sample mode: read-only demo", "Work is fetched from the live CLI; the demo dataset cannot run it.");
      return;
    }
    const body = root.querySelector("[data-work-body]");
    const original = body ? body.innerHTML : "";
    if (body) body.innerHTML = loadingBlock("Running xdesign next…");
    target.disabled = true;
    try {
      const result = await api.work(wishId);
      detailState = { ...(detailState || {}), work: result, workError: null };
      if (body) body.innerHTML = renderWorkResult({ status: "active" }, result);
      refreshHandoff(app, root);
      toastOk("Work fetched", result.response?.work ? `Stage ${result.response.work.stage_id} is waiting for artifacts.` : "Run completed.");
    } catch (error) {
      const detail = error instanceof ApiError ? error : null;
      const workError = { message: error.message, detail: detail?.detail };
      detailState = { ...(detailState || {}), workError, work: null };
      if (body) body.innerHTML = renderWorkError(workError);
      refreshHandoff(app, root);
      if (error instanceof ApiError && error.code === "work_failed") {
        toastError("Next work failed", error.message);
      } else {
        toastError("Next work failed", error.message || "Unexpected error");
      }
    } finally {
      const refreshed = root.querySelector("[data-get-work]");
      if (refreshed) refreshed.disabled = false;
      else if (body && original === "") body.innerHTML = renderWorkPrompt(app, { current_stage: null, run: {} });
    }
  };
  root.addEventListener("click", onClick);
  return () => root.removeEventListener("click", onClick);
}

/* ------------------------------------------------------------------ */
/* Delivered receipt / history                                        */
/* ------------------------------------------------------------------ */

function renderReceipt(detail) {
  const plan = detail.plan || {};
  const assessments = (detail.run?.visits || []).flatMap((visit) => visit.assessments || []);
  const events = getEvents(detail);
  const completed = [...events].reverse().find((event) => event.kind === "advance" && /completed/i.test(event.title || ""));
  return `<section class="side-panel">
    <div class="side-panel__head"><div class="side-panel__title">${icon("check-circle")}Delivery receipt</div><span class="chip chip--green chip--sm">complete</span></div>
    <div class="side-panel__body" style="display:flex;flex-direction:column;gap:8px">
      ${receiptRow("Status", "delivered")}
      ${receiptRow("Head revision", detail.head_revision ?? "—", true)}
      ${receiptRow("Plan", plan.plan_id || "—", true)}
      ${receiptRow("Plan digest", shortHash(plan.plan_sha256), true)}
      ${receiptRow("Completed", completed ? (completed.at ? relativeTime(completed.at) : "recorded") : "recorded")}
      ${receiptRow("Assessments", `${assessments.filter((assessment) => assessment.outcome === "passed").length}/${assessments.length} passed`)}
      ${receiptRow("Source", detail.source || "store")}
    </div>
    <div class="side-panel__note">A delivered wish keeps its frozen plan and evidence. No lifecycle action can advance it further; create a new wish for follow-up work.</div>
  </section>`;
}

function receiptRow(label, value, mono = false) {
  return `<div class="summary-row"><span class="summary-row__key">${esc(label)}</span><span class="summary-row__value${mono ? " mono" : ""}">${esc(value)}</span></div>`;
}

function renderHistory(detail) {
  const history = detail.history || [];
  if (history.length === 0) {
    return `<section class="side-panel">
      <div class="side-panel__head"><div class="side-panel__title">${icon("database")}Store history</div></div>
      <div class="side-panel__body"><div class="text-faint" style="font-size:11.5px">${detail.source === "legacy" ? "Legacy traces do not have store revisions; the run record is the authority." : "No store revisions are reported for this item."}</div></div>
    </section>`;
  }
  const rows = [...history]
    .slice(-6)
    .reverse()
    .map(
      (entry) => `<div class="summary-row">
        <span class="summary-row__key"><span class="mono">rev ${esc(entry.revision)}</span></span>
        <span class="summary-row__value mono" title="operation ${esc(entry.operation_id || "")}">${esc(entry.material_count ?? "—")} materials</span>
      </div>`,
    )
    .join("");
  return `<section class="side-panel">
    <div class="side-panel__head"><div class="side-panel__title">${icon("database")}Store history</div><span class="chip chip--mono chip--sm">${history.length} rev</span></div>
    <div class="side-panel__body" style="display:flex;flex-direction:column;gap:2px">
      ${rows}
      <div class="field__hint" style="margin-top:6px">Newest first · materials remain addressable through the material viewer.</div>
    </div>
  </section>`;
}

function renderRawDetail() {
  return `<section class="side-panel">
    <div class="side-panel__head"><div class="side-panel__title">${icon("code")}Raw payload</div></div>
    <div class="side-panel__body">
      <button class="btn btn--sm" type="button" data-raw-toggle>${icon("code")}Toggle detail JSON</button>
      <pre class="raw-json" hidden>${esc(prettyJson(detailPlaceholder()))}</pre>
      <div class="field__hint" style="margin-top:8px">The header also exposes the raw GET /api/wishes/&lt;id&gt; response.</div>
    </div>
  </section>`;
}

function detailPlaceholder() {
  return { note: "detail payload is rendered by the header Raw JSON toggle" };
}

/* ------------------------------------------------------------------ */
/* Sample notice                                                      */
/* ------------------------------------------------------------------ */

function renderSampleNote(app) {
  if (!app.state.sampleMode) return "";
  return `<div class="banner banner--sample" style="margin:0 0 16px;width:100%">
    <span class="banner__icon">${icon("info")}</span>
    <div class="banner__text"><strong>Sample data</strong><span>This wish belongs to the labelled demo dataset. Lifecycle actions are disabled and the material viewer shows generated sample material.</span></div>
  </div>`;
}
