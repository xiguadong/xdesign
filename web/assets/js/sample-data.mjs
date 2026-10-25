// xdesign · WishFlow Studio — labelled sample dataset.
//
// Used by the page when the API is unreachable or the store has no wishes, and
// by the standalone dev mock server (web/dev/). Every payload below follows the
// shapes in web/SPEC.md §5 exactly, so the UI code never branches on demo data.
// Sample data is display-only: the page disables every write action in sample
// mode and never reports a fabricated success.

const NOW = Date.now();
const AGO = (minutes) => new Date(NOW - minutes * 60_000).toISOString();

function digest(seed) {
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

const RISK = (seed) => digest(seed);

/* ------------------------------------------------------------------ */
/* Health                                                             */
/* ------------------------------------------------------------------ */

export const SAMPLE_HEALTH = {
  version: "0.1.0",
  project_root: "/Users/operator/projects/xdesign",
  store_root: "/Users/operator/projects/xdesign/.xdesign",
  store_present: true,
  agents: [
    { cli: "codex", available: true, version: "codex-cli 0.162.1" },
    { cli: "kimi", available: false, version: null },
  ],
  now: new Date(NOW).toISOString(),
};

/* ------------------------------------------------------------------ */
/* Flow catalog                                                       */
/* ------------------------------------------------------------------ */

const simpleFlow = {
  flow_id: "simple-flow",
  title: "SimpleFlow",
  summary: "Four gated stages: requirements, design, implement, verify.",
  custom_checks_supported: true,
  stages: [
    {
      stage_id: "requirements",
      title: "Requirements",
      goal: "Define the requirements and the rationale that makes them necessary.",
      instructions:
        "State the requirements, their rationale and the satisfy relationships in the project model, and write the wish source so every requirement traces back to the wish.",
      expected_outputs: ["checker:simpleRequirementsCheck", "mbse/model/requirements.sysml"],
    },
    {
      stage_id: "design",
      title: "Design",
      goal: "Decide how the system satisfies each requirement, and record why.",
      instructions:
        "Record the design decisions, the alternatives that were rejected, and the trace from each requirement to the design element that satisfies it.",
      expected_outputs: ["checker:simpleDesignCheck", "docs/design/flows/simple-flow/definition.md"],
    },
    {
      stage_id: "implement",
      title: "Implementation",
      goal: "Implement the design and produce the artifacts the implementation needs.",
      instructions:
        "Produce the code and artifacts that realise the design; record every artifact URI on the stage activity so the evidence set is complete.",
      expected_outputs: ["checker:simpleImplementationCheck", "src/lib.rs"],
    },
    {
      stage_id: "verify",
      title: "Verification",
      goal: "Verify the implementation against the requirements and retain the result.",
      instructions:
        "Run the verification, retain the evidence, and record the verification outcome as an assessment with an explicit authority and rationale.",
      expected_outputs: ["checker:simpleRecordCheck", "mbse/model/verification.sysml"],
    },
  ],
  checkpoints: [
    {
      checkpoint_id: "simpleRequirementsAccepted",
      stage_id: "requirements",
      kind: "syntactic",
      title: "simpleRequirementsCheck",
      criteria: [
        "Every requirement declared in the project model carries its rationale and at least one satisfy relationship.",
        "Each declared requirement traces to the wish source and the claim it supports.",
      ],
      agent_may_judge: false,
      human_may_judge: false,
      checker: {
        checker_id: "simpleRequirementsCheck",
        command_uri: "tools/checks/simple_requirements.py",
        establishes: ["identifier_conformance", "trace_conformance"],
        rules: [
          {
            rule_id: "requirements-carry-rationale",
            condition: "Every XDesignRequirement declares a non-empty rationale attribute.",
            reads: ["mbse/model/requirements.sysml"],
          },
          {
            rule_id: "satisfy-links-resolve",
            condition:
              "Every satisfy relationship names a requirement and a design element that exist in the same project model.",
            reads: ["mbse/model/requirements.sysml", "mbse/model/design.sysml"],
          },
          {
            rule_id: "wish-source-declared",
            condition: "The wish source declares the wish package and at least one requirement identifier.",
            reads: ["cache/web/wishes/wish-0012.sysml"],
          },
        ],
        modes: ["default", "cli"],
      },
    },
    {
      checkpoint_id: "simpleDesignAccepted",
      stage_id: "design",
      kind: "syntactic",
      title: "simpleDesignCheck",
      criteria: [
        "Each requirement is satisfied by at least one design element.",
        "No orphan design element: every element is referenced by a requirement or another design element.",
      ],
      agent_may_judge: false,
      human_may_judge: false,
      checker: {
        checker_id: "simpleDesignCheck",
        command_uri: "tools/checks/simple_design.py",
        establishes: ["trace_conformance", "layout_conformance"],
        rules: [
          {
            rule_id: "requirement-satisfied-by-design",
            condition: "Every requirement participates in at least one satisfy relationship with a design element.",
            reads: ["mbse/model/requirements.sysml", "mbse/model/design.sysml"],
          },
          {
            rule_id: "design-identifiers-unique",
            condition: "No two design elements share an identifier in the project model.",
            reads: ["mbse/model/design.sysml"],
          },
        ],
        modes: ["default", "cli"],
      },
    },
    {
      checkpoint_id: "simpleImplementationAccepted",
      stage_id: "implement",
      kind: "syntactic",
      title: "simpleImplementationCheck",
      criteria: [
        "The implementation declares every artifact the design requires.",
        "Artifact digests are stable for the recorded revision.",
      ],
      agent_may_judge: false,
      human_may_judge: false,
      checker: {
        checker_id: "simpleImplementationCheck",
        command_uri: "tools/checks/simple_implementation.py",
        establishes: ["artifact_completeness", "digest_stability"],
        rules: [
          {
            rule_id: "implementation-artifacts-present",
            condition: "Every artifact URI required by the frozen plan exists in the project tree.",
            reads: ["src/lib.rs", "web/assets/index.html"],
          },
          {
            rule_id: "artifact-digests-stable",
            condition: "Artifact digests for the recorded revision match the current project bytes.",
            reads: ["src/lib.rs", "web/assets/styles.css"],
          },
        ],
        modes: ["default", "cli"],
      },
    },
    {
      checkpoint_id: "simpleRecordAccepted",
      stage_id: "verify",
      kind: "syntactic",
      title: "simpleRecordCheck",
      criteria: [
        "The run record names every assessed checkpoint and its retained evidence.",
        "Verification evidence is retained under the project caches.",
      ],
      agent_may_judge: false,
      human_may_judge: false,
      checker: {
        checker_id: "simpleRecordCheck",
        command_uri: "tools/checks/simple_record.py",
        establishes: ["record_conformance"],
        rules: [
          {
            rule_id: "assessments-have-evidence",
            condition: "Every assessment in the run record carries at least one retained evidence URI.",
            reads: ["cache/web/attempts/wish-0011-4f2a.json"],
          },
          {
            rule_id: "record-revisions-monotonic",
            condition: "Recorded artifact revisions increase monotonically across visits.",
            reads: [".xdesign/items/wish-0011"],
          },
        ],
        modes: ["default", "cli"],
      },
    },
    {
      checkpoint_id: "verificationAccepted",
      stage_id: "verify",
      kind: "semantic",
      title: "verificationAccepted",
      criteria: [
        "Verification evidence demonstrates that the implementation satisfies the requirements.",
        "Residual risk is stated explicitly and accepted by the judging authority.",
      ],
      agent_may_judge: true,
      human_may_judge: true,
      checker: null,
    },
  ],
};

const wishVerifyFlow = {
  flow_id: "wish-verify",
  title: "WishVerifyFlow",
  summary: "Two judged stages: wish, verify.",
  custom_checks_supported: false,
  stages: [
    {
      stage_id: "wish",
      title: "Wish",
      goal: "State what is wanted and why it matters.",
      instructions:
        "Record the wish statement, its motivation, and the acceptance conditions the verification will later check.",
      expected_outputs: ["Wish source with acceptance conditions"],
    },
    {
      stage_id: "verify",
      title: "Verification",
      goal: "Judge whether the delivered evidence satisfies the wish.",
      instructions:
        "Review the evidence set, state the outcome, and record the authority and rationale for the judgment.",
      expected_outputs: ["Assessment record with evidence references"],
    },
  ],
  checkpoints: [
    {
      checkpoint_id: "wishAccepted",
      stage_id: "wish",
      kind: "semantic",
      title: "wishAccepted",
      criteria: [
        "The wish statement is specific enough to be verified.",
        "The acceptance conditions are stated and testable.",
      ],
      agent_may_judge: true,
      human_may_judge: true,
      checker: null,
    },
    {
      checkpoint_id: "outcomeAccepted",
      stage_id: "verify",
      kind: "semantic",
      title: "outcomeAccepted",
      criteria: [
        "The verification evidence speaks to every acceptance condition.",
        "The reported outcome is consistent with the evidence set.",
      ],
      agent_may_judge: true,
      human_may_judge: true,
      checker: null,
    },
  ],
};

export const SAMPLE_FLOWS = {
  flows: [simpleFlow, wishVerifyFlow],
  agents: [
    {
      cli: "codex",
      label: "Codex CLI",
      command: "codex",
      available: true,
      version: "codex-cli 0.162.1",
      default_model: "gpt-6-astra",
      models: [
        { id: "gpt-6-astra", label: "GPT-6 Astra" },
        { id: "gpt-5.6-terra", label: "GPT-5.6 Terra" },
        { id: "gpt-5.1-codex", label: "GPT-5.1 Codex" },
        { id: "gpt-5-codex", label: "GPT-5 Codex" },
        { id: "o4-mini", label: "o4-mini" },
      ],
    },
    {
      cli: "kimi",
      label: "Kimi Code CLI",
      command: "kimi",
      available: false,
      version: null,
      default_model: "kimi-code/k3.1-pro",
      models: [
        { id: "kimi-code/k3.1-pro", label: "K3.1 Pro" },
        { id: "kimi-code/k3.1-pro-highspeed", label: "K3.1 Pro Highspeed" },
        { id: "kimi-code/k3", label: "K3" },
        { id: "kimi-code/k3-256k", label: "K3 256k" },
        { id: "kimi-code/kimi-for-coding", label: "K2.8 Preview" },
        { id: "kimi-code/kimi-for-coding-highspeed", label: "K2.7 Code Highspeed" },
      ],
    },
  ],
};

export function getSampleFlow(flowId) {
  return SAMPLE_FLOWS.flows.find((flow) => flow.flow_id === flowId) || null;
}

export function getSampleAgent(cli) {
  return SAMPLE_FLOWS.agents.find((agent) => agent.cli === cli) || null;
}

/* ------------------------------------------------------------------ */
/* Wish list                                                          */
/* ------------------------------------------------------------------ */

export const SAMPLE_WISHES = [
  {
    wish_id: "wish-0013",
    title: "Operator web console",
    description:
      "Serve the operator page from the xdesign binary: embedded assets, a lifecycle board and a read-only material viewer.",
    flow_id: "simple-flow",
    status: "blocked",
    current_stage: "implement",
    head_revision: 2,
    source: "store",
    updated_at: AGO(57),
  },
  {
    wish_id: "wish-0007",
    title: "Legacy bootstrap trace",
    description:
      "Bootstrap wish trace retained in Git; the legacy run record reports the wish stage as blocked.",
    flow_id: "wish-verify",
    status: "blocked",
    current_stage: "wish",
    head_revision: null,
    source: "legacy",
    updated_at: null,
  },
  {
    wish_id: "wish-0012",
    title: "Custom check bindings",
    description:
      "Bind an agent CLI verdict to the design gate and freeze the binding into the plan without changing the deterministic defaults.",
    flow_id: "simple-flow",
    status: "active",
    current_stage: "design",
    head_revision: 3,
    source: "store",
    updated_at: AGO(18),
  },
  {
    wish_id: "wish-0006",
    title: "Agent gate audit",
    description: "Audit the two judged gates of WishVerifyFlow against the retained evidence set.",
    flow_id: "wish-verify",
    status: "active",
    current_stage: "verify",
    head_revision: 2,
    source: "store",
    updated_at: AGO(33),
  },
  {
    wish_id: "wish-0014",
    title: "Evidence retention review",
    description:
      "Review retained reports and plan materials for the storage slice before the next verification pass.",
    flow_id: "simple-flow",
    status: "received",
    current_stage: null,
    head_revision: 1,
    source: "store",
    updated_at: AGO(96),
  },
  {
    wish_id: "wish-0011",
    title: "Storage slice",
    description:
      "Move wish runs onto the operational store: revisions, materials, receipts and deterministic replay from the frozen plan.",
    flow_id: "simple-flow",
    status: "delivered",
    current_stage: null,
    head_revision: 5,
    source: "store",
    updated_at: AGO(240),
  },
  {
    wish_id: "wish-0009",
    title: "Check runner diagnostics",
    description: "Surface declared checker rules and collection diagnostics in the stage reports.",
    flow_id: "simple-flow",
    status: "delivered",
    current_stage: null,
    head_revision: 4,
    source: "store",
    updated_at: AGO(1_440),
  },
  {
    wish_id: "wish-0005",
    title: "Documentation refresh",
    description: "Produce documentation under docs/ for the executed flow slices and their evidence.",
    flow_id: "wish-verify",
    status: "delivered",
    current_stage: null,
    head_revision: null,
    source: "legacy",
    updated_at: null,
  },
];

export function getSampleWish(wishId) {
  return SAMPLE_WISHES.find((wish) => wish.wish_id === wishId) || null;
}

/* ------------------------------------------------------------------ */
/* Wish detail                                                        */
/* ------------------------------------------------------------------ */

// stage_id -> what happened in the sample run for each wish.
const PROFILES = {
  "wish-0011": {
    visits: [
      { stage: "requirements", status: "completed", assessments: ["passed"] },
      { stage: "design", status: "completed", assessments: ["passed"] },
      { stage: "implement", status: "completed", assessments: ["passed"] },
      { stage: "verify", status: "completed", assessments: ["passed", "passed"] },
    ],
  },
  "wish-0009": {
    visits: [
      { stage: "requirements", status: "completed", assessments: ["passed"] },
      { stage: "design", status: "completed", assessments: ["passed"] },
      { stage: "implement", status: "completed", assessments: ["passed"] },
      { stage: "verify", status: "completed", assessments: ["passed", "passed"] },
    ],
  },
  "wish-0012": {
    visits: [
      { stage: "requirements", status: "completed", assessments: ["passed"] },
      { stage: "design", status: "active", assessments: [] },
    ],
  },
  "wish-0013": {
    visits: [
      { stage: "requirements", status: "completed", assessments: ["passed"] },
      { stage: "design", status: "completed", assessments: ["passed"] },
      { stage: "implement", status: "blocked", assessments: ["failed"] },
    ],
  },
  "wish-0014": { visits: [] },
  "wish-0006": {
    visits: [
      { stage: "wish", status: "completed", assessments: ["passed"] },
      { stage: "verify", status: "active", assessments: [] },
    ],
  },
  "wish-0007": {
    visits: [
      { stage: "wish", status: "blocked", assessments: ["failed"] },
    ],
  },
  "wish-0005": {
    visits: [
      { stage: "wish", status: "completed", assessments: ["passed"] },
      { stage: "verify", status: "completed", assessments: ["passed"] },
    ],
  },
};

const BINDINGS = {
  "wish-0009": {
    simpleDesignAccepted: { mode: "cli", cli: "codex", model: "gpt-5.6-terra", command_uri: null },
  },
};

const SEMANTIC_ASSESSORS = {
  "wish-0011": { judged_by: "human", assessor: "operator", rationale: "The retained verification evidence and the deterministic gate reports satisfy every acceptance condition; residual risk is limited to the untested recovery path and is accepted." },
  "wish-0005": { judged_by: "human", assessor: "operator", rationale: "Every acceptance condition in the wish is addressed by the docs audit evidence." },
  "wish-0006": { judged_by: "agent", assessor: "codex:gpt-6-astra", rationale: "The evidence set covers both acceptance conditions." },
  "wish-0009": { judged_by: "agent", assessor: "codex:gpt-5.6-terra", rationale: "The CLI-executed design check reports the declared rules and retains its logs." },
  "wish-0013": { judged_by: "agent", assessor: "xdesign web operator", rationale: "Stage blocked: declared checker rules did not pass." },
  "wish-0007": { judged_by: "human", assessor: "operator", rationale: "The wish statement does not state testable acceptance conditions." },
};

const STAGE_ARTIFACTS = {
  requirements: ["mbse/model/requirements.sysml", "cache/web/wishes/%ID%.sysml"],
  design: ["docs/design/flows/simple-flow/definition.md", "mbse/model/design.sysml"],
  implement: ["src/lib.rs", "web/assets/index.html", "web/assets/styles.css"],
  verify: ["mbse/model/verification.sysml", "cache/web/attempts/%ID%-4f2a.json"],
  wish: ["cache/web/wishes/%ID%.sysml"],
};

function stageTitle(flow, stageId) {
  const stage = flow.stages.find((entry) => entry.stage_id === stageId);
  return stage ? stage.title : stageId;
}

function stageCheckpoints(flow, stageId) {
  return flow.checkpoints.filter((entry) => entry.stage_id === stageId);
}

function buildPlan(wish, flow) {
  const overrides = BINDINGS[wish.wish_id] || {};
  return {
    plan_id: flow.flow_id === "simple-flow" ? "designedSimpleFlow" : "designedWishVerifyFlow",
    plan_sha256: digest(`plan:${wish.wish_id}:${flow.flow_id}`),
    stages: flow.stages.map((stage, index) => ({
      stage_id: stage.stage_id,
      next_stage_ids: index + 1 < flow.stages.length ? [flow.stages[index + 1].stage_id] : [],
      exit_checkpoint_ids: stageCheckpoints(flow, stage.stage_id).map((cp) => cp.checkpoint_id),
      completes_flow: index + 1 === flow.stages.length,
    })),
    checkpoints: flow.checkpoints.map((checkpoint) => {
      const binding = checkpoint.checker
        ? overrides[checkpoint.checkpoint_id] || {
            mode: "default",
            cli: null,
            model: null,
            command_uri: null,
          }
        : null;
      return { ...checkpoint, binding };
    }),
  };
}

function buildAssessment(wish, flow, checkpoint, outcome, revision, at, binding) {
  const semantic = checkpoint.kind === "semantic";
  const semanticMeta = SEMANTIC_ASSESSORS[wish.wish_id] || {
    judged_by: "human",
    assessor: "operator",
    rationale: "Evidence reviewed.",
  };
  const cliBound = binding && binding.mode === "cli";
  const judgedBy = semantic ? semanticMeta.judged_by : cliBound ? "agent" : "script";
  const assessor = semantic ? semanticMeta.assessor : cliBound ? `${binding.cli}:${binding.model}` : checkpoint.checker.checker_id;
  const rationale = semantic
    ? outcome === "failed"
      ? semanticMeta.rationale
      : semanticMeta.rationale
    : outcome === "passed"
      ? cliBound
        ? `Agent CLI execution reported all ${checkpoint.checker.rules.length} declared checker rules satisfied; stdout and stderr logs retained.`
        : `All ${checkpoint.checker.rules.length} declared checker rules passed.`
      : `Rule ${checkpoint.checker.rules[0].rule_id} failed: declared artifact is missing from the recorded revision.`;
  return {
    checkpoint_id: checkpoint.checkpoint_id,
    evaluated_revision: revision,
    evaluated_snapshot_digest: digest(`snapshot:${wish.wish_id}:${checkpoint.checkpoint_id}:${revision}`),
    outcome,
    judged_by: judgedBy,
    assessor,
    rationale,
    evidence_uris: [
      `cache/xdesign/${wish.wish_id}/reports/${checkpoint.title}_${revision}_${digest(`${wish.wish_id}:${checkpoint.checkpoint_id}`).slice(0, 12)}.json`,
      ...(semantic ? [`cache/xdesign/${wish.wish_id}/evidence/review_${revision}.md`] : []),
    ],
  };
}

function activityPerformedBy(stageId) {
  const map = {
    requirements: "xdesign simple-flow caller",
    design: "xdesign web operator",
    implement: "xdesign simple-flow caller",
    verify: "xdesign check caller",
    wish: "xdesign wish-verify caller",
  };
  return map[stageId] || "xdesign operator";
}

function buildActivities(wish, flow, stageId, revision, at) {
  const artifacts = (STAGE_ARTIFACTS[stageId] || []).map((uri) => uri.replace("%ID%", wish.wish_id));
  return [
    {
      description: `Record ${stageTitle(flow, stageId).toLowerCase()} stage input artifacts`,
      performed_by: activityPerformedBy(stageId),
      occurred_at: at,
      artifact_uris: artifacts,
      artifacts_changed: true,
      artifact_digests: artifacts.map((uri) => ({ uri, sha256: RISK(`${wish.wish_id}:${stageId}:${uri}:${revision}`) })),
      removed_artifact_uris: [],
    },
  ];
}

function buildDetail(wish) {
  const flow = getSampleFlow(wish.flow_id);
  const plan = buildPlan(wish, flow);
  const profile = PROFILES[wish.wish_id] || { visits: [] };
  const baseMinutes = 60 * 30;
  let revision = 0;
  const visits = [];
  const events = [];

  events.push({
    kind: "init",
    stage_id: null,
    visit_index: null,
    title: "Wish initialized",
    detail: `flow ${flow.flow_id}`,
    at: wish.updated_at || AGO(baseMinutes),
    actor: "xdesign init",
    outcome: null,
    evidence_uris: [],
  });

  profile.visits.forEach((visitProfile, visitIndex) => {
    const stage = flow.stages.find((entry) => entry.stage_id === visitProfile.stage);
    const checkpoints = stageCheckpoints(flow, visitProfile.stage);
    revision += 1;
    const at = AGO(baseMinutes - visitIndex * 45);
    const activities = buildActivities(wish, flow, visitProfile.stage, revision, at);
    const assessments = [];
    checkpoints.forEach((checkpoint, index) => {
      const outcome = visitProfile.assessments[index];
      if (!outcome) return;
      const binding = plan.checkpoints.find((entry) => entry.checkpoint_id === checkpoint.checkpoint_id)?.binding;
      assessments.push(buildAssessment(wish, flow, checkpoint, outcome, revision, at, binding));
    });

    const visit = {
      stage_id: visitProfile.stage,
      visit_index: 0,
      status: visitProfile.status,
      artifact_revision: revision,
      artifact_snapshot_digest: digest(`snapshot:${wish.wish_id}:${visitProfile.stage}:${revision}`),
      artifact_digests: activities.flatMap((activity) => activity.artifact_digests),
      activities,
      assessments,
    };
    visits.push(visit);

    for (const activity of activities) {
      events.push({
        kind: "activity",
        stage_id: visitProfile.stage,
        visit_index: 0,
        title: activity.description,
        detail: `${activity.artifact_uris.length} artifact${activity.artifact_uris.length === 1 ? "" : "s"}`,
        at: activity.occurred_at,
        actor: activity.performed_by,
        outcome: null,
        evidence_uris: [],
      });
    }
    for (const assessment of assessments) {
      events.push({
        kind: "assessment",
        stage_id: visitProfile.stage,
        visit_index: 0,
        title: assessment.assessor,
        detail: assessment.rationale,
        at,
        actor: assessment.assessor,
        outcome: assessment.outcome,
        evidence_uris: assessment.evidence_uris,
      });
    }
    if (visitProfile.status === "completed") {
      const nextStage = flow.stages[flow.stages.findIndex((entry) => entry.stage_id === visitProfile.stage) + 1];
      events.push({
        kind: "advance",
        stage_id: visitProfile.stage,
        visit_index: 0,
        title: nextStage ? `Entered ${nextStage.title.toLowerCase()}` : "Run completed",
        detail: null,
        at,
        actor: "xdesign",
        outcome: null,
        evidence_uris: [],
      });
    }
  });

  const run =
    visits.length > 0 || !["received"].includes(wish.status)
      ? {
          run_id: digest(`run:${wish.wish_id}`).slice(0, 32),
          execution_type: "FlowRun",
          plan_id: plan.plan_id,
          status:
            wish.status === "delivered"
              ? "completed"
              : wish.status === "blocked"
                ? "blocked"
                : visits.length
                  ? "active"
                  : "active",
          current_visit: Math.max(0, visits.length - 1),
          visits,
        }
      : {
          run_id: digest(`run:${wish.wish_id}`).slice(0, 32),
          execution_type: "FlowRun",
          plan_id: plan.plan_id,
          status: "active",
          current_visit: 0,
          visits: [],
        };

  const history = [];
  if (wish.source === "store") {
    const head = wish.head_revision ?? 1;
    for (let entry = 1; entry <= head; entry += 1) {
      history.push({
        revision: entry,
        operation_id: digest(`op:${wish.wish_id}:${entry}`).slice(0, 24),
        material_count: 6 + entry * 3,
      });
    }
  }

  return {
    wish_id: wish.wish_id,
    title: wish.title,
    description: wish.description,
    flow_id: wish.flow_id,
    status: wish.status,
    current_stage: wish.current_stage,
    head_revision: wish.head_revision,
    source: wish.source,
    plan,
    run,
    history,
    events,
  };
}

export function getSampleDetail(wishId) {
  const wish = getSampleWish(wishId);
  if (!wish) return null;
  return buildDetail(wish);
}

/* ------------------------------------------------------------------ */
/* Work response                                                      */
/* ------------------------------------------------------------------ */

export function getSampleWork(wishId) {
  const wish = getSampleWish(wishId);
  const detail = getSampleDetail(wishId);
  if (!wish || !detail) return null;
  const flow = getSampleFlow(wish.flow_id);
  const stageId = wish.current_stage || flow.stages[0].stage_id;
  const stage = flow.stages.find((entry) => entry.stage_id === stageId) || flow.stages[0];
  const exitCheckpoints = stageCheckpoints(flow, stageId).map((cp) => cp.checkpoint_id);
  const completed = wish.status === "delivered";
  const response = {
    schema_version: 1,
    wish_id: wish.wish_id,
    run_id: detail.run.run_id,
    flow_id: flow.flow_id,
    status: wish.status === "received" ? "active" : wish.status,
    completed,
    state_token: digest(`state:${wish.wish_id}`).slice(0, 40),
    store_revision: wish.head_revision ?? 0,
    work: completed
      ? null
      : {
          stage_id: stage.stage_id,
          visit_index: 0,
          artifact_revision: detail.run.current_visit,
          artifact_snapshot_digest: digest(`snapshot:${wish.wish_id}:${stage.stage_id}:${detail.run.current_visit}`),
          goal: stage.goal,
          instructions: stage.instructions,
          inputs: ["wish source", "frozen plan"],
          input_artifacts: (STAGE_ARTIFACTS[stage.stage_id] || []).map((uri) => uri.replace("%ID%", wish.wish_id)),
          expected_outputs: stage.expected_outputs,
          frozen_output_paths: [],
          exit_checkpoints: exitCheckpoints,
          current_assessments:
            wish.status === "blocked"
              ? [
                  {
                    checkpoint_id: exitCheckpoints[0],
                    outcome: "failed",
                    assessor: "simpleImplementationCheck",
                    rationale: "Required artifact web/assets/app.js is missing from the recorded revision.",
                  },
                ]
              : [],
          correction_reasons:
            wish.status === "blocked"
              ? [
                  "simpleImplementationAccepted failed: required artifact web/assets/app.js is missing from the recorded revision.",
                  "Record the implementation artifacts again after the page module is restored.",
                ]
              : [],
          collection_diagnostics: [],
          candidate_artifacts: [],
          candidate_snapshot_digest: digest(`candidate:${wish.wish_id}`),
        },
  };
  return {
    exit_code: 0,
    response,
    stdout: JSON.stringify(response, null, 2),
    stderr: "",
  };
}

/* ------------------------------------------------------------------ */
/* Check response                                                     */
/* ------------------------------------------------------------------ */

export function getSampleCheckResponse(wishId, body) {
  const wish = getSampleWish(wishId);
  const detail = getSampleDetail(wishId);
  if (!wish || !detail) return null;
  const flow = getSampleFlow(wish.flow_id);
  const stageId = wish.current_stage || flow.stages[0].stage_id;
  const semantic = flow.checkpoints.find((cp) => cp.stage_id === stageId && cp.kind === "semantic");
  const syntactic = flow.checkpoints.find((cp) => cp.stage_id === stageId && cp.kind === "syntactic");
  const judgment = body && body.judgment;
  const passed = judgment ? judgment.outcome === "passed" : true;
  const assessed = {
    checkpoint_id: semantic ? semantic.checkpoint_id : syntactic ? syntactic.checkpoint_id : "unknown",
    evaluated_revision: (wish.head_revision ?? 0) + 1,
    evaluated_snapshot_digest: digest(`check:${wish.wish_id}:${(wish.head_revision ?? 0) + 1}`),
    outcome: judgment ? judgment.outcome : "passed",
    judged_by: judgment ? judgment.authority : "script",
    assessor: judgment ? judgment.identity : syntactic ? syntactic.checker.checker_id : "xdesign web operator",
    rationale: judgment
      ? judgment.rationale
      : "All declared checker rules passed for the recorded revision.",
    evidence_uris: [
      `cache/xdesign/${wish.wish_id}/reports/${assessedCheckpoint(stageId, semantic, syntactic)}_${(wish.head_revision ?? 0) + 1}_web.json`,
    ],
  };
  const response = {
    assessed,
    advance: passed ? "enter_next_stage" : "remain",
    stage_id: stageId,
    attempt: {
      description: body?.description || `Web check, stage ${stageId}`,
      performed_by: body?.performed_by || "xdesign web operator",
      artifacts: (body?.artifacts || []).map((artifact) => artifact.uri),
    },
    next_stage_id: nextStageId(flow, stageId),
    correction_reasons: passed
      ? []
      : ["The supplied judgment did not accept the evidence set; the stage remains blocked for correction."],
  };
  const exitCode = passed ? 0 : 1;
  return {
    exit_code: exitCode,
    response,
    stdout: JSON.stringify(response, null, 2),
    stderr: "",
  };
}

function assessedCheckpoint(stageId, semantic, syntactic) {
  if (semantic) return semantic.checkpoint_id;
  if (syntactic) return syntactic.title;
  return `${stageId}Check`;
}

function nextStageId(flow, stageId) {
  const index = flow.stages.findIndex((stage) => stage.stage_id === stageId);
  const next = flow.stages[index + 1];
  return next ? next.stage_id : null;
}

/* ------------------------------------------------------------------ */
/* Create response (mock server only)                                 */
/* ------------------------------------------------------------------ */

export function getSampleInitResponse(body) {
  return {
    wish_id: body.wish_id,
    flow_id: body.flow_id,
    init: {
      schema_version: 1,
      wish_id: body.wish_id,
      store_revision: 1,
      flow_id: body.flow_id,
      checks: (body.checks || []).map((entry) => ({ ...entry })),
    },
    stdout: JSON.stringify(
      {
        schema_version: 1,
        wish_id: body.wish_id,
        store_revision: 1,
        flow_id: body.flow_id,
      },
      null,
      2,
    ),
    stderr: "",
  };
}

/* ------------------------------------------------------------------ */
/* Material viewer                                                    */
/* ------------------------------------------------------------------ */

export function getSampleMaterial(wishId, uri) {
  const wish = getSampleWish(wishId);
  const detail = wish ? getSampleDetail(wishId) : null;
  if (!wish || !detail) return null;
  const flow = getSampleFlow(wish.flow_id);

  if (uri.endsWith(".json")) {
    const checkpoint = flow.checkpoints.find((cp) => uri.includes(cp.title) || uri.includes(cp.checkpoint_id));
    const assessed = detail.run.visits.flatMap((visit) => visit.assessments).find((assessment) => assessment.checkpoint_id === checkpoint?.checkpoint_id);
    return {
      contentType: "application/json; charset=utf-8",
      text: JSON.stringify(
        {
          schema_version: 1,
          report_kind: checkpoint?.kind === "semantic" ? "agent_assessment" : "checker_report",
          wish_id: wish.wish_id,
          flow_id: wish.flow_id,
          plan_id: detail.plan.plan_id,
          checkpoint_id: checkpoint?.checkpoint_id || "unknown",
          checker_id: checkpoint?.checker?.checker_id || null,
          evaluation: {
            outcome: assessed?.outcome || "passed",
            evaluated_revision: assessed?.evaluated_revision ?? 1,
            judged_by: assessed?.judged_by || "script",
            assessor: assessed?.assessor || "xdesign",
            rationale: assessed?.rationale || "Sample evidence retained for the demo dataset.",
            evaluated_at: assessed ? AGO(120) : AGO(60),
          },
          rules: (checkpoint?.checker?.rules || []).map((rule) => ({
            rule_id: rule.rule_id,
            outcome: "passed",
            condition: rule.condition,
            reads: rule.reads,
          })),
          artifacts: [],
          notice: "Sample material generated by the WishFlow Studio demo dataset.",
        },
        null,
        2,
      ),
    };
  }

  if (uri.endsWith(".md")) {
    return {
      contentType: "text/markdown; charset=utf-8",
      text: `# ${wish.title}

Sample material retained for the demo dataset.

- **Wish**: \`${wish.wish_id}\`
- **Flow**: \`${wish.flow_id}\`
- **Plan**: \`${detail.plan.plan_id}\`
- **Plan digest**: \`${detail.plan.plan_sha256}\`
- **URI**: \`${uri}\`

## Evidence notes

The retained set shows the stage evidence recorded for the frozen plan. Each
assessment names its authority and the checker or executor that produced it.
`,
    };
  }

  if (uri.endsWith(".sysml")) {
    return {
      contentType: "text/plain; charset=utf-8",
      text: `package Wish_${wish.wish_id.replace(/[^A-Za-z0-9]/g, "_")} {
    doc /*
    ${wish.title}

    ${wish.description || ""}
    */

    requirement ${wish.wish_id.replace(/-/g, "_")} {
        attribute rationale = "Sample source retained for the demo dataset";
        attribute flow = "${wish.flow_id}";
    }
}
`,
    };
  }

  return {
    contentType: "application/json; charset=utf-8",
    text: JSON.stringify(
      {
        uri,
        wish_id: wish.wish_id,
        material: "Sample material for the demo dataset.",
        sample: true,
      },
      null,
      2,
    ),
  };
}
