//! API assembly for the embedded operator web program.
//!
//! Every handler receives the process-local [`AppState`] and returns either a
//! JSON success envelope or an [`ApiError`]. The module reads the operational
//! `.xdesign` store through [`crate::storage::RsStore`] and drives lifecycle
//! actions by running the same `xdesign` executable as a subprocess; it never
//! opens the store for writing itself.

use crate::core::{hash_artifact, RsCheckpointSpec, RsFlowPlan};
use crate::simple_flow::designed_simple_flow_plan;
use crate::storage::{RsItemRevision, RsMaterial, RsStore, RsStoreError, RsWishStatus};
use crate::wish_verify::wish_verify_flow_plan;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::http::{self, Method, Request, Response};

/// Largest material payload served by the material endpoint (512 KiB).
const MAX_MATERIAL_BYTES: u64 = 512 * 1024;
/// Bound on one lifecycle subprocess: the stage CLI already bounds its
/// checkers, so this only guards against a wedged child.
fn lifecycle_timeout() -> Duration {
    crate::simple_flow::checker_timeout() + Duration::from_secs(120)
}

// ---------------------------------------------------------------------------
// Lifecycle subprocess boundary
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleOutput {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Runs one `xdesign init|next|check` invocation in the project root.
///
/// Injecting this trait keeps API tests independent of the real stage CLI and
/// of this repository's `.xdesign` store.
pub trait Lifecycle: Send + Sync {
    fn run(&self, args: &[String], cwd: &Path) -> Result<LifecycleOutput, String>;
}

/// Production runner: the same executable that is serving this page.
pub struct ProcessLifecycle;

impl Lifecycle for ProcessLifecycle {
    fn run(&self, args: &[String], cwd: &Path) -> Result<LifecycleOutput, String> {
        let executable = std::env::current_exe()
            .map_err(|error| format!("cannot locate the xdesign executable: {error}"))?;
        let mut command = std::process::Command::new(executable);
        command.args(args).current_dir(cwd);
        let outcome =
            crate::simple_flow::run_command_with_timeout(&mut command, lifecycle_timeout())
                .map_err(|error| format!("cannot run xdesign {}: {error}", command_name(args)))?;
        if outcome.timed_out {
            return Err(format!(
                "xdesign {} exceeded its deadline and was killed",
                command_name(args)
            ));
        }
        // `run_command_with_timeout` reports success rather than the number.
        // The stage CLI prints its response envelope and exits 1 only for a
        // blocked check, so a successful parse with `advance=remain_blocked`
        // recovers that documented code.
        let exit_code = if outcome.success {
            0
        } else if blocked_check_output(&outcome.stdout) {
            1
        } else {
            2
        };
        Ok(LifecycleOutput {
            exit_code,
            stdout: outcome.stdout,
            stderr: outcome.stderr,
        })
    }
}

fn command_name(args: &[String]) -> &str {
    args.first().map(String::as_str).unwrap_or("command")
}

fn blocked_check_output(stdout: &[u8]) -> bool {
    serde_json::from_slice::<Value>(stdout)
        .ok()
        .and_then(|value| {
            value
                .get("advance")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .as_deref()
        == Some("remain_blocked")
}

// ---------------------------------------------------------------------------
// Shared state and routing
// ---------------------------------------------------------------------------

pub struct AppState {
    pub project_root: PathBuf,
    pub store_root: PathBuf,
    pub lifecycle: Box<dyn Lifecycle>,
}

impl AppState {
    /// Production state: spawn the current binary for lifecycle actions.
    pub fn new(project_root: PathBuf, store_root: PathBuf) -> Self {
        Self {
            project_root,
            store_root,
            lifecycle: Box::new(ProcessLifecycle),
        }
    }

    /// Structured state for tests: injected lifecycle runner.
    pub fn with_parts(
        project_root: PathBuf,
        store_root: PathBuf,
        lifecycle: Box<dyn Lifecycle>,
    ) -> Self {
        Self {
            project_root,
            store_root,
            lifecycle,
        }
    }
}

#[derive(Debug)]
struct ApiError {
    status: u16,
    code: &'static str,
    message: String,
    detail: Option<Value>,
}

impl ApiError {
    fn new(status: u16, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            detail: None,
        }
    }

    fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(400, code, message)
    }

    fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(404, code, message)
    }

    fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(409, code, message)
    }

    fn internal(message: impl Into<String>) -> Self {
        Self::new(500, "internal_error", message)
    }

    fn method_not_allowed() -> Self {
        Self::new(
            405,
            "method_not_allowed",
            "This route does not accept that method.",
        )
    }

    fn with_detail(mut self, detail: Value) -> Self {
        self.detail = Some(detail);
        self
    }

    fn into_response(self) -> Response {
        match self.detail {
            Some(detail) => {
                http::json_error_with_detail(self.status, self.code, self.message, detail)
            }
            None => http::json_error(self.status, self.code, self.message),
        }
    }
}

type ApiResult = Result<Response, ApiError>;

/// Route one parsed request. API paths answer JSON envelopes; every other path
/// is served from the embedded asset table.
pub fn handle(state: &AppState, request: &Request) -> Response {
    if request.path == "/api" || request.path.starts_with("/api/") {
        match route_api(state, request) {
            Ok(response) => response,
            Err(error) => error.into_response(),
        }
    } else {
        route_asset(request)
    }
}

fn route_api(state: &AppState, request: &Request) -> ApiResult {
    let segments: Vec<&str> = request
        .path
        .trim_start_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    match segments.as_slice() {
        ["api", "health"] => {
            require_method(request, Method::Get)?;
            Ok(health(state))
        }
        ["api", "flows"] => {
            require_method(request, Method::Get)?;
            Ok(flows(state))
        }
        ["api", "wishes"] => match request.method {
            Method::Get => wishes(state),
            Method::Post => create_wish(state, request),
        },
        ["api", "wishes", wish_id] if valid_wish_id(wish_id) => {
            if request.method != Method::Get {
                return Err(ApiError::method_not_allowed());
            }
            wish_detail(state, wish_id)
        }
        ["api", "wishes", wish_id, "work"] if valid_wish_id(wish_id) => {
            require_method(request, Method::Get)?;
            work(state, wish_id)
        }
        ["api", "wishes", wish_id, "check"] if valid_wish_id(wish_id) => {
            require_method(request, Method::Post)?;
            check(state, wish_id, request)
        }
        ["api", "wishes", wish_id, "material"] if valid_wish_id(wish_id) => {
            require_method(request, Method::Get)?;
            material(state, wish_id, request)
        }
        ["api", "checkpoints"] => match request.method {
            Method::Get => checkpoint_list(state),
            Method::Post => checkpoint_save(state, request),
        },
        ["api", "checkpoints", checkpoint_id] if valid_checkpoint_id(checkpoint_id) => {
            require_method(request, Method::Get)?;
            checkpoint_show(state, checkpoint_id)
        }
        _ => Err(ApiError::not_found(
            "not_found",
            format!("Unknown API route: {}", request.path),
        )),
    }
}

fn require_method(request: &Request, expected: Method) -> Result<(), ApiError> {
    if request.method == expected {
        Ok(())
    } else {
        Err(ApiError::method_not_allowed())
    }
}

fn valid_wish_id(wish_id: &str) -> bool {
    let mut bytes = wish_id.bytes();
    wish_id.len() <= 128
        && bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

/// Checkpoint library ids are lower-case slugs per the library model.
fn valid_checkpoint_id(checkpoint_id: &str) -> bool {
    let mut bytes = checkpoint_id.bytes();
    checkpoint_id.len() <= 64
        && bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
}

// ---------------------------------------------------------------------------
// GET /api/health
// ---------------------------------------------------------------------------

fn health(state: &AppState) -> Response {
    let store_present = matches!(open_store(state), Ok(Some(_)));
    http::json_ok(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "project_root": state.project_root.to_string_lossy(),
        "store_root": state.store_root.to_string_lossy(),
        "store_present": store_present,
        "now": utc_now(),
    }))
}

// ---------------------------------------------------------------------------
// GET /api/flows
// ---------------------------------------------------------------------------

struct StageDisplay {
    stage_id: &'static str,
    title: &'static str,
    goal: &'static str,
    instructions: &'static str,
    expected_outputs: &'static [&'static str],
}

/// Stage display text mirrors the CLI work templates in
/// `src/stage_cli/work.rs`; the binary-side templates are not reachable from
/// the library, so the operator catalog keeps a copy of the display strings.
const SIMPLE_FLOW_STAGES: &[StageDisplay] = &[
    StageDisplay {
        stage_id: "requirements",
        title: "Requirements",
        goal: "Define requirements and their rationale.",
        instructions: "State the requirements, rationale and satisfaction relationships in the project model.",
        expected_outputs: &["checker:simpleRequirementsCheck"],
    },
    StageDisplay {
        stage_id: "design",
        title: "Design",
        goal: "Design the solution and its verification.",
        instructions: "Update the SysML design, API contracts and documentation; keep model views and checker declarations consistent.",
        expected_outputs: &["checker:simpleDesignCheck"],
    },
    StageDisplay {
        stage_id: "implement",
        title: "Implementation",
        goal: "Implement the reviewed design.",
        instructions: "Implement the source and tests required by the design; retain source artifacts for checking.",
        expected_outputs: &["source:src"],
    },
    StageDisplay {
        stage_id: "verify",
        title: "Verification",
        goal: "Demonstrate that the implementation satisfies the requirements.",
        instructions: "Run the planned tests, retain verification evidence and supply a permitted semantic judgment.",
        expected_outputs: &["result:artifacts"],
    },
];

const WISH_VERIFY_STAGES: &[StageDisplay] = &[
    StageDisplay {
        stage_id: "wish",
        title: "Wish",
        goal: "Record the wish and the artifacts that answer it.",
        instructions: "Record the requested outcome and its answering artifacts, then supply a permitted semantic judgment.",
        expected_outputs: &["wish:model"],
    },
    StageDisplay {
        stage_id: "verify",
        title: "Verification",
        goal: "Demonstrate that the implementation satisfies the requirements.",
        instructions: "Run the planned tests, retain verification evidence and supply a permitted semantic judgment.",
        expected_outputs: &["result:artifacts"],
    },
];

fn flows(_state: &AppState) -> Response {
    let simple_flow = designed_simple_flow_plan();
    let wish_verify_flow = wish_verify_flow_plan();
    http::json_ok(json!({
        "flows": [
            flow_json(
                "simple-flow",
                "SimpleFlow",
                "Four gated stages: requirements, design, implement, verify.",
                true,
                &simple_flow,
                SIMPLE_FLOW_STAGES,
            ),
            flow_json(
                "wish-verify",
                "WishVerifyFlow",
                "Two judged stages: wish, verify.",
                false,
                &wish_verify_flow,
                WISH_VERIFY_STAGES,
            ),
        ],
    }))
}

fn flow_json(
    flow_id: &str,
    title: &str,
    summary: &str,
    custom_checks_supported: bool,
    plan: &RsFlowPlan,
    stages: &[StageDisplay],
) -> Value {
    let stages: Vec<Value> = stages
        .iter()
        .map(|stage| {
            json!({
                "stage_id": stage.stage_id,
                "title": stage.title,
                "goal": stage.goal,
                "instructions": stage.instructions,
                "expected_outputs": stage.expected_outputs,
            })
        })
        .collect();
    let checkpoints: Vec<Value> = plan
        .checkpoints
        .iter()
        .map(checkpoint_catalog_json)
        .collect();
    json!({
        "flow_id": flow_id,
        "title": title,
        "summary": summary,
        "custom_checks_supported": custom_checks_supported,
        "stages": stages,
        "checkpoints": checkpoints,
    })
}

fn checkpoint_catalog_json(checkpoint: &RsCheckpointSpec) -> Value {
    let title = checkpoint_title_from_spec(checkpoint);
    let checker = checkpoint.checker.as_ref().map(|checker| {
        json!({
            "checker_id": checker.checker_id,
            "command_uri": checker.command_uri,
            "establishes": serde_json::to_value(&checker.establishes).unwrap_or(Value::Null),
            "rules": checker.rules.iter().map(|rule| json!({
                "rule_id": rule.rule_id,
                "condition": rule.condition,
                "reads": rule.reads,
            })).collect::<Vec<_>>(),
            "modes": ["default", "library"],
        })
    });
    json!({
        "checkpoint_id": checkpoint.checkpoint_id,
        "stage_id": checkpoint.stage_id,
        "kind": check_kind_str(checkpoint.kind),
        "title": title,
        "criteria": checkpoint.criteria,
        "agent_may_judge": checkpoint.agent_may_judge,
        "human_may_judge": checkpoint.human_may_judge,
        "checker": checker.unwrap_or(Value::Null),
    })
}

fn checkpoint_title_from_spec(checkpoint: &RsCheckpointSpec) -> String {
    match checkpoint.kind {
        crate::core::RsCheckKind::Syntactic => checkpoint
            .checker
            .as_ref()
            .map(|checker| checker.checker_id.clone())
            .unwrap_or_else(|| checkpoint.checkpoint_id.clone()),
        crate::core::RsCheckKind::Semantic => checkpoint.checkpoint_id.clone(),
    }
}

fn check_kind_str(kind: crate::core::RsCheckKind) -> &'static str {
    match kind {
        crate::core::RsCheckKind::Syntactic => "syntactic",
        crate::core::RsCheckKind::Semantic => "semantic",
    }
}

// ---------------------------------------------------------------------------
// GET /api/wishes
// ---------------------------------------------------------------------------

struct WishListEntry {
    wish_id: String,
    title: String,
    description: Option<String>,
    flow_id: String,
    status: String,
    current_stage: Option<String>,
    head_revision: Option<u64>,
    source: &'static str,
}

impl WishListEntry {
    fn to_json(&self) -> Value {
        json!({
            "wish_id": self.wish_id,
            "title": self.title,
            "description": self.description,
            "flow_id": self.flow_id,
            "status": self.status,
            "current_stage": self.current_stage,
            "head_revision": self.head_revision,
            "source": self.source,
            "updated_at": Value::Null,
        })
    }
}

fn wishes(state: &AppState) -> ApiResult {
    let store = open_store(state)?;
    let mut store_ids = BTreeSet::new();
    let mut entries = Vec::new();
    if let Some(store) = &store {
        for summary in store
            .list(Some("wish"), None)
            .map_err(|error| store_error("cannot list wishes", error))?
        {
            store_ids.insert(summary.item_id.clone());
            let flow_id = summary
                .projection
                .as_ref()
                .map(|projection| projection.flow_id.clone())
                .unwrap_or_else(|| "simple-flow".into());
            let status = summary
                .projection
                .as_ref()
                .map(|projection| wish_status_str(projection.status).to_owned())
                .unwrap_or_else(|| "received".into());
            let current_stage = summary
                .projection
                .as_ref()
                .and_then(|projection| projection.current_stage.clone());
            let source_text =
                store_wish_source(store, &summary.item_id, Some(summary.head_revision));
            let (title, description) =
                wish_title_description(&summary.item_id, source_text.as_deref());
            entries.push(WishListEntry {
                wish_id: summary.item_id,
                title,
                description,
                flow_id,
                status,
                current_stage,
                head_revision: Some(summary.head_revision),
                source: "store",
            });
        }
    }
    for (wish_id, directory) in legacy_registrations(&state.project_root) {
        if store_ids.contains(&wish_id) {
            continue;
        }
        let Some(registry) = read_registry_file(&directory.join("registry.json")) else {
            continue;
        };
        if !matches!(registry.flow_id.as_str(), "simple-flow" | "wish-verify") {
            continue;
        }
        let run = fs::read_to_string(directory.join("run.json"))
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok());
        let run_status = run
            .as_ref()
            .and_then(|run| run.get("status"))
            .and_then(Value::as_str)
            .unwrap_or("pending");
        let status = map_run_status(run_status);
        let current_stage = if status == "delivered" {
            None
        } else {
            run.as_ref().and_then(current_stage_of_run)
        };
        let source_text = read_legacy_wish_source(&state.project_root, &registry.wish_model_uri);
        let (title, description) = wish_title_description(&wish_id, source_text.as_deref());
        entries.push(WishListEntry {
            wish_id,
            title,
            description,
            flow_id: registry.flow_id,
            status: status.into(),
            current_stage,
            head_revision: None,
            source: "legacy",
        });
    }
    entries.sort_by(|left, right| {
        status_priority(&left.status)
            .cmp(&status_priority(&right.status))
            .then_with(|| left.wish_id.cmp(&right.wish_id))
    });
    let wishes = entries
        .iter()
        .map(WishListEntry::to_json)
        .collect::<Vec<_>>();
    Ok(http::json_ok(json!({"wishes": wishes})))
}

fn status_priority(status: &str) -> u8 {
    match status {
        "blocked" => 0,
        "active" => 1,
        "received" => 2,
        _ => 3,
    }
}

fn wish_status_str(status: RsWishStatus) -> &'static str {
    match status {
        RsWishStatus::Received => "received",
        RsWishStatus::Active => "active",
        RsWishStatus::Blocked => "blocked",
        RsWishStatus::Delivered => "delivered",
    }
}

fn map_run_status(status: &str) -> &'static str {
    match status {
        "active" => "active",
        "blocked" => "blocked",
        "completed" => "delivered",
        _ => "received",
    }
}

/// Legacy cache-backed registrations: writable `cache/xdesign/<wish_id>/`
/// directories carrying a `registry.json`. Read-only for the web program.
fn legacy_registrations(project_root: &Path) -> Vec<(String, PathBuf)> {
    let directory = project_root.join("cache/xdesign");
    let Ok(entries) = fs::read_dir(&directory) else {
        return Vec::new();
    };
    let mut registrations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(wish_id) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !valid_wish_id(wish_id) || !path.join("registry.json").is_file() {
            continue;
        }
        registrations.push((wish_id.to_owned(), path));
    }
    registrations.sort();
    registrations
}

fn current_stage_of_run(run: &Value) -> Option<String> {
    let current_visit = run.get("current_visit").and_then(Value::as_u64)? as usize;
    run.get("visits")?
        .as_array()?
        .get(current_visit)?
        .get("stage_id")?
        .as_str()
        .map(str::to_owned)
}

// ---------------------------------------------------------------------------
// GET /api/wishes/{wish_id}
// ---------------------------------------------------------------------------

struct LoadedWish {
    flow_id: String,
    status: String,
    current_stage: Option<String>,
    head_revision: Option<u64>,
    source: &'static str,
    plan: Value,
    run: Value,
    history: Vec<Value>,
    source_text: Option<String>,
}

fn load_wish(state: &AppState, wish_id: &str) -> Result<LoadedWish, ApiError> {
    if let Some(store) = open_store(state)? {
        match store.get(wish_id, None) {
            Ok(head) => return load_store_wish(&store, wish_id, &head),
            Err(RsStoreError::NotFound(_)) => {}
            Err(error) => {
                return Err(store_error(
                    &format!("cannot read wish {wish_id} from the store"),
                    error,
                ))
            }
        }
    }
    if state
        .project_root
        .join("cache/xdesign")
        .join(wish_id)
        .join("registry.json")
        .is_file()
    {
        return load_legacy_wish(state, wish_id);
    }
    Err(ApiError::not_found(
        "unknown_wish",
        format!("No wish with id {wish_id} exists in the store or the legacy cache."),
    ))
}

fn load_store_wish(
    store: &RsStore,
    wish_id: &str,
    head: &RsItemRevision,
) -> Result<LoadedWish, ApiError> {
    let registry_uri = head
        .metadata
        .get("registry_uri")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| material_by_role(head, "registration").map(|material| material.uri.clone()))
        .ok_or_else(|| {
            ApiError::internal(format!(
                "stored wish {wish_id} has no registration material"
            ))
        })?;
    let registry_bytes = read_head_material(store, head, &registry_uri)?;
    let registry: RegistryView = serde_json::from_slice(&registry_bytes).map_err(|error| {
        ApiError::internal(format!(
            "stored registration for {wish_id} is invalid: {error}"
        ))
    })?;
    let plan_bytes = read_head_material(store, head, &registry.plan_uri).or_else(|_| {
        head_uri_by_role(head, "frozen-plan")
            .ok_or_else(|| ApiError::internal("frozen plan material is missing".to_owned()))
            .and_then(|uri| read_head_material(store, head, &uri))
    })?;
    let run_bytes = read_head_material(store, head, &registry.run_record_uri).or_else(|_| {
        head_uri_by_role(head, "run-record")
            .ok_or_else(|| ApiError::internal("run record material is missing".to_owned()))
            .and_then(|uri| read_head_material(store, head, &uri))
    })?;
    let source_text = read_head_material(store, head, &registry.wish_model_uri)
        .ok()
        .or_else(|| {
            head_uri_by_role(head, "wish-source")
                .and_then(|uri| read_head_material(store, head, &uri).ok())
        })
        .and_then(|bytes| String::from_utf8(bytes).ok());
    let plan = plan_projection(&plan_bytes, &registry.plan_sha256)?;
    let run = run_projection(&run_bytes)?;
    let history = store
        .history(wish_id)
        .map(|revisions| {
            revisions
                .into_iter()
                .map(|revision| {
                    json!({
                        "revision": revision.revision,
                        "operation_id": revision.operation_id,
                        "material_count": revision.materials.len(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let projection = head.projection.as_ref();
    let status = projection
        .map(|projection| wish_status_str(projection.status).to_owned())
        .or_else(|| {
            run.get("status")
                .and_then(Value::as_str)
                .map(|status| map_run_status(status).to_owned())
        })
        .unwrap_or_else(|| "received".into());
    Ok(LoadedWish {
        flow_id: registry.flow_id,
        status,
        current_stage: projection.and_then(|projection| projection.current_stage.clone()),
        head_revision: Some(head.revision),
        source: "store",
        plan,
        run,
        history,
        source_text,
    })
}

fn load_legacy_wish(state: &AppState, wish_id: &str) -> Result<LoadedWish, ApiError> {
    let directory = state.project_root.join("cache/xdesign").join(wish_id);
    let registry = read_registry_file(&directory.join("registry.json")).ok_or_else(|| {
        ApiError::not_found(
            "unknown_wish",
            format!("No wish with id {wish_id} exists in the store or the legacy cache."),
        )
    })?;
    let plan_bytes = fs::read(state.project_root.join(&registry.plan_uri)).map_err(|error| {
        ApiError::internal(format!(
            "legacy plan {} is unreadable: {error}",
            registry.plan_uri
        ))
    })?;
    let run_bytes =
        fs::read(state.project_root.join(&registry.run_record_uri)).map_err(|error| {
            ApiError::internal(format!(
                "legacy run record {} is unreadable: {error}",
                registry.run_record_uri
            ))
        })?;
    let plan = plan_projection(&plan_bytes, &registry.plan_sha256)?;
    let run = run_projection(&run_bytes)?;
    let status = run
        .get("status")
        .and_then(Value::as_str)
        .map(map_run_status)
        .unwrap_or("received")
        .to_owned();
    let current_stage = if status == "delivered" {
        None
    } else {
        current_stage_of_run(&run)
    };
    Ok(LoadedWish {
        flow_id: registry.flow_id,
        status,
        current_stage,
        head_revision: None,
        source: "legacy",
        plan,
        run,
        history: Vec::new(),
        source_text: read_legacy_wish_source(&state.project_root, &registry.wish_model_uri),
    })
}

fn wish_detail(state: &AppState, wish_id: &str) -> ApiResult {
    let mut wish = load_wish(state, wish_id)?;
    let warnings = attach_library_summaries(state, &mut wish.plan);
    let (title, description) = wish_title_description(wish_id, wish.source_text.as_deref());
    let events = derive_events(&wish.plan, &wish.run, &wish.flow_id);
    Ok(http::json_ok(json!({
        "wish_id": wish_id,
        "title": title,
        "description": description,
        "flow_id": wish.flow_id,
        "status": wish.status,
        "current_stage": wish.current_stage,
        "head_revision": wish.head_revision,
        "source": wish.source,
        "plan": wish.plan,
        "run": wish.run,
        "history": wish.history,
        "events": events,
        "warnings": warnings,
    })))
}

/// Join the frozen library ids of a plan with the *current* project library so
/// the page can show a display title/version for each bound checkpoint.
///
/// The frozen plan (and its recorded rules/criteria) is always authoritative;
/// this join is display-only. The library is read through one best-effort
/// `xdesign checkpoint list` subprocess, so a missing or malformed library
/// never fails the detail response: unresolved checkpoints get `library: null`
/// and the reason is reported in the top-level `warnings` array.
fn attach_library_summaries(state: &AppState, plan: &mut Value) -> Vec<String> {
    let mut bound_ids: Vec<String> = Vec::new();
    if let Some(checkpoints) = plan.get("checkpoints").and_then(Value::as_array) {
        for checkpoint in checkpoints {
            if let Some(library_id) = checkpoint
                .get("library_id")
                .and_then(Value::as_str)
                .filter(|library_id| !library_id.is_empty())
            {
                if !bound_ids.iter().any(|bound| bound == library_id) {
                    bound_ids.push(library_id.to_owned());
                }
            }
        }
    }
    let mut warnings = Vec::new();
    let mut summaries: BTreeMap<String, Value> = BTreeMap::new();
    if !bound_ids.is_empty() {
        match fetch_library_entries(state) {
            Ok(entries) => {
                for entry in entries {
                    if let Some(checkpoint_id) =
                        entry.get("checkpoint_id").and_then(Value::as_str)
                    {
                        summaries.insert(checkpoint_id.to_owned(), library_summary(&entry));
                    }
                }
            }
            Err(message) => warnings.push(format!(
                "Cannot read the checkpoint library ({message}); frozen library checkpoints {} cannot be resolved.",
                bound_ids.join(", ")
            )),
        }
    }
    if let Some(checkpoints) = plan.get_mut("checkpoints").and_then(Value::as_array_mut) {
        for checkpoint in checkpoints {
            let library_id = checkpoint
                .get("library_id")
                .and_then(Value::as_str)
                .filter(|library_id| !library_id.is_empty())
                .map(str::to_owned);
            let Some(library_id) = library_id else {
                checkpoint["library"] = Value::Null;
                continue;
            };
            match summaries.get(&library_id) {
                Some(summary) => checkpoint["library"] = summary.clone(),
                None => {
                    checkpoint["library"] = Value::Null;
                    if warnings.is_empty() || !warnings[0].starts_with("Cannot read") {
                        warnings.push(format!(
                            "Frozen library checkpoint {library_id} is not present in the current project library."
                        ));
                    }
                }
            }
        }
    }
    warnings
}

/// One `checkpoint list` invocation, parsed into its checkpoint entries.
fn fetch_library_entries(state: &AppState) -> Result<Vec<Value>, String> {
    let output = run_lifecycle(state, &["checkpoint", "list"])?;
    if output.exit_code != 0 {
        return Err(lifecycle_message(&output, "xdesign checkpoint list failed"));
    }
    let reply: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("xdesign checkpoint list returned invalid JSON: {error}"))?;
    Ok(reply
        .get("checkpoints")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

/// The display projection of one current library entry.
fn library_summary(entry: &Value) -> Value {
    json!({
        "checkpoint_id": entry.get("checkpoint_id").cloned().unwrap_or(Value::Null),
        "title": entry.get("title").cloned().unwrap_or(Value::Null),
        "kind": entry.get("kind").cloned().unwrap_or(Value::Null),
        "version": entry.get("version").cloned().unwrap_or(Value::Null),
        "builtin": entry.get("builtin").cloned().unwrap_or(Value::Bool(false)),
    })
}

// ---------------------------------------------------------------------------
// Plan and run projections
// ---------------------------------------------------------------------------

fn plan_projection(plan_bytes: &[u8], plan_sha256: &str) -> Result<Value, ApiError> {
    let frozen: Value = serde_json::from_slice(plan_bytes)
        .map_err(|error| ApiError::internal(format!("frozen plan is not valid JSON: {error}")))?;
    let stages: Vec<Value> = frozen
        .get("stages")
        .and_then(Value::as_array)
        .map(|stages| {
            stages
                .iter()
                .map(|stage| {
                    json!({
                        "stage_id": stage.get("stage_id").cloned().unwrap_or(Value::Null),
                        "next_stage_ids": stage.get("next_stage_ids").cloned().unwrap_or_else(|| json!([])),
                        "exit_checkpoint_ids": stage.get("exit_checkpoint_ids").cloned().unwrap_or_else(|| json!([])),
                        "completes_flow": stage.get("completes_flow").cloned().unwrap_or(Value::Bool(false)),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let checkpoints: Vec<Value> = frozen
        .get("checkpoints")
        .and_then(Value::as_array)
        .map(|checkpoints| checkpoints.iter().map(checkpoint_detail_json).collect())
        .unwrap_or_default();
    Ok(json!({
        "plan_id": frozen.get("plan_id").cloned().unwrap_or(Value::Null),
        "plan_sha256": plan_sha256,
        "stages": stages,
        "checkpoints": checkpoints,
    }))
}

fn checkpoint_detail_json(checkpoint: &Value) -> Value {
    let checkpoint_id = checkpoint
        .get("checkpoint_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let kind = checkpoint
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let checker = checkpoint
        .get("checker")
        .filter(|checker| !checker.is_null());
    // A frozen library binding is recorded on the checkpoint itself
    // (`checkpoint.library`), independent of the checker executor the entry
    // declared; the frozen plan stays authoritative over the current library.
    let library_id = checkpoint
        .get("library")
        .and_then(Value::as_str)
        .filter(|library_id| !library_id.is_empty())
        .map(str::to_owned);
    let library_binding = library_id
        .as_ref()
        .map(|library_id| json!({"mode": "library", "library_id": library_id}));
    let title = if kind == "syntactic" {
        checker
            .and_then(|checker| checker.get("checker_id"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| checkpoint_id.clone())
    } else {
        checkpoint_id.clone()
    };
    let checker_json = checker.map(|checker| {
        json!({
            "checker_id": checker.get("checker_id").cloned().unwrap_or(Value::Null),
            "command_uri": checker.get("command_uri").cloned().unwrap_or(Value::Null),
            "establishes": checker.get("establishes").cloned().unwrap_or_else(|| json!([])),
            "rules": checker.get("rules").cloned().unwrap_or_else(|| json!([])),
            "binding": library_binding
                .clone()
                .unwrap_or_else(|| derive_binding(checker)),
        })
    });
    let mut projected = json!({
        "checkpoint_id": checkpoint_id,
        "stage_id": checkpoint.get("stage_id").cloned().unwrap_or(Value::Null),
        "kind": kind,
        "title": title,
        "criteria": checkpoint.get("criteria").cloned().unwrap_or_else(|| json!([])),
        "agent_may_judge": checkpoint.get("agent_may_judge").cloned().unwrap_or(Value::Bool(false)),
        "human_may_judge": checkpoint.get("human_may_judge").cloned().unwrap_or(Value::Bool(false)),
        "library_id": library_id,
        "checker": checker_json.unwrap_or(Value::Null),
    });
    // Semantic checkpoints have no checker object to host a binding; expose the
    // frozen library binding on the checkpoint itself for a uniform read path.
    if let Some(library_binding) = library_binding {
        projected["binding"] = library_binding;
    }
    projected
}

/// Binding derived from the frozen plan checkpoint. The CLI freezes the chosen
/// executor into the plan so the page can reconstruct it without a side file.
fn derive_binding(checker: &Value) -> Value {
    match checker.get("custom") {
        None | Some(Value::Null) => json!({"mode": "default"}),
        Some(custom) => match custom.get("runner").and_then(Value::as_str) {
            Some("cli") => json!({
                "mode": "cli",
                "cli": custom.get("cli").cloned().unwrap_or(Value::Null),
                "model": custom.get("model").cloned().unwrap_or(Value::Null),
            }),
            Some("process") => json!({
                "mode": "command",
                "command_uri": checker.get("command_uri").cloned().unwrap_or(Value::Null),
            }),
            _ => json!({"mode": "default"}),
        },
    }
}

fn run_projection(run_bytes: &[u8]) -> Result<Value, ApiError> {
    let run: Value = serde_json::from_slice(run_bytes)
        .map_err(|error| ApiError::internal(format!("run record is not valid JSON: {error}")))?;
    Ok(json!({
        "run_id": run.get("run_id").cloned().unwrap_or(Value::Null),
        "execution_type": run.get("execution_type").cloned().unwrap_or(Value::Null),
        "plan_id": run.get("plan_id").cloned().unwrap_or(Value::Null),
        "status": run.get("status").cloned().unwrap_or(Value::Null),
        "current_visit": run.get("current_visit").cloned().unwrap_or(Value::Null),
        "visits": run.get("visits").cloned().unwrap_or_else(|| json!([])),
    }))
}

/// Derive the page's event timeline from the visit order: one `init` event,
/// then per visit its activities, its assessments, and an `advance` event when
/// the visit completed.
fn derive_events(plan: &Value, run: &Value, flow_id: &str) -> Vec<Value> {
    let mut events = vec![json!({
        "kind": "init",
        "stage_id": Value::Null,
        "visit_index": Value::Null,
        "title": "Wish initialized",
        "detail": format!("flow {flow_id}"),
        "at": Value::Null,
        "actor": "xdesign init",
        "outcome": Value::Null,
        "evidence_uris": [],
    })];
    let Some(visits) = run.get("visits").and_then(Value::as_array) else {
        return events;
    };
    let run_status = run
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default();
    for visit in visits {
        let stage_id = visit.get("stage_id").cloned().unwrap_or(Value::Null);
        let visit_index = visit.get("visit_index").cloned().unwrap_or(Value::Null);
        for activity in visit
            .get("activities")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let artifact_count = activity
                .get("artifact_uris")
                .and_then(Value::as_array)
                .map(|uris| uris.len())
                .unwrap_or(0);
            let detail = match artifact_count {
                0 => Value::Null,
                1 => Value::String("1 artifact".into()),
                count => Value::String(format!("{count} artifacts")),
            };
            events.push(json!({
                "kind": "activity",
                "stage_id": stage_id,
                "visit_index": visit_index,
                "title": activity.get("description").cloned().unwrap_or(Value::Null),
                "detail": detail,
                "at": activity.get("occurred_at").cloned().unwrap_or(Value::Null),
                "actor": activity.get("performed_by").cloned().unwrap_or(Value::Null),
                "outcome": Value::Null,
                "evidence_uris": [],
            }));
        }
        for assessment in visit
            .get("assessments")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let checkpoint_id = assessment
                .get("checkpoint_id")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let rationale = assessment
                .get("rationale")
                .and_then(Value::as_str)
                .filter(|rationale| !rationale.is_empty())
                .map(|rationale| Value::String(rationale.to_owned()))
                .unwrap_or(Value::Null);
            events.push(json!({
                "kind": "assessment",
                "stage_id": stage_id,
                "visit_index": visit_index,
                "title": checkpoint_title(plan, checkpoint_id),
                "detail": rationale,
                "at": Value::Null,
                "actor": assessment.get("assessor").cloned().unwrap_or(Value::Null),
                "outcome": assessment.get("outcome").cloned().unwrap_or(Value::Null),
                "evidence_uris": assessment.get("evidence_uris").cloned().unwrap_or_else(|| json!([])),
            }));
        }
        if visit.get("status").and_then(Value::as_str) == Some("completed") {
            let stage = stage_id.as_str().unwrap_or_default();
            if let Some(next_stage) = next_stage_id(plan, stage) {
                events.push(json!({
                    "kind": "advance",
                    "stage_id": stage_id,
                    "visit_index": visit_index,
                    "title": format!("Entered {next_stage}"),
                    "detail": Value::Null,
                    "at": Value::Null,
                    "actor": "xdesign",
                    "outcome": Value::Null,
                    "evidence_uris": [],
                }));
            } else if run_status == "completed" {
                events.push(json!({
                    "kind": "advance",
                    "stage_id": stage_id,
                    "visit_index": visit_index,
                    "title": "Run completed",
                    "detail": Value::Null,
                    "at": Value::Null,
                    "actor": "xdesign",
                    "outcome": Value::Null,
                    "evidence_uris": [],
                }));
            }
        }
    }
    events
}

fn checkpoint_title(plan: &Value, checkpoint_id: &str) -> String {
    if let Some(checkpoints) = plan.get("checkpoints").and_then(Value::as_array) {
        if let Some(checkpoint) = checkpoints.iter().find(|checkpoint| {
            checkpoint.get("checkpoint_id").and_then(Value::as_str) == Some(checkpoint_id)
        }) {
            if checkpoint.get("kind").and_then(Value::as_str) == Some("syntactic") {
                if let Some(checker_id) = checkpoint
                    .get("checker")
                    .and_then(|checker| checker.get("checker_id"))
                    .and_then(Value::as_str)
                {
                    return checker_id.to_owned();
                }
            }
        }
    }
    checkpoint_id.to_owned()
}

fn next_stage_id(plan: &Value, stage_id: &str) -> Option<String> {
    let stages = plan.get("stages")?.as_array()?;
    let stage = stages
        .iter()
        .find(|stage| stage.get("stage_id").and_then(Value::as_str) == Some(stage_id))?;
    stage
        .get("next_stage_ids")?
        .as_array()?
        .first()?
        .as_str()
        .map(str::to_owned)
}

// ---------------------------------------------------------------------------
// Work and check lifecycle actions
// ---------------------------------------------------------------------------

fn work(state: &AppState, wish_id: &str) -> ApiResult {
    ensure_wish_exists(state, wish_id)?;
    let output = run_lifecycle(state, &["next", wish_id])
        .map_err(|message| ApiError::new(500, "work_failed", message))?;
    if output.exit_code == 0 {
        let response: Value = serde_json::from_slice(&output.stdout).map_err(|error| {
            ApiError::internal(format!("xdesign next returned invalid JSON: {error}"))
        })?;
        Ok(http::json_ok(json!({
            "exit_code": output.exit_code,
            "response": response,
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
        })))
    } else {
        Err(ApiError::new(
            500,
            "work_failed",
            lifecycle_message(&output, "xdesign next failed"),
        )
        .with_detail(lifecycle_detail(&output)))
    }
}

#[derive(Deserialize)]
struct ArtifactRequest {
    uri: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JudgmentRequest {
    authority: String,
    identity: String,
    outcome: String,
    rationale: String,
}

struct ArtifactDigest {
    uri: String,
    sha256: String,
}

fn check(state: &AppState, wish_id: &str, request: &Request) -> ApiResult {
    let body: Value = parse_json_body(request)?;
    let object = body.as_object().ok_or_else(|| {
        ApiError::bad_request("invalid_json", "The request body must be a JSON object.")
    })?;
    let description = optional_nonempty_string(object, "description")?;
    let performed_by = optional_nonempty_string(object, "performed_by")?;
    let artifacts = parse_artifacts(state, object.get("artifacts"))?;
    let judgment = match object.get("judgment") {
        None | Some(Value::Null) => None,
        Some(value) => {
            let parsed: JudgmentRequest =
                serde_json::from_value(value.clone()).map_err(|error| {
                    ApiError::bad_request(
                        "invalid_judgment",
                        format!("The judgment is incomplete or malformed: {error}"),
                    )
                })?;
            if !matches!(parsed.authority.as_str(), "agent" | "human") {
                return Err(ApiError::bad_request(
                    "invalid_judgment",
                    "judgment.authority must be agent or human.",
                ));
            }
            if !matches!(parsed.outcome.as_str(), "passed" | "failed" | "unevaluated") {
                return Err(ApiError::bad_request(
                    "invalid_judgment",
                    "judgment.outcome must be passed, failed, or unevaluated.",
                ));
            }
            if parsed.identity.trim().is_empty() || parsed.rationale.trim().is_empty() {
                return Err(ApiError::bad_request(
                    "invalid_judgment",
                    "judgment.identity and judgment.rationale must be nonempty.",
                ));
            }
            Some(parsed)
        }
    };
    ensure_wish_exists(state, wish_id)?;
    let next = run_lifecycle(state, &["next", wish_id])
        .map_err(|message| ApiError::new(500, "check_failed", message))?;
    if next.exit_code != 0 {
        return Err(ApiError::new(
            500,
            "work_failed",
            lifecycle_message(&next, "xdesign next failed before the check"),
        )
        .with_detail(lifecycle_detail(&next)));
    }
    let next_response: Value = serde_json::from_slice(&next.stdout).map_err(|error| {
        ApiError::internal(format!("xdesign next returned invalid JSON: {error}"))
    })?;
    if next_response.get("completed") == Some(&Value::Bool(true)) {
        return Err(ApiError::conflict(
            "already_delivered",
            format!("Wish {wish_id} is already delivered; there is no stage to check."),
        ));
    }
    let work = next_response.get("work").ok_or_else(|| {
        ApiError::internal("xdesign next returned no work for an unfinished wish.")
    })?;
    let stage_id = work
        .get("stage_id")
        .and_then(Value::as_str)
        .ok_or_else(|| ApiError::internal("xdesign next work has no stage_id."))?;
    let has_semantic_gate = work
        .get("exit_checkpoints")
        .and_then(Value::as_array)
        .map(|checkpoints| {
            checkpoints.iter().any(|checkpoint| {
                checkpoint.get("kind").and_then(Value::as_str) == Some("semantic")
            })
        })
        .unwrap_or(false);
    if judgment.is_some() && !has_semantic_gate {
        return Err(ApiError::bad_request(
            "invalid_judgment",
            format!("Stage {stage_id} has no semantic exit checkpoint to judge."),
        ));
    }
    let candidate_snapshot_digest = work
        .get("candidate_snapshot_digest")
        .and_then(Value::as_str)
        .ok_or_else(|| ApiError::internal("xdesign next work has no candidate_snapshot_digest."))?;
    let run_id = next_response
        .get("run_id")
        .and_then(Value::as_str)
        .ok_or_else(|| ApiError::internal("xdesign next response has no run_id."))?;
    let state_token = next_response
        .get("state_token")
        .and_then(Value::as_str)
        .ok_or_else(|| ApiError::internal("xdesign next response has no state_token."))?;
    let visit_index = work
        .get("visit_index")
        .and_then(Value::as_u64)
        .ok_or_else(|| ApiError::internal("xdesign next work has no visit_index."))?;
    let artifact_revision = work
        .get("artifact_revision")
        .and_then(Value::as_u64)
        .ok_or_else(|| ApiError::internal("xdesign next work has no artifact_revision."))?;
    let description = description.unwrap_or_else(|| format!("Web check, stage {stage_id}"));
    let performed_by = performed_by.unwrap_or_else(|| "xdesign web operator".into());
    let mut result = json!({
        "wish_id": wish_id,
        "run_id": run_id,
        "stage_id": stage_id,
        "visit_index": visit_index,
        "base_revision": artifact_revision,
        "state_token": state_token,
        "description": description,
        "performed_by": performed_by,
        "artifacts": artifacts
            .iter()
            .map(|artifact| json!({"uri": artifact.uri, "sha256": artifact.sha256}))
            .collect::<Vec<_>>(),
    });
    if let Some(judgment) = judgment {
        result["judgment"] = json!({
            "authority": judgment.authority,
            "identity": judgment.identity,
            "outcome": judgment.outcome,
            "rationale": judgment.rationale,
            "artifact_snapshot_digest": candidate_snapshot_digest,
        });
    }
    let attempt_uri = format!("cache/web/attempts/{wish_id}-{}.json", uuid::Uuid::new_v4());
    let attempt_path = project_path(&state.project_root, &attempt_uri)?;
    write_project_file(
        &attempt_path,
        &serde_json::to_vec_pretty(&result).map_err(|error| {
            ApiError::internal(format!("cannot encode the check result: {error}"))
        })?,
    )?;
    let output = run_lifecycle(state, &["check", wish_id, "--result", attempt_uri.as_str()])
        .map_err(|message| ApiError::new(500, "check_failed", message))?;
    match output.exit_code {
        0 | 1 => {
            let response: Value = serde_json::from_slice(&output.stdout).map_err(|error| {
                ApiError::internal(format!("xdesign check returned invalid JSON: {error}"))
            })?;
            Ok(http::json_ok(json!({
                "exit_code": output.exit_code,
                "response": response,
                "stdout": String::from_utf8_lossy(&output.stdout),
                "stderr": String::from_utf8_lossy(&output.stderr),
            })))
        }
        _ => Err(ApiError::new(
            500,
            "check_failed",
            lifecycle_message(&output, "xdesign check failed"),
        )
        .with_detail(lifecycle_detail(&output))),
    }
}

fn parse_artifacts(
    state: &AppState,
    value: Option<&Value>,
) -> Result<Vec<ArtifactDigest>, ApiError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    if value.is_null() {
        return Ok(Vec::new());
    }
    let entries = value
        .as_array()
        .ok_or_else(|| ApiError::bad_request("invalid_artifact", "artifacts must be an array."))?;
    let mut seen = BTreeSet::new();
    let mut artifacts = Vec::new();
    for entry in entries {
        let artifact: ArtifactRequest = serde_json::from_value(entry.clone()).map_err(|error| {
            ApiError::bad_request(
                "invalid_artifact",
                format!("each artifact must be an object with a uri: {error}"),
            )
        })?;
        if artifact.uri.is_empty() || !crate::core::valid_relative_path(&artifact.uri) {
            return Err(ApiError::bad_request(
                "invalid_artifact",
                format!(
                    "artifact URI must be a normalized project-relative path: {}",
                    artifact.uri
                ),
            ));
        }
        if !seen.insert(artifact.uri.clone()) {
            return Err(ApiError::bad_request(
                "invalid_artifact",
                format!("duplicate artifact URI: {}", artifact.uri),
            ));
        }
        let sha256 = project_artifact_sha256(&state.project_root, &artifact.uri)?;
        artifacts.push(ArtifactDigest {
            uri: artifact.uri,
            sha256,
        });
    }
    Ok(artifacts)
}

/// SHA-256 of an artifact that must exist in the project. `hash_artifact`
/// resolves project-relative URIs against the process directory, which the
/// server sets to the project root; tests that inspect another project root use
/// the equivalent explicit join below.
fn project_artifact_sha256(project_root: &Path, uri: &str) -> Result<String, ApiError> {
    let relative_to_process = std::env::current_dir()
        .ok()
        .filter(|directory| directory == project_root);
    if relative_to_process.is_some() {
        return hash_artifact(uri)
            .map(|digest| digest.sha256)
            .map_err(|error| ApiError::bad_request("invalid_artifact", error));
    }
    let path = project_path(project_root, uri)?;
    let bytes = fs::read(&path).map_err(|error| {
        ApiError::bad_request(
            "invalid_artifact",
            format!("artifact {uri} is missing or unreadable: {error}"),
        )
    })?;
    Ok(crate::core::sha256_bytes(&bytes))
}

fn ensure_wish_exists(state: &AppState, wish_id: &str) -> Result<(), ApiError> {
    if let Some(store) = open_store(state)? {
        match store.get(wish_id, None) {
            Ok(_) => return Ok(()),
            Err(RsStoreError::NotFound(_)) => {}
            Err(error) => {
                return Err(store_error(
                    &format!("cannot read wish {wish_id} from the store"),
                    error,
                ))
            }
        }
    }
    if state
        .project_root
        .join("cache/xdesign")
        .join(wish_id)
        .join("registry.json")
        .is_file()
    {
        return Ok(());
    }
    Err(ApiError::not_found(
        "unknown_wish",
        format!("No wish with id {wish_id} exists in the store or the legacy cache."),
    ))
}

fn run_lifecycle(state: &AppState, args: &[&str]) -> Result<LifecycleOutput, String> {
    let owned = args
        .iter()
        .map(|argument| argument.to_string())
        .collect::<Vec<_>>();
    state.lifecycle.run(&owned, &state.project_root)
}

fn lifecycle_message(output: &LifecycleOutput, fallback: &str) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let trimmed = stderr.trim();
    if !trimmed.is_empty() {
        return trimmed.to_owned();
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    if !trimmed.is_empty() {
        return trimmed.to_owned();
    }
    format!("{fallback} (exit code {})", output.exit_code)
}

fn lifecycle_detail(output: &LifecycleOutput) -> Value {
    json!({
        "exit_code": output.exit_code,
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
    })
}

// ---------------------------------------------------------------------------
// POST /api/wishes
// ---------------------------------------------------------------------------

const SIMPLE_FLOW_CHECKPOINTS: &[&str] = &[
    "simpleRequirementsAccepted",
    "simpleDesignAccepted",
    "simpleImplementationAccepted",
    "simpleRecordAccepted",
];

/// The SimpleFlow semantic gate, which accepts a semantic library checkpoint.
const SIMPLE_FLOW_SEMANTIC_CHECKPOINT: &str = "verificationAccepted";

/// Library-bound gates per SPEC 9.3: the four syntactic gates and the single
/// SimpleFlow semantic gate.
fn library_bindable_checkpoint(checkpoint_id: &str) -> bool {
    SIMPLE_FLOW_CHECKPOINTS.contains(&checkpoint_id)
        || checkpoint_id == SIMPLE_FLOW_SEMANTIC_CHECKPOINT
}

struct BindingRequest {
    checkpoint_id: String,
    mode: String,
    library_id: Option<String>,
}

fn create_wish(state: &AppState, request: &Request) -> ApiResult {
    let body: Value = parse_json_body(request)?;
    let object = body.as_object().ok_or_else(|| {
        ApiError::bad_request("invalid_json", "The request body must be a JSON object.")
    })?;
    let wish_id = required_string(object, "wish_id")?;
    if !valid_wish_id(&wish_id) {
        return Err(ApiError::bad_request(
            "invalid_id",
            "wish_id must match [A-Za-z0-9][A-Za-z0-9_-]{0,127}.",
        ));
    }
    let title = required_string(object, "title")?;
    if title.trim().is_empty() || title.chars().count() > 200 {
        return Err(ApiError::bad_request(
            "invalid_title",
            "title must be 1 to 200 characters.",
        ));
    }
    let description = match object.get("description") {
        None | Some(Value::Null) => None,
        Some(Value::String(description)) => {
            if description.chars().count() > 4000 {
                return Err(ApiError::bad_request(
                    "invalid_description",
                    "description must be at most 4000 characters.",
                ));
            }
            Some(description.clone())
        }
        Some(_) => {
            return Err(ApiError::bad_request(
                "invalid_description",
                "description must be a string.",
            ))
        }
    };
    let flow_id = required_string(object, "flow_id")?;
    if !matches!(flow_id.as_str(), "simple-flow" | "wish-verify") {
        return Err(ApiError::bad_request(
            "invalid_flow",
            "flow_id must be simple-flow or wish-verify.",
        ));
    }
    let bindings = parse_bindings(&flow_id, object.get("checks"))?;
    ensure_wish_absent(state, &wish_id)?;
    validate_library_bindings(state, &bindings)?;
    let source_uri = format!("cache/web/wishes/{wish_id}.sysml");
    let source_path = project_path(&state.project_root, &source_uri)?;
    write_project_file(
        &source_path,
        wish_source(&wish_id, &title, description.as_deref(), &flow_id).as_bytes(),
    )?;
    let non_default = bindings.iter().any(|binding| binding.mode != "default");
    let checks_uri = if non_default {
        let uri = format!("cache/web/checks/{wish_id}.json");
        let path = project_path(&state.project_root, &uri)?;
        let config = json!({
            "schema_version": 1,
            "checks": bindings
                .iter()
                .map(|binding| {
                    let mut entry = Map::new();
                    entry.insert("checkpoint_id".into(), json!(binding.checkpoint_id));
                    entry.insert("mode".into(), json!(binding.mode));
                    match binding.mode.as_str() {
                        "library" => {
                            entry.insert("library_id".into(), json!(binding.library_id));
                        }
                        _ => {}
                    }
                    Value::Object(entry)
                })
                .collect::<Vec<_>>(),
        });
        write_project_file(
            &path,
            &serde_json::to_vec_pretty(&config)
                .map_err(|error| ApiError::internal(format!("cannot encode checks: {error}")))?,
        )?;
        Some(uri)
    } else {
        None
    };
    let mut args = vec![
        "init".to_owned(),
        wish_id.clone(),
        "--flow".to_owned(),
        flow_id.clone(),
        "--wish".to_owned(),
        source_uri,
    ];
    if let Some(checks_uri) = &checks_uri {
        args.push("--checks".to_owned());
        args.push(checks_uri.clone());
    }
    let output = state
        .lifecycle
        .run(&args, &state.project_root)
        .map_err(|message| ApiError::new(500, "init_failed", message))?;
    if output.exit_code != 0 {
        if init_rejected_binding(&output) {
            return Err(ApiError::bad_request(
                "invalid_checks",
                lifecycle_message(&output, "invalid check bindings"),
            )
            .with_detail(lifecycle_detail(&output)));
        }
        return Err(ApiError::new(
            500,
            "init_failed",
            lifecycle_message(&output, "xdesign init failed"),
        )
        .with_detail(lifecycle_detail(&output)));
    }
    let init: Value = serde_json::from_slice(&output.stdout).map_err(|error| {
        ApiError::internal(format!("xdesign init returned invalid JSON: {error}"))
    })?;
    Ok(http::json_ok(json!({
        "wish_id": wish_id,
        "flow_id": flow_id,
        "init": init,
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
    })))
}

/// The library kind a gate requires, or `None` when the gate cannot be
/// library-bound (structural validation has already rejected that case).
fn required_library_kind(checkpoint_id: &str) -> Option<&'static str> {
    if SIMPLE_FLOW_CHECKPOINTS.contains(&checkpoint_id) {
        Some("syntactic")
    } else if checkpoint_id == SIMPLE_FLOW_SEMANTIC_CHECKPOINT {
        Some("semantic")
    } else {
        None
    }
}

/// Cheap create-time validation of library bindings: one best-effort
/// `checkpoint list` resolves the current library, and a missing id or a
/// kind mismatch is a client error rather than a server failure. When the
/// library cannot be read, `init` stays authoritative and
/// [`init_rejected_binding`] maps its rejection to the same 400 contract.
fn validate_library_bindings(
    state: &AppState,
    bindings: &[BindingRequest],
) -> Result<(), ApiError> {
    let library_bindings: Vec<&BindingRequest> = bindings
        .iter()
        .filter(|binding| binding.mode == "library")
        .collect();
    if library_bindings.is_empty() {
        return Ok(());
    }
    let Ok(entries) = fetch_library_entries(state) else {
        return Ok(());
    };
    let mut kinds: BTreeMap<String, String> = BTreeMap::new();
    for entry in entries {
        if let (Some(checkpoint_id), Some(kind)) = (
            entry.get("checkpoint_id").and_then(Value::as_str),
            entry.get("kind").and_then(Value::as_str),
        ) {
            kinds.insert(checkpoint_id.to_owned(), kind.to_owned());
        }
    }
    for binding in library_bindings {
        let Some(library_id) = binding.library_id.as_deref() else {
            continue;
        };
        let Some(required) = required_library_kind(&binding.checkpoint_id) else {
            continue;
        };
        let Some(kind) = kinds.get(library_id) else {
            return Err(ApiError::bad_request(
                "invalid_checks",
                format!("unknown library checkpoint: {library_id}"),
            ));
        };
        if kind != required {
            return Err(ApiError::bad_request(
                "invalid_checks",
                format!(
                    "library checkpoint {library_id} is {kind} but the gate requires {required}"
                ),
            ));
        }
    }
    Ok(())
}

/// Whether a failed `init` is the CLI's authoritative rejection of a check
/// binding (a client error) rather than an engine or I/O failure.
fn init_rejected_binding(output: &LifecycleOutput) -> bool {
    if output.exit_code != 2 {
        return false;
    }
    let stderr = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
    let stdout = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
    [stderr, stdout].iter().any(|text| {
        text.contains("invalid check binding")
            || text.contains("library checkpoint")
            || text.contains("library entry")
    })
}

/// The page-created wish source: a marker comment read back by the list/detail
/// API, then a SysML package carrying the title and description as a comment.
fn wish_source(wish_id: &str, title: &str, description: Option<&str>, flow_id: &str) -> String {
    let description = description.unwrap_or_default();
    let marker = json!({
        "title": title,
        "description": description,
        "flow_id": flow_id,
    });
    let package: String = wish_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect();
    let title = comment_safe(title);
    let description = comment_safe(description);
    format!(
        "// xdesign-wish: {marker}\n\
         package Wish_{package} {{\n\
         \x20   doc /* {title}\n\
         \n\
         {description} */\n\
         }}\n"
    )
}

/// A SysML block comment ends at `*/`; title and description are display text,
/// so neutralize that sequence instead of rejecting ordinary prose.
fn comment_safe(text: &str) -> String {
    text.replace("*/", "* /")
}

fn parse_bindings(flow_id: &str, value: Option<&Value>) -> Result<Vec<BindingRequest>, ApiError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    if value.is_null() {
        return Ok(Vec::new());
    }
    let entries = value
        .as_array()
        .ok_or_else(|| ApiError::bad_request("invalid_checks", "checks must be an array."))?;
    if flow_id != "simple-flow" && !entries.is_empty() {
        return Err(ApiError::bad_request(
            "invalid_checks",
            "Custom check bindings are only supported for simple-flow.",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut bindings = Vec::new();
    for entry in entries {
        let object = entry.as_object().ok_or_else(|| {
            ApiError::bad_request("invalid_checks", "each checks entry must be an object.")
        })?;
        if object.contains_key("cli") || object.contains_key("model") {
            return Err(ApiError::bad_request(
                "invalid_checks",
                "Agent CLI check bindings are no longer supported.",
            ));
        }
        let checkpoint_id = required_string(object, "checkpoint_id")?;
        if !seen.insert(checkpoint_id.clone()) {
            return Err(ApiError::bad_request(
                "invalid_checks",
                format!("duplicate checkpoint binding: {checkpoint_id}"),
            ));
        }
        let mode = match object.get("mode") {
            None | Some(Value::Null) => "default".to_owned(),
            Some(Value::String(mode)) => mode.clone(),
            Some(_) => {
                return Err(ApiError::bad_request(
                    "invalid_checks",
                    "mode must be a string.",
                ))
            }
        };
        match mode.as_str() {
            "default" => {
                if !SIMPLE_FLOW_CHECKPOINTS.contains(&checkpoint_id.as_str()) {
                    return Err(ApiError::bad_request(
                        "invalid_checks",
                        format!("{checkpoint_id} is not a simple-flow syntactic checkpoint."),
                    ));
                }
                bindings.push(BindingRequest {
                    checkpoint_id,
                    mode,
                    library_id: None,
                });
            }
            "library" => {
                if !library_bindable_checkpoint(&checkpoint_id) {
                    return Err(ApiError::bad_request(
                        "invalid_checks",
                        format!(
                            "{checkpoint_id} does not accept a library binding; the four simple-flow syntactic checkpoints and {SIMPLE_FLOW_SEMANTIC_CHECKPOINT} do."
                        ),
                    ));
                }
                let library_id = match object.get("library_id") {
                    Some(Value::String(library_id)) => library_id.clone(),
                    Some(_) => {
                        return Err(ApiError::bad_request(
                            "invalid_checks",
                            "library_id must be a string.",
                        ))
                    }
                    None => {
                        return Err(ApiError::bad_request(
                            "invalid_checks",
                            "mode \"library\" requires library_id.",
                        ))
                    }
                };
                if !valid_checkpoint_id(&library_id) {
                    return Err(ApiError::bad_request(
                        "invalid_checks",
                        "library_id must be a lower-case checkpoint id slug.",
                    ));
                }
                bindings.push(BindingRequest {
                    checkpoint_id,
                    mode,
                    library_id: Some(library_id),
                });
            }
            "command" => {
                return Err(ApiError::bad_request(
                    "invalid_checks",
                    "mode \"command\" is configured through `xdesign init --checks`, not through the web server.",
                ))
            }
            _ => {
                return Err(ApiError::bad_request(
                    "invalid_checks",
                    "mode must be default or library.",
                ))
            }
        }
    }
    Ok(bindings)
}

fn ensure_wish_absent(state: &AppState, wish_id: &str) -> Result<(), ApiError> {
    if let Some(store) = open_store(state)? {
        match store.get(wish_id, None) {
            Ok(_) => {
                return Err(ApiError::conflict(
                    "already_exists",
                    format!("Wish {wish_id} already exists in the store."),
                ))
            }
            Err(RsStoreError::NotFound(_)) => {}
            Err(error) => {
                return Err(store_error(
                    &format!("cannot inspect wish {wish_id} in the store"),
                    error,
                ))
            }
        }
    }
    if fs::symlink_metadata(state.project_root.join("cache/xdesign").join(wish_id)).is_ok() {
        return Err(ApiError::conflict(
            "already_exists",
            format!("Wish {wish_id} already exists in the legacy cache."),
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// GET /api/wishes/{wish_id}/material
// ---------------------------------------------------------------------------

fn material(state: &AppState, wish_id: &str, request: &Request) -> ApiResult {
    let uri = request
        .query_param("uri")
        .filter(|uri| !uri.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            ApiError::bad_request(
                "missing_uri",
                "The material request requires a ?uri= parameter.",
            )
        })?;
    if !crate::core::valid_relative_path(&uri) {
        return Err(ApiError::bad_request(
            "invalid_uri",
            "The material URI must be a normalized project-relative path.",
        ));
    }
    if let Some(store) = open_store(state)? {
        match store.get(wish_id, None) {
            Ok(head) => return store_material(&store, wish_id, &head, &uri),
            Err(RsStoreError::NotFound(_)) => {}
            Err(error) => {
                return Err(store_error(
                    &format!("cannot read wish {wish_id} from the store"),
                    error,
                ))
            }
        }
    }
    let directory = state.project_root.join("cache/xdesign").join(wish_id);
    let registry = read_registry_file(&directory.join("registry.json")).ok_or_else(|| {
        ApiError::not_found(
            "unknown_wish",
            format!("No wish with id {wish_id} exists in the store or the legacy cache."),
        )
    })?;
    let prefix = format!("cache/xdesign/{wish_id}/");
    let mut allowed = run_record_uris(&state.project_root, &registry.run_record_uri);
    if uri.starts_with(&prefix) {
        allowed.insert(uri.clone());
    }
    if !allowed.contains(&uri) {
        return Err(ApiError::not_found(
            "material_not_found",
            format!("{uri} is not retained evidence or an artifact of wish {wish_id}."),
        ));
    }
    let path = project_path(&state.project_root, &uri)?;
    let metadata = fs::metadata(&path).map_err(|error| {
        ApiError::not_found(
            "material_not_found",
            format!("material {uri} does not exist: {error}"),
        )
    })?;
    if !metadata.is_file() {
        return Err(ApiError::not_found(
            "material_not_found",
            format!("material {uri} is not a regular file."),
        ));
    }
    if metadata.len() > MAX_MATERIAL_BYTES {
        return Err(ApiError::new(
            413,
            "material_too_large",
            format!("material {uri} exceeds the 512 KiB limit."),
        ));
    }
    let bytes = fs::read(&path).map_err(|error| {
        ApiError::not_found(
            "material_not_found",
            format!("material {uri} cannot be read: {error}"),
        )
    })?;
    Ok(http::raw(material_content_type(&uri), bytes))
}

fn store_material(store: &RsStore, wish_id: &str, head: &RsItemRevision, uri: &str) -> ApiResult {
    let Some(material) = head.materials.iter().find(|material| material.uri == uri) else {
        return Err(ApiError::not_found(
            "material_not_found",
            format!("The current revision of wish {wish_id} has no material at {uri}."),
        ));
    };
    if material.byte_length > MAX_MATERIAL_BYTES {
        return Err(ApiError::new(
            413,
            "material_too_large",
            format!("material {uri} exceeds the 512 KiB limit."),
        ));
    }
    let bytes = store
        .read_material(wish_id, Some(head.revision), uri)
        .map_err(|error| match error {
            RsStoreError::NotFound(_) => ApiError::not_found(
                "material_not_found",
                format!("The current revision of wish {wish_id} has no material at {uri}."),
            ),
            other => store_error(&format!("cannot read material {uri}"), other),
        })?;
    Ok(http::raw(material_content_type(uri), bytes))
}

fn material_content_type(uri: &str) -> &'static str {
    match uri
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("json") => "application/json; charset=utf-8",
        Some("md" | "txt" | "sysml") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn run_record_uris(project_root: &Path, run_record_uri: &str) -> BTreeSet<String> {
    let mut uris = BTreeSet::new();
    let Ok(bytes) = fs::read(project_root.join(run_record_uri)) else {
        return uris;
    };
    let Ok(run) = serde_json::from_slice::<Value>(&bytes) else {
        return uris;
    };
    let Some(visits) = run.get("visits").and_then(Value::as_array) else {
        return uris;
    };
    for visit in visits {
        for digest in visit
            .get("artifact_digests")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(uri) = digest.get("uri").and_then(Value::as_str) {
                uris.insert(uri.to_owned());
            }
        }
        for activity in visit
            .get("activities")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            for key in ["artifact_uris", "removed_artifact_uris"] {
                for uri in activity
                    .get(key)
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    uris.insert(uri.to_owned());
                }
            }
            for digest in activity
                .get("artifact_digests")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Some(uri) = digest.get("uri").and_then(Value::as_str) {
                    uris.insert(uri.to_owned());
                }
            }
        }
        for assessment in visit
            .get("assessments")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            for uri in assessment
                .get("evidence_uris")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                uris.insert(uri.to_owned());
            }
        }
    }
    uris
}

// ---------------------------------------------------------------------------
// Checkpoint library endpoints (round 2, SPEC 9.4)
// ---------------------------------------------------------------------------

/// Which CLI action failed, for the small differences in HTTP error mapping.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CheckpointAction {
    List,
    Show,
    Save,
}

impl CheckpointAction {
    fn command(self) -> &'static str {
        match self {
            Self::List => "xdesign checkpoint list",
            Self::Show => "xdesign checkpoint show",
            Self::Save => "xdesign checkpoint save",
        }
    }
}

fn checkpoint_list(state: &AppState) -> ApiResult {
    let output = run_lifecycle(state, &["checkpoint", "list"]).map_err(checkpoint_spawn_error)?;
    if output.exit_code != 0 {
        return Err(classify_checkpoint_error(&output, CheckpointAction::List));
    }
    Ok(http::json_ok(checkpoint_reply(
        &output,
        CheckpointAction::List,
    )?))
}

fn checkpoint_show(state: &AppState, checkpoint_id: &str) -> ApiResult {
    let output = run_lifecycle(state, &["checkpoint", "show", checkpoint_id])
        .map_err(checkpoint_spawn_error)?;
    if output.exit_code != 0 {
        return Err(classify_checkpoint_error(&output, CheckpointAction::Show));
    }
    Ok(http::json_ok(checkpoint_reply(
        &output,
        CheckpointAction::Show,
    )?))
}

fn checkpoint_save(state: &AppState, request: &Request) -> ApiResult {
    let body = parse_json_body(request)?;
    if !body.is_object() {
        return Err(ApiError::bad_request(
            "invalid_checkpoint",
            "The checkpoint save request must be a JSON object.",
        ));
    }
    let request_uri = format!(
        "cache/web/checkpoint-requests/{}.json",
        uuid::Uuid::new_v4()
    );
    let request_path = project_path(&state.project_root, &request_uri)?;
    let encoded = serde_json::to_vec_pretty(&body).map_err(|error| {
        ApiError::internal(format!("cannot encode the checkpoint request: {error}"))
    })?;
    write_project_file(&request_path, &encoded)?;
    let output = run_lifecycle(
        state,
        &["checkpoint", "save", "--request", request_uri.as_str()],
    )
    .map_err(checkpoint_spawn_error)?;
    if output.exit_code != 0 {
        return Err(classify_checkpoint_error(&output, CheckpointAction::Save));
    }
    Ok(http::json_ok(checkpoint_reply(
        &output,
        CheckpointAction::Save,
    )?))
}

fn checkpoint_spawn_error(message: String) -> ApiError {
    ApiError::new(500, "checkpoint_failed", message)
}

/// Parse one successful CLI checkpoint reply and pass it through as `data`.
fn checkpoint_reply(output: &LifecycleOutput, action: CheckpointAction) -> Result<Value, ApiError> {
    let reply: Value = serde_json::from_slice(&output.stdout).map_err(|error| {
        ApiError::new(
            500,
            "checkpoint_failed",
            format!(
                "{} returned a reply that is not JSON: {error}",
                action.command()
            ),
        )
        .with_detail(lifecycle_detail(output))
    })?;
    if reply.get("schema_version").and_then(Value::as_u64) != Some(1) {
        return Err(ApiError::new(
            500,
            "checkpoint_failed",
            format!("{} returned an unsupported reply", action.command()),
        )
        .with_detail(lifecycle_detail(output)));
    }
    Ok(reply)
}

/// Parse a CLI error object if the binary printed one (on stdout or stderr).
fn checkpoint_error_object(output: &LifecycleOutput) -> Option<Value> {
    for bytes in [&output.stdout, &output.stderr] {
        if let Ok(value) = serde_json::from_slice::<Value>(bytes) {
            if value.is_object() {
                return Some(value);
            }
        }
    }
    None
}

fn checkpoint_error_fields(output: &LifecycleOutput) -> (Option<String>, Option<String>) {
    let Some(value) = checkpoint_error_object(output) else {
        return (None, None);
    };
    let error = value.get("error");
    let code = error
        .and_then(|error| error.get("code"))
        .or_else(|| value.get("code"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let message = error
        .and_then(|error| error.get("message"))
        .or_else(|| value.get("message"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    (code, message)
}

fn checkpoint_message(output: &LifecycleOutput, fallback: &str) -> String {
    let (_, message) = checkpoint_error_fields(output);
    message.unwrap_or_else(|| lifecycle_message(output, fallback))
}

/// Map CLI failures to the HTTP contract: 400 invalid definition, 409 built-in
/// mutation, 404 unknown id for `show`, and 500 for everything else.
fn classify_checkpoint_error(output: &LifecycleOutput, action: CheckpointAction) -> ApiError {
    let (code, _) = checkpoint_error_fields(output);
    let message = checkpoint_message(output, &format!("{} failed", action.command()));
    let lower = message.to_ascii_lowercase();
    let detail = lifecycle_detail(output);
    let code_lower = code.as_deref().unwrap_or_default().to_ascii_lowercase();
    let builtin = code_lower.contains("builtin")
        || code_lower.contains("immutable")
        || lower.contains("built-in")
        || lower.contains("builtin");
    let unknown = code_lower.contains("unknown")
        || code_lower.contains("not_found")
        || code_lower.contains("not-found")
        || lower.contains("unknown")
        || lower.contains("does not exist")
        || lower.contains("not found")
        || lower.contains("not present");
    // The CLI contract distinguishes `invalid_checkpoint` (definition problems)
    // from `invalid_request` (bad CLI invocation, a server-side defect here).
    let invalid_definition = code_lower.contains("invalid_checkpoint")
        || code_lower.contains("validation")
        || code_lower.contains("schema")
        || code_lower.contains("unsupported");
    match action {
        CheckpointAction::Save => {
            if builtin {
                return ApiError::new(409, "builtin_checkpoint", message).with_detail(detail);
            }
            if invalid_definition {
                return ApiError::new(400, "invalid_checkpoint", message).with_detail(detail);
            }
            // A plain-text/uncoded exit-2 failure is treated as an invalid
            // definition; a coded non-definition failure is a server error.
            if code.is_none() && output.exit_code == 2 {
                return ApiError::new(400, "invalid_checkpoint", message).with_detail(detail);
            }
            ApiError::new(500, "checkpoint_failed", message).with_detail(detail)
        }
        CheckpointAction::Show => {
            if unknown {
                return ApiError::not_found("unknown_checkpoint", message);
            }
            ApiError::new(500, "checkpoint_failed", message).with_detail(detail)
        }
        CheckpointAction::List => {
            ApiError::new(500, "checkpoint_failed", message).with_detail(detail)
        }
    }
}

// ---------------------------------------------------------------------------
// Static assets
// ---------------------------------------------------------------------------

enum AssetLookup {
    Found(&'static [u8], &'static str),
    SpaIndex(&'static [u8], &'static str),
    NotFound,
}

fn route_asset(request: &Request) -> Response {
    if request.method != Method::Get {
        return ApiError::method_not_allowed().into_response();
    }
    match lookup_asset(super::ASSET_FILES, &request.path) {
        AssetLookup::Found(bytes, content_type) => http::raw(content_type, bytes.to_vec()),
        AssetLookup::SpaIndex(bytes, content_type) => http::raw(content_type, bytes.to_vec()),
        AssetLookup::NotFound => http::text_error(404, "Not found"),
    }
}

/// Serve an embedded asset by path. `GET /` maps to `/index.html`; unknown
/// extensionless paths fall back to the page entry so hash routing works,
/// while an unknown asset-like path is a hard 404.
fn lookup_asset(
    assets: &'static [(&'static str, &'static [u8], &'static str)],
    path: &str,
) -> AssetLookup {
    let direct = if path == "/" { "/index.html" } else { path };
    if let Some((_, bytes, content_type)) = assets
        .iter()
        .find(|(asset_path, _, _)| *asset_path == direct)
    {
        return AssetLookup::Found(bytes, content_type);
    }
    if path != "/" && !asset_like(path) {
        if let Some((_, bytes, content_type)) = assets
            .iter()
            .find(|(asset_path, _, _)| *asset_path == "/index.html")
        {
            return AssetLookup::SpaIndex(bytes, content_type);
        }
    }
    AssetLookup::NotFound
}

fn asset_like(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .and_then(|name| name.rsplit_once('.'))
        .is_some_and(|(stem, extension)| !stem.is_empty() && !extension.is_empty())
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn open_store(state: &AppState) -> Result<Option<RsStore>, ApiError> {
    if !state.store_root.exists() {
        return Ok(None);
    }
    match RsStore::open(&state.store_root) {
        Ok(store) => Ok(Some(store)),
        Err(RsStoreError::NotFound(_)) | Err(RsStoreError::UnsupportedFormat(_)) => Ok(None),
        Err(error) => Err(store_error("cannot open the operational store", error)),
    }
}

fn store_error(context: &str, error: RsStoreError) -> ApiError {
    match error {
        RsStoreError::NotFound(message) => ApiError::not_found("not_found", message),
        RsStoreError::InvalidInput(message) => ApiError::bad_request("invalid_request", message),
        RsStoreError::UnsupportedFormat(message) => {
            ApiError::internal(format!("{context}: {message}"))
        }
        other => ApiError::internal(format!("{context}: {other}")),
    }
}

fn material_by_role<'a>(head: &'a RsItemRevision, role: &str) -> Option<&'a RsMaterial> {
    head.materials.iter().find(|material| material.role == role)
}

fn head_uri_by_role(head: &RsItemRevision, role: &str) -> Option<String> {
    material_by_role(head, role).map(|material| material.uri.clone())
}

fn read_head_material(
    store: &RsStore,
    head: &RsItemRevision,
    uri: &str,
) -> Result<Vec<u8>, ApiError> {
    store
        .read_material(&head.item_id, Some(head.revision), uri)
        .map_err(|error| match error {
            RsStoreError::NotFound(message) => ApiError::not_found("material_not_found", message),
            other => store_error(&format!("cannot read material {uri}"), other),
        })
}

fn store_wish_source(store: &RsStore, wish_id: &str, revision: Option<u64>) -> Option<String> {
    let head = store.get(wish_id, revision).ok()?;
    let registry_uri = head
        .metadata
        .get("registry_uri")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| head_uri_by_role(&head, "registration"))?;
    let registry_bytes = store
        .read_material(wish_id, Some(head.revision), &registry_uri)
        .ok()?;
    let registry: RegistryView = serde_json::from_slice(&registry_bytes).ok()?;
    let bytes = store
        .read_material(wish_id, Some(head.revision), &registry.wish_model_uri)
        .ok()
        .or_else(|| {
            head_uri_by_role(&head, "wish-source")
                .and_then(|uri| store.read_material(wish_id, Some(head.revision), &uri).ok())
        })?;
    String::from_utf8(bytes).ok()
}

fn read_legacy_wish_source(project_root: &Path, wish_model_uri: &str) -> Option<String> {
    if !crate::core::valid_relative_path(wish_model_uri) {
        return None;
    }
    fs::read(project_root.join(wish_model_uri))
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
}

#[derive(Deserialize)]
struct RegistryView {
    #[serde(default)]
    flow_id: String,
    #[serde(default)]
    wish_model_uri: String,
    #[serde(default)]
    plan_uri: String,
    #[serde(default)]
    plan_sha256: String,
    #[serde(default)]
    run_record_uri: String,
}

fn read_registry_file(path: &Path) -> Option<RegistryView> {
    let bytes = fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Title and description of a wish source: the page-created marker comment
/// first, then the first SysML `attribute redefines description` value, then
/// the wish id with no description.
fn wish_title_description(wish_id: &str, source_text: Option<&str>) -> (String, Option<String>) {
    let Some(text) = source_text else {
        return (wish_id.to_owned(), None);
    };
    for line in text.lines() {
        let line = line.trim();
        let Some(marker) = line.strip_prefix("// xdesign-wish:") else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(marker.trim()) else {
            continue;
        };
        let title = value
            .get("title")
            .and_then(Value::as_str)
            .filter(|title| !title.is_empty());
        let description = value
            .get("description")
            .and_then(Value::as_str)
            .filter(|description| !description.is_empty())
            .map(str::to_owned);
        if let Some(title) = title {
            return (title.to_owned(), description);
        }
    }
    if let Some(description) = first_description_attribute(text) {
        return (wish_id.to_owned(), Some(description));
    }
    (wish_id.to_owned(), None)
}

fn first_description_attribute(text: &str) -> Option<String> {
    let marker = "attribute redefines description";
    let start = text.find(marker)?;
    let rest = &text[start + marker.len()..];
    let quote = rest.find('"')?;
    let mut value = String::new();
    let mut chars = rest[quote + 1..].chars();
    while let Some(character) = chars.next() {
        match character {
            '\\' => match chars.next()? {
                'n' => value.push('\n'),
                't' => value.push('\t'),
                'r' => value.push('\r'),
                other => value.push(other),
            },
            '"' => {
                let value = value.trim().to_owned();
                return (!value.is_empty()).then_some(value);
            }
            other => value.push(other),
        }
    }
    None
}

fn parse_json_body(request: &Request) -> Result<Value, ApiError> {
    serde_json::from_slice(&request.body).map_err(|error| {
        ApiError::bad_request(
            "invalid_json",
            format!("The request body is not valid JSON: {error}"),
        )
    })
}

fn required_string(object: &Map<String, Value>, field: &str) -> Result<String, ApiError> {
    match object.get(field) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(ApiError::bad_request(
            "invalid_request",
            format!("{field} must be a string."),
        )),
        None => Err(ApiError::bad_request(
            "invalid_request",
            format!("{field} is required."),
        )),
    }
}

fn optional_nonempty_string(
    object: &Map<String, Value>,
    field: &str,
) -> Result<Option<String>, ApiError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => {
            if value.trim().is_empty() {
                Err(ApiError::bad_request(
                    "invalid_request",
                    format!("{field} must be nonempty when supplied."),
                ))
            } else {
                Ok(Some(value.clone()))
            }
        }
        Some(_) => Err(ApiError::bad_request(
            "invalid_request",
            format!("{field} must be a string."),
        )),
    }
}

/// Resolve a project-relative URI to a path without escaping the project root.
fn project_path(project_root: &Path, uri: &str) -> Result<PathBuf, ApiError> {
    if !crate::core::valid_relative_path(uri) {
        return Err(ApiError::bad_request(
            "invalid_uri",
            format!("path must be normalized and project-relative: {uri}"),
        ));
    }
    Ok(project_root.join(uri))
}

fn write_project_file(path: &Path, bytes: &[u8]) -> Result<(), ApiError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            ApiError::internal(format!("cannot create {}: {error}", parent.display()))
        })?;
    }
    fs::write(path, bytes)
        .map_err(|error| ApiError::internal(format!("cannot write {}: {error}", path.display())))
}

/// Current UTC time as the compact RFC 3339 form used by the health payload.
fn utc_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let days = seconds.div_euclid(86_400);
    let time = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time / 3_600,
        time % 3_600 / 60,
        time % 60
    )
}

/// Howard Hinnant's days-to-civil date algorithm, valid for the Unix epoch
/// range the operator page can display.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_position = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_position + 2) / 5 + 1;
    let month = if month_position < 10 {
        month_position + 3
    } else {
        month_position - 9
    };
    let year = if month <= 2 { year + 1 } else { year };
    (year, month as u32, day as u32)
}

// Keep the type imports used only in documentation honest without leaving an
// unused import behind on platforms where a helper compiles differently.
const _: fn() = || {};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simple_flow::designed_simple_flow_plan;
    use crate::storage::{RsOperationOutcome, RsSaveMaterial, RsSaveRequest, RsStore};
    use crate::wish_verify::wish_verify_flow_plan;
    use serde_json::json;
    use std::collections::{BTreeMap, VecDeque};
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    // ------------------------------------------------------------------
    // Fixtures
    // ------------------------------------------------------------------

    struct TestProject {
        root: PathBuf,
    }

    impl TestProject {
        fn new(label: &str) -> Self {
            let root =
                PathBuf::from("cache/web-tests").join(format!("{label}-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn root(&self) -> &Path {
            &self.root
        }

        fn write(&self, uri: &str, bytes: &[u8]) {
            let path = self.root.join(uri);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }

        fn write_text(&self, uri: &str, text: &str) {
            self.write(uri, text.as_bytes());
        }

        fn store_root(&self) -> PathBuf {
            self.root.join(".xdesign")
        }

        fn read_text(&self, uri: &str) -> String {
            fs::read_to_string(self.root.join(uri)).unwrap()
        }
    }

    impl Drop for TestProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    type LifecycleReply = Result<LifecycleOutput, String>;

    #[derive(Clone, Default)]
    struct FakeLifecycle {
        inner: Arc<FakeLifecycleInner>,
    }

    #[derive(Default)]
    struct FakeLifecycleInner {
        calls: Mutex<Vec<(Vec<String>, PathBuf)>>,
        replies: Mutex<VecDeque<LifecycleReply>>,
        delays: Mutex<BTreeMap<String, Duration>>,
    }

    impl FakeLifecycle {
        fn push(&self, reply: LifecycleReply) {
            self.inner.replies.lock().unwrap().push_back(reply);
        }

        fn delay(&self, command: &str, delay: Duration) {
            self.inner
                .delays
                .lock()
                .unwrap()
                .insert(command.to_owned(), delay);
        }

        fn calls(&self) -> Vec<(Vec<String>, PathBuf)> {
            self.inner.calls.lock().unwrap().clone()
        }

        fn call_args(&self) -> Vec<Vec<String>> {
            self.calls().into_iter().map(|(args, _)| args).collect()
        }
    }

    impl Lifecycle for FakeLifecycle {
        fn run(&self, args: &[String], cwd: &Path) -> Result<LifecycleOutput, String> {
            self.inner
                .calls
                .lock()
                .unwrap()
                .push((args.to_vec(), cwd.to_path_buf()));
            let command = args.first().cloned().unwrap_or_default();
            if let Some(delay) = self.inner.delays.lock().unwrap().get(&command).copied() {
                std::thread::sleep(delay);
            }
            self.inner
                .replies
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| Err("unexpected lifecycle call".into()))
        }
    }

    fn lifecycle_reply(exit_code: i32, stdout: Value) -> LifecycleReply {
        Ok(LifecycleOutput {
            exit_code,
            stdout: serde_json::to_vec(&stdout).unwrap(),
            stderr: Vec::new(),
        })
    }

    fn lifecycle_failure(exit_code: i32, stderr: &str) -> LifecycleReply {
        Ok(LifecycleOutput {
            exit_code,
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        })
    }

    fn test_state(project: &TestProject, lifecycle: FakeLifecycle) -> AppState {
        AppState::with_parts(
            project.root().to_path_buf(),
            project.store_root(),
            Box::new(lifecycle),
        )
    }

    // ------------------------------------------------------------------
    // Store seeding
    // ------------------------------------------------------------------

    fn flow_plan(flow_id: &str) -> RsFlowPlan {
        match flow_id {
            "simple-flow" => designed_simple_flow_plan(),
            "wish-verify" => wish_verify_flow_plan(),
            other => panic!("unknown flow {other}"),
        }
    }

    fn frozen_plan_bytes(plan: &RsFlowPlan) -> Vec<u8> {
        serde_json::to_vec_pretty(&json!({
            "plan_id": plan.plan_id,
            "initial_stage_id": plan.initial_stage_id,
            "stages": plan.stages,
            "checkpoints": plan.checkpoints,
        }))
        .unwrap()
    }

    /// A frozen SimpleFlow plan carrying library bindings, shaped like the CLI
    /// freezes them (SPEC 9.3): `checkpoint.library` plus the library entry's
    /// rules/executor replacing the built-in checker definition.
    fn library_frozen_plan(flow_id: &str, bindings: &[(&str, &str)]) -> Value {
        let plan = flow_plan(flow_id);
        let mut frozen: Value = serde_json::from_slice(&frozen_plan_bytes(&plan)).unwrap();
        for checkpoint in frozen["checkpoints"].as_array_mut().unwrap() {
            let checkpoint_id = checkpoint["checkpoint_id"]
                .as_str()
                .unwrap_or_default()
                .to_owned();
            let Some((_, library_id)) = bindings
                .iter()
                .find(|(bound_id, _)| *bound_id == checkpoint_id)
            else {
                continue;
            };
            checkpoint["library"] = json!(library_id);
            if let Some(checker) = checkpoint
                .get_mut("checker")
                .filter(|checker| !checker.is_null())
            {
                checker["checker_id"] = json!(library_id);
                checker["command_uri"] = json!(format!(
                    "checkpoints/scripts/{}.py",
                    library_id.replace('-', "_")
                ));
                checker["custom"] = json!({"runner": "process"});
            }
        }
        frozen
    }

    fn stage_ids(flow_id: &str) -> &'static [&'static str] {
        match flow_id {
            "simple-flow" => &["requirements", "design", "implement", "verify"],
            _ => &["wish", "verify"],
        }
    }

    fn hex(fill: char, length: usize) -> String {
        std::iter::repeat(fill).take(length).collect()
    }

    /// A structurally valid run record for a wish: stage visits in visit order.
    fn seeded_run(wish_id: &str, flow_id: &str, plan_sha256: &str, status: &str) -> Value {
        let directory = format!("cache/xdesign/{wish_id}");
        let stages = stage_ids(flow_id);
        let current_visit = match status {
            "active" => 1,
            "blocked" => 1,
            _ => stages.len() - 1,
        };
        let mut visits = Vec::new();
        for (index, stage_id) in stages.iter().enumerate() {
            let stage_id = *stage_id;
            let visit_status = match status {
                "active" | "blocked" => {
                    if index == 0 {
                        "completed"
                    } else if index == 1 {
                        "active"
                    } else {
                        "pending"
                    }
                }
                _ => "completed",
            };
            let completed = visit_status == "completed";
            let artifact_uri = format!("mbse/model/{stage_id}.sysml");
            let digest = hex('c', 64);
            let snapshot = hex('d', 64);
            let evidence_uri = format!("{directory}/reports/{stage_id}_1_report.json");
            let visit = json!({
                "stage_id": stage_id,
                "visit_index": 0,
                "status": visit_status,
                "artifact_revision": if completed { 1 } else { 0 },
                "artifact_snapshot_digest": if completed { snapshot.clone() } else { hex('e', 64) },
                "artifact_digests": if completed {
                    json!([{"uri": artifact_uri, "sha256": digest}])
                } else {
                    json!([])
                },
                "activities": if completed {
                    json!([{
                        "description": format!("Record {stage_id} stage input artifacts"),
                        "performed_by": "test operator",
                        "occurred_at": "2026-10-20T10:00:00Z",
                        "artifact_uris": [artifact_uri],
                        "artifacts_changed": true,
                        "artifact_digests": [{"uri": artifact_uri, "sha256": digest}],
                        "removed_artifact_uris": [],
                    }])
                } else {
                    json!([])
                },
                "assessments": if completed {
                    json!([{
                        "checkpoint_id": match stage_id {
                            "wish" => "wishAccepted",
                            "verify" if flow_id == "wish-verify" => "outcomeAccepted",
                            "verify" => "simpleRecordAccepted",
                            "requirements" => "simpleRequirementsAccepted",
                            "design" => "simpleDesignAccepted",
                            _ => "simpleImplementationAccepted",
                        },
                        "evaluated_revision": 1,
                        "evaluated_snapshot_digest": snapshot,
                        "outcome": "passed",
                        "judged_by": "script",
                        "assessor": format!("{stage_id}Check"),
                        "rationale": "All declared checker rules passed",
                        "evidence_uris": [evidence_uri],
                    }])
                } else {
                    json!([])
                },
            });
            visits.push(visit);
        }
        json!({
            "run_id": uuid::Uuid::new_v4().to_string(),
            "execution_type": "FlowRun",
            "plan_id": if flow_id == "simple-flow" { "designedSimpleFlow" } else { "designedWishVerifyFlow" },
            "plan_uri": format!("{directory}/plan.json"),
            "plan_sha256": plan_sha256,
            "wish_model_uri": format!("cache/wishes/{wish_id}.sysml"),
            "status": status,
            "current_visit": current_visit,
            "visits": visits,
        })
    }

    fn wish_source_text(wish_id: &str, title: Option<&str>, description: Option<&str>) -> String {
        match title {
            Some(title) => format!(
                "// xdesign-wish: {}\npackage Wish_{} {{}}\n",
                json!({
                    "title": title,
                    "description": description.unwrap_or_default(),
                    "flow_id": "simple-flow",
                }),
                wish_id.replace('-', "_")
            ),
            None => format!(
                "package Wish_{} {{\n    attribute redefines description = \"{}\";\n}}\n",
                wish_id.replace('-', "_"),
                description.unwrap_or("A test wish.").replace('"', "'")
            ),
        }
    }

    /// Create a real operational store wish (registry, frozen plan, work, run
    /// record, and wish source saved as store materials).
    fn seed_store_wish(
        project: &TestProject,
        wish_id: &str,
        flow_id: &str,
        status: &str,
        marker: Option<(&str, &str)>,
        description: Option<&str>,
    ) -> Value {
        seed_store_wish_with_plan(project, wish_id, flow_id, status, marker, description, None)
    }

    fn seed_store_wish_with_plan(
        project: &TestProject,
        wish_id: &str,
        flow_id: &str,
        status: &str,
        marker: Option<(&str, &str)>,
        description: Option<&str>,
        frozen_plan: Option<Value>,
    ) -> Value {
        let plan = flow_plan(flow_id);
        let plan_bytes = match frozen_plan {
            Some(frozen) => serde_json::to_vec_pretty(&frozen).unwrap(),
            None => frozen_plan_bytes(&plan),
        };
        let plan_sha256 = crate::core::sha256_bytes(&plan_bytes);
        let run = seeded_run(wish_id, flow_id, &plan_sha256, status);
        let run_bytes = serde_json::to_vec_pretty(&run).unwrap();
        let work_bytes = serde_json::to_vec_pretty(
            &json!({"schema_version": 1, "flow_id": flow_id, "stages": []}),
        )
        .unwrap();
        let source_text = match marker {
            Some((title, marker_description)) => {
                wish_source_text(wish_id, Some(title), Some(marker_description))
            }
            None => wish_source_text(wish_id, None, description),
        };
        let directory = format!("cache/xdesign/{wish_id}");
        let plan_uri = format!("{directory}/plan.json");
        let work_uri = format!("{directory}/work.json");
        let run_uri = format!("{directory}/run.json");
        let registry_uri = format!("{directory}/registry.json");
        let wish_uri = format!("cache/wishes/{wish_id}.sysml");
        let registry = json!({
            "schema_version": 1,
            "wish_id": wish_id,
            "flow_id": flow_id,
            "run_id": run["run_id"],
            "wish_model_uri": wish_uri,
            "plan_uri": plan_uri,
            "plan_sha256": plan_sha256,
            "work_uri": work_uri,
            "work_sha256": crate::core::sha256_bytes(&work_bytes),
            "run_record_uri": run_uri,
        });
        let registry_bytes = serde_json::to_vec_pretty(&registry).unwrap();
        project.write(&plan_uri, &plan_bytes);
        project.write(&work_uri, &work_bytes);
        project.write(&run_uri, &run_bytes);
        project.write(&registry_uri, &registry_bytes);
        project.write_text(&wish_uri, &source_text);
        let registry_uri_for_save = registry_uri.clone();
        let run_uri_for_save = run_uri.clone();
        save_wish_request(
            project,
            wish_id,
            flow_id,
            &registry_uri_for_save,
            &run_uri_for_save,
            vec![
                (registry_uri, "registration", registry_bytes),
                (plan_uri, "frozen-plan", plan_bytes),
                (work_uri, "frozen-work", work_bytes),
                (run_uri, "run-record", run_bytes),
                (wish_uri, "wish-source", source_text.into_bytes()),
            ],
        );
        run
    }

    fn save_wish_request(
        project: &TestProject,
        wish_id: &str,
        flow_id: &str,
        registry_uri: &str,
        run_uri: &str,
        materials: Vec<(String, &str, Vec<u8>)>,
    ) {
        let store_root = project.store_root();
        let store = match RsStore::open(&store_root) {
            Ok(store) => store,
            Err(_) => RsStore::init(&store_root).unwrap(),
        };
        let save_materials = materials
            .into_iter()
            .map(|(uri, role, bytes)| RsSaveMaterial {
                source: Some(PathBuf::from(&uri)),
                uri,
                role: role.into(),
                sha256: crate::core::sha256_bytes(&bytes),
            })
            .collect();
        let metadata = json!({
            "flow_id": flow_id,
            "registry_uri": registry_uri,
            "run_uri": run_uri,
        })
        .as_object()
        .unwrap()
        .clone();
        let request = RsSaveRequest {
            schema_version: 1,
            operation_id: uuid::Uuid::new_v4().to_string(),
            item_id: wish_id.into(),
            kind: "wish".into(),
            expected_revision: 0,
            metadata,
            materials: save_materials,
            result: json!({"command": "init", "response": {}, "exit_code": 0}),
        };
        let receipt = store.save(&project.root, &request).unwrap();
        assert_eq!(receipt.outcome, RsOperationOutcome::Committed);
    }

    /// A legacy cache-backed registration: files on disk, no store item.
    fn seed_legacy_wish(
        project: &TestProject,
        wish_id: &str,
        flow_id: &str,
        run_status: &str,
        description: &str,
    ) {
        seed_legacy_wish_with_plan(project, wish_id, flow_id, run_status, description, None)
    }

    fn seed_legacy_wish_with_plan(
        project: &TestProject,
        wish_id: &str,
        flow_id: &str,
        run_status: &str,
        description: &str,
        frozen_plan: Option<Value>,
    ) {
        let plan = flow_plan(flow_id);
        let plan_bytes = match frozen_plan {
            Some(frozen) => serde_json::to_vec_pretty(&frozen).unwrap(),
            None => frozen_plan_bytes(&plan),
        };
        let plan_sha256 = crate::core::sha256_bytes(&plan_bytes);
        let run = seeded_run(wish_id, flow_id, &plan_sha256, run_status);
        let directory = format!("cache/xdesign/{wish_id}");
        let plan_uri = format!("{directory}/plan.json");
        let work_uri = format!("{directory}/work.json");
        let run_uri = format!("{directory}/run.json");
        let registry_uri = format!("{directory}/registry.json");
        let wish_uri = format!("mbse/wishes/{wish_id}.sysml");
        let work_bytes = b"{}".to_vec();
        let registry = json!({
            "schema_version": 1,
            "wish_id": wish_id,
            "flow_id": flow_id,
            "run_id": run["run_id"],
            "wish_model_uri": wish_uri,
            "plan_uri": plan_uri,
            "plan_sha256": plan_sha256,
            "work_uri": work_uri,
            "work_sha256": crate::core::sha256_bytes(&work_bytes),
            "run_record_uri": run_uri,
        });
        project.write(&plan_uri, &plan_bytes);
        project.write(&work_uri, &work_bytes);
        project.write(&run_uri, &serde_json::to_vec_pretty(&run).unwrap());
        project.write(
            &registry_uri,
            &serde_json::to_vec_pretty(&registry).unwrap(),
        );
        project.write_text(
            &wish_uri,
            &wish_source_text(wish_id, None, Some(description)),
        );
    }

    // ------------------------------------------------------------------
    // Response helpers
    // ------------------------------------------------------------------

    fn get(project_state: &AppState, path: &str) -> Response {
        handle(project_state, &Request::get(path))
    }

    fn get_query(project_state: &AppState, path: &str, query: &str) -> Response {
        handle(project_state, &Request::get(path).with_query(query))
    }

    fn post(project_state: &AppState, path: &str, body: Value) -> Response {
        handle(
            project_state,
            &Request::post(path, serde_json::to_vec(&body).unwrap()),
        )
    }

    fn post_raw(project_state: &AppState, path: &str, body: &[u8]) -> Response {
        handle(project_state, &Request::post(path, body.to_vec()))
    }

    fn ok_data(response: &Response) -> Value {
        assert_eq!(
            response.status,
            200,
            "unexpected status: {}",
            String::from_utf8_lossy(&response.body)
        );
        let value: Value = serde_json::from_slice(&response.body).unwrap();
        assert_eq!(value["ok"], true, "{value}");
        value["data"].clone()
    }

    fn error_code(response: &Response) -> (u16, String) {
        let value: Value = serde_json::from_slice(&response.body).unwrap();
        assert_eq!(value["ok"], false, "{value}");
        (
            response.status,
            value["error"]["code"].as_str().unwrap().to_owned(),
        )
    }

    // ------------------------------------------------------------------
    // Health and catalog
    // ------------------------------------------------------------------

    #[test]
    fn health_reports_missing_store() {
        let project = TestProject::new("health");
        let state = test_state(&project, FakeLifecycle::default());
        let data = ok_data(&get(&state, "/api/health"));
        assert_eq!(data["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(
            data["project_root"].as_str().unwrap(),
            project.root().to_str().unwrap()
        );
        assert_eq!(
            data["store_root"].as_str().unwrap(),
            project.store_root().to_str().unwrap()
        );
        assert_eq!(data["store_present"], false);
        assert!(data.get("agents").is_none());
        let now = data["now"].as_str().unwrap();
        assert_eq!(now.len(), 20, "{now}");
        assert!(now.ends_with('Z'), "{now}");
        assert_eq!(&now[4..5], "-");
        assert_eq!(&now[10..11], "T");
        let data = ok_data(&get(&state, "/api/wishes"));
        assert_eq!(data["wishes"], json!([]));
    }

    #[test]
    fn health_reports_a_present_store() {
        let project = TestProject::new("health-store");
        seed_store_wish(
            &project,
            "wish-store-1",
            "simple-flow",
            "active",
            Some(("Store wish", "A store wish")),
            None,
        );
        let state = test_state(&project, FakeLifecycle::default());
        let data = ok_data(&get(&state, "/api/health"));
        assert_eq!(data["store_present"], true);
    }

    #[test]
    fn flows_catalog_exposes_stages_and_checkpoints() {
        let project = TestProject::new("flows");
        let state = test_state(&project, FakeLifecycle::default());
        let data = ok_data(&get(&state, "/api/flows"));
        let flows = data["flows"].as_array().unwrap();
        assert_eq!(flows.len(), 2);

        let simple = &flows[0];
        assert_eq!(simple["flow_id"], "simple-flow");
        assert_eq!(simple["title"], "SimpleFlow");
        assert_eq!(
            simple["summary"],
            "Four gated stages: requirements, design, implement, verify."
        );
        assert_eq!(simple["custom_checks_supported"], true);
        let stages = simple["stages"].as_array().unwrap();
        assert_eq!(stages.len(), 4);
        assert_eq!(
            stages[0],
            json!({
                "stage_id": "requirements",
                "title": "Requirements",
                "goal": "Define requirements and their rationale.",
                "instructions": "State the requirements, rationale and satisfaction relationships in the project model.",
                "expected_outputs": ["checker:simpleRequirementsCheck"],
            })
        );
        assert_eq!(stages[1]["title"], "Design");
        assert_eq!(stages[2]["title"], "Implementation");
        assert_eq!(stages[3]["title"], "Verification");
        assert_eq!(stages[2]["expected_outputs"], json!(["source:src"]));
        assert_eq!(stages[3]["expected_outputs"], json!(["result:artifacts"]));

        let checkpoints = simple["checkpoints"].as_array().unwrap();
        assert_eq!(checkpoints.len(), 5);
        let requirements = &checkpoints[0];
        assert_eq!(requirements["checkpoint_id"], "simpleRequirementsAccepted");
        assert_eq!(requirements["stage_id"], "requirements");
        assert_eq!(requirements["kind"], "syntactic");
        assert_eq!(requirements["title"], "simpleRequirementsCheck");
        assert_eq!(requirements["agent_may_judge"], false);
        assert_eq!(requirements["human_may_judge"], false);
        assert_eq!(
            requirements["checker"]["command_uri"],
            "tools/checks/simple_requirements.py"
        );
        assert_eq!(
            requirements["checker"]["modes"],
            json!(["default", "library"])
        );
        assert_eq!(
            requirements["checker"]["establishes"],
            json!(["identifier_conformance", "trace_conformance"])
        );
        assert_eq!(
            requirements["checker"]["rules"][0]["rule_id"],
            "requirements-carry-rationale"
        );
        let verification = &checkpoints[4];
        assert_eq!(verification["checkpoint_id"], "verificationAccepted");
        assert_eq!(verification["kind"], "semantic");
        assert_eq!(verification["title"], "verificationAccepted");
        assert_eq!(verification["checker"], Value::Null);
        assert_eq!(verification["agent_may_judge"], true);

        let wish_verify = &flows[1];
        assert_eq!(wish_verify["flow_id"], "wish-verify");
        assert_eq!(wish_verify["title"], "WishVerifyFlow");
        assert_eq!(wish_verify["custom_checks_supported"], false);
        assert_eq!(wish_verify["stages"].as_array().unwrap().len(), 2);
        assert_eq!(wish_verify["stages"][0]["title"], "Wish");
        assert_eq!(wish_verify["stages"][1]["title"], "Verification");

        assert!(data.get("agents").is_none());
    }

    // ------------------------------------------------------------------
    // Wish list and detail
    // ------------------------------------------------------------------

    #[test]
    fn wishes_list_merges_store_and_legacy_and_sorts_by_status() {
        let project = TestProject::new("list");
        seed_store_wish(
            &project,
            "wish-a-delivered",
            "simple-flow",
            "completed",
            Some(("Delivered wish", "Delivered description")),
            None,
        );
        seed_store_wish(
            &project,
            "wish-b-active",
            "simple-flow",
            "active",
            None,
            Some("Active description"),
        );
        seed_legacy_wish(
            &project,
            "wish-c-legacy",
            "wish-verify",
            "completed",
            "Legacy text",
        );
        let state = test_state(&project, FakeLifecycle::default());
        let data = ok_data(&get(&state, "/api/wishes"));
        let wishes = data["wishes"].as_array().unwrap();
        let ids: Vec<&str> = wishes
            .iter()
            .map(|wish| wish["wish_id"].as_str().unwrap())
            .collect();
        assert_eq!(
            ids,
            vec!["wish-b-active", "wish-a-delivered", "wish-c-legacy"]
        );

        let active = &wishes[0];
        assert_eq!(active["title"], "wish-b-active");
        assert_eq!(active["description"], "Active description");
        assert_eq!(active["flow_id"], "simple-flow");
        assert_eq!(active["status"], "active");
        assert_eq!(active["current_stage"], "design");
        assert_eq!(active["head_revision"], 1);
        assert_eq!(active["source"], "store");
        assert_eq!(active["updated_at"], Value::Null);

        let delivered = &wishes[1];
        assert_eq!(delivered["title"], "Delivered wish");
        assert_eq!(delivered["status"], "delivered");
        assert_eq!(delivered["current_stage"], Value::Null);
        assert_eq!(delivered["source"], "store");

        let legacy = &wishes[2];
        assert_eq!(legacy["title"], "wish-c-legacy");
        assert_eq!(legacy["description"], "Legacy text");
        assert_eq!(legacy["flow_id"], "wish-verify");
        assert_eq!(legacy["status"], "delivered");
        assert_eq!(legacy["head_revision"], Value::Null);
        assert_eq!(legacy["source"], "legacy");
    }

    #[test]
    fn wish_list_falls_back_to_marker_title_first() {
        let (title, description) = wish_title_description(
            "wish-1",
            Some(
                "// xdesign-wish: {\"title\":\"Marker title\",\"description\":\"Marker description\"}\n\
                 package Wish_1 {}\n",
            ),
        );
        assert_eq!(title, "Marker title");
        assert_eq!(description.as_deref(), Some("Marker description"));

        let (title, description) = wish_title_description(
            "wish-1",
            Some("package Wish_1 {\n attribute redefines description = \"Attr text\";\n}\n"),
        );
        assert_eq!(title, "wish-1");
        assert_eq!(description.as_deref(), Some("Attr text"));

        let (title, description) = wish_title_description("wish-1", Some("package Wish_1 {}\n"));
        assert_eq!(title, "wish-1");
        assert_eq!(description, None);

        let (title, description) = wish_title_description("wish-1", None);
        assert_eq!(title, "wish-1");
        assert_eq!(description, None);
    }

    #[test]
    fn wish_detail_projects_plan_run_history_and_events() {
        let project = TestProject::new("detail-store");
        let run = seed_store_wish(
            &project,
            "wish-detail",
            "simple-flow",
            "active",
            Some(("Detail wish", "Detail description")),
            None,
        );
        let state = test_state(&project, FakeLifecycle::default());
        let data = ok_data(&get(&state, "/api/wishes/wish-detail"));
        assert_eq!(data["wish_id"], "wish-detail");
        assert_eq!(data["title"], "Detail wish");
        assert_eq!(data["description"], "Detail description");
        assert_eq!(data["flow_id"], "simple-flow");
        assert_eq!(data["status"], "active");
        assert_eq!(data["current_stage"], "design");
        assert_eq!(data["head_revision"], 1);
        assert_eq!(data["source"], "store");
        assert_eq!(data["plan"]["plan_id"], "designedSimpleFlow");
        assert_eq!(data["plan"]["plan_sha256"].as_str().unwrap().len(), 64);
        assert_eq!(data["plan"]["stages"].as_array().unwrap().len(), 4);
        assert_eq!(
            data["plan"]["stages"][0]["next_stage_ids"],
            json!(["design"])
        );
        assert_eq!(
            data["plan"]["stages"][0]["exit_checkpoint_ids"],
            json!(["simpleRequirementsAccepted"])
        );
        let checkpoints = data["plan"]["checkpoints"].as_array().unwrap();
        assert_eq!(checkpoints.len(), 5);
        assert_eq!(checkpoints[0]["title"], "simpleRequirementsCheck");
        assert_eq!(
            checkpoints[0]["checker"]["binding"],
            json!({"mode": "default"})
        );
        assert_eq!(checkpoints[4]["kind"], "semantic");
        assert_eq!(checkpoints[4]["checker"], Value::Null);

        assert_eq!(data["run"]["run_id"], run["run_id"]);
        assert_eq!(data["run"]["execution_type"], "FlowRun");
        assert_eq!(data["run"]["plan_id"], "designedSimpleFlow");
        assert_eq!(data["run"]["status"], "active");
        assert_eq!(data["run"]["current_visit"], 1);
        assert_eq!(data["run"]["visits"].as_array().unwrap().len(), 4);

        let history = data["history"].as_array().unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0]["revision"], 1);
        assert_eq!(history[0]["material_count"], 5);
        assert!(history[0]["operation_id"].is_string());

        let events = data["events"].as_array().unwrap();
        let kinds: Vec<&str> = events
            .iter()
            .map(|event| event["kind"].as_str().unwrap())
            .collect();
        assert_eq!(kinds, vec!["init", "activity", "assessment", "advance"]);
        assert_eq!(events[0]["title"], "Wish initialized");
        assert_eq!(events[0]["detail"], "flow simple-flow");
        assert_eq!(events[0]["actor"], "xdesign init");
        assert_eq!(events[0]["stage_id"], Value::Null);
        assert_eq!(events[1]["stage_id"], "requirements");
        assert_eq!(
            events[1]["title"],
            "Record requirements stage input artifacts"
        );
        assert_eq!(events[1]["detail"], "1 artifact");
        assert_eq!(events[1]["at"], "2026-10-20T10:00:00Z");
        assert_eq!(events[1]["actor"], "test operator");
        assert_eq!(events[2]["kind"], "assessment");
        assert_eq!(events[2]["title"], "simpleRequirementsCheck");
        assert_eq!(events[2]["actor"], "requirementsCheck");
        assert_eq!(events[2]["outcome"], "passed");
        assert_eq!(events[2]["detail"], "All declared checker rules passed");
        assert_eq!(
            events[2]["evidence_uris"],
            json!(["cache/xdesign/wish-detail/reports/requirements_1_report.json"])
        );
        assert_eq!(events[3]["kind"], "advance");
        assert_eq!(events[3]["title"], "Entered design");
    }

    #[test]
    fn wish_detail_completed_run_ends_with_run_completed_event() {
        let project = TestProject::new("detail-delivered");
        seed_store_wish(
            &project,
            "wish-done",
            "simple-flow",
            "completed",
            Some(("Done", "Done")),
            None,
        );
        let state = test_state(&project, FakeLifecycle::default());
        let data = ok_data(&get(&state, "/api/wishes/wish-done"));
        assert_eq!(data["status"], "delivered");
        assert_eq!(data["current_stage"], Value::Null);
        let events = data["events"].as_array().unwrap();
        assert_eq!(events.last().unwrap()["title"], "Run completed");
        let advances: Vec<&str> = events
            .iter()
            .filter(|event| event["kind"] == "advance")
            .map(|event| event["title"].as_str().unwrap())
            .collect();
        assert_eq!(
            advances,
            vec![
                "Entered design",
                "Entered implement",
                "Entered verify",
                "Run completed"
            ]
        );
    }

    #[test]
    fn wish_detail_legacy_has_plan_run_and_empty_history() {
        let project = TestProject::new("detail-legacy");
        seed_legacy_wish(
            &project,
            "wish-legacy",
            "wish-verify",
            "completed",
            "Legacy detail",
        );
        let state = test_state(&project, FakeLifecycle::default());
        let data = ok_data(&get(&state, "/api/wishes/wish-legacy"));
        assert_eq!(data["source"], "legacy");
        assert_eq!(data["head_revision"], Value::Null);
        assert_eq!(data["status"], "delivered");
        assert_eq!(data["description"], "Legacy detail");
        assert_eq!(data["history"], json!([]));
        assert_eq!(data["plan"]["plan_id"], "designedWishVerifyFlow");
        assert_eq!(data["run"]["status"], "completed");
        let events = data["events"].as_array().unwrap();
        assert_eq!(events.last().unwrap()["title"], "Run completed");
    }

    #[test]
    fn unknown_wish_and_invalid_id_are_404() {
        let project = TestProject::new("missing");
        let state = test_state(&project, FakeLifecycle::default());
        let (status, code) = error_code(&get(&state, "/api/wishes/wish-nope"));
        assert_eq!((status, code.as_str()), (404, "unknown_wish"));
        let (status, code) = error_code(&get(&state, "/api/wishes/bad%20id"));
        assert_eq!((status, code.as_str()), (404, "not_found"));
        let (status, code) = error_code(&get(&state, "/api/nope"));
        assert_eq!((status, code.as_str()), (404, "not_found"));
        let (status, code) = error_code(&post(&state, "/api/health", json!({})));
        assert_eq!((status, code.as_str()), (405, "method_not_allowed"));
    }

    // ------------------------------------------------------------------
    // Create wish
    // ------------------------------------------------------------------

    fn valid_create_body() -> Value {
        json!({
            "wish_id": "wish-web-0001",
            "title": "Add energy budget check",
            "description": "Introduce a deterministic energy budget gate.",
            "flow_id": "simple-flow",
            "checks": [
                {"checkpoint_id": "simpleRequirementsAccepted", "mode": "default"},
                {"checkpoint_id": "simpleDesignAccepted", "mode": "library",
                 "library_id": "energy-budget-positive"},
            ],
        })
    }

    #[test]
    fn create_wish_validation_rules() {
        let project = TestProject::new("create-valid");
        let lifecycle = FakeLifecycle::default();
        let state = test_state(&project, lifecycle.clone());
        let cases: Vec<(Value, &str)> = vec![
            (json!([]), "invalid_json"),
            (
                json!({"wish_id": "bad id", "title": "T", "flow_id": "simple-flow"}),
                "invalid_id",
            ),
            (
                json!({"wish_id": "w1", "title": "", "flow_id": "simple-flow"}),
                "invalid_title",
            ),
            (
                json!({"wish_id": "w1", "title": "x".repeat(201), "flow_id": "simple-flow"}),
                "invalid_title",
            ),
            (
                json!({"wish_id": "w1", "title": "T", "flow_id": "simple-flow",
                       "description": "x".repeat(4001)}),
                "invalid_description",
            ),
            (
                json!({"wish_id": "w1", "title": "T", "flow_id": "other-flow"}),
                "invalid_flow",
            ),
            (
                json!({"wish_id": "w1", "title": "T", "flow_id": "wish-verify",
                       "checks": [{"checkpoint_id": "simpleRequirementsAccepted"}]}),
                "invalid_checks",
            ),
            (
                json!({"wish_id": "w1", "title": "T", "flow_id": "simple-flow",
                       "checks": [{"checkpoint_id": "bogus", "mode": "default"}]}),
                "invalid_checks",
            ),
            (
                json!({"wish_id": "w1", "title": "T", "flow_id": "simple-flow",
                       "checks": [
                           {"checkpoint_id": "simpleRequirementsAccepted", "mode": "default"},
                           {"checkpoint_id": "simpleRequirementsAccepted", "mode": "default"}]}),
                "invalid_checks",
            ),
            (
                json!({"wish_id": "w1", "title": "T", "flow_id": "simple-flow",
                       "checks": [{"checkpoint_id": "simpleRequirementsAccepted", "mode": "command",
                                   "command_uri": "tools/checks/x.py"}]}),
                "invalid_checks",
            ),
            (
                json!({"wish_id": "w1", "title": "T", "flow_id": "simple-flow",
                       "checks": [{"checkpoint_id": "simpleDesignAccepted", "mode": "cli",
                                   "cli": "codex", "model": "m"}]}),
                "invalid_checks",
            ),
            (
                json!({"wish_id": "w1", "title": "T", "flow_id": "simple-flow",
                       "checks": [{"checkpoint_id": "simpleDesignAccepted", "mode": "default",
                                   "cli": "codex", "model": "m"}]}),
                "invalid_checks",
            ),
        ];
        for (body, expected_code) in cases {
            let response = post(&state, "/api/wishes", body.clone());
            let (status, code) = error_code(&response);
            assert_eq!(status, 400, "case {body}: {code}");
            assert_eq!(code, expected_code, "case {body}");
        }
        assert!(
            lifecycle.calls().is_empty(),
            "validation must never run the lifecycle"
        );
        assert!(!project.root().join("cache/web").exists());
    }

    #[test]
    fn create_wish_writes_source_and_checks_then_runs_init() {
        let project = TestProject::new("create-ok");
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_reply(0, checkpoint_list_reply()));
        lifecycle.push(lifecycle_reply(
            0,
            json!({"schema_version": 1, "wish_id": "wish-web-0001",
                   "flow_id": "simple-flow", "status": "pending",
                   "completed": false, "state_token": "token", "store_revision": 1,
                   "work": null}),
        ));
        let state = test_state(&project, lifecycle.clone());
        let data = ok_data(&post(&state, "/api/wishes", valid_create_body()));
        assert_eq!(data["wish_id"], "wish-web-0001");
        assert_eq!(data["flow_id"], "simple-flow");
        assert_eq!(data["init"]["store_revision"], 1);
        assert!(data["stdout"].as_str().unwrap().contains("wish-web-0001"));
        assert_eq!(data["stderr"], "");

        // Source file: marker plus SysML package.
        let source = project.read_text("cache/web/wishes/wish-web-0001.sysml");
        assert!(source.starts_with("// xdesign-wish: "), "{source}");
        assert!(source.contains("package Wish_wish_web_0001 {"), "{source}");
        let marker = source
            .lines()
            .next()
            .unwrap()
            .strip_prefix("// xdesign-wish: ")
            .unwrap();
        let marker: Value = serde_json::from_str(marker).unwrap();
        assert_eq!(marker["title"], "Add energy budget check");
        assert_eq!(
            marker["description"],
            "Introduce a deterministic energy budget gate."
        );
        assert_eq!(marker["flow_id"], "simple-flow");

        // Checks file: only when at least one non-default binding.
        let config: Value =
            serde_json::from_str(&project.read_text("cache/web/checks/wish-web-0001.json"))
                .unwrap();
        assert_eq!(config["schema_version"], 1);
        let checks = config["checks"].as_array().unwrap();
        assert_eq!(checks.len(), 2);
        assert_eq!(checks[0]["mode"], "default");
        assert_eq!(checks[1]["mode"], "library");
        assert_eq!(checks[1]["library_id"], "energy-budget-positive");

        let calls = lifecycle.calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0, vec!["checkpoint", "list"]);
        assert_eq!(
            calls[1].0,
            vec![
                "init",
                "wish-web-0001",
                "--flow",
                "simple-flow",
                "--wish",
                "cache/web/wishes/wish-web-0001.sysml",
                "--checks",
                "cache/web/checks/wish-web-0001.json",
            ]
        );
        assert_eq!(calls[1].1, project.root());
    }

    #[test]
    fn create_wish_without_custom_bindings_skips_the_checks_file() {
        let project = TestProject::new("create-default");
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_reply(0, json!({"schema_version": 1})));
        let state = test_state(&project, lifecycle.clone());
        let body = json!({
            "wish_id": "wish-default",
            "title": "Default bindings",
            "flow_id": "simple-flow",
        });
        ok_data(&post(&state, "/api/wishes", body));
        assert!(!project.root().join("cache/web/checks").exists());
        assert_eq!(
            lifecycle.call_args(),
            vec![vec![
                "init",
                "wish-default",
                "--flow",
                "simple-flow",
                "--wish",
                "cache/web/wishes/wish-default.sysml",
            ]]
        );
    }

    #[test]
    fn create_wish_rejects_existing_store_and_legacy_wishes() {
        let project = TestProject::new("create-exists");
        seed_store_wish(
            &project,
            "wish-taken",
            "simple-flow",
            "active",
            None,
            Some("Taken"),
        );
        let lifecycle = FakeLifecycle::default();
        let state = test_state(&project, lifecycle.clone());
        let body = json!({"wish_id": "wish-taken", "title": "T", "flow_id": "simple-flow"});
        let (status, code) = error_code(&post(&state, "/api/wishes", body));
        assert_eq!((status, code.as_str()), (409, "already_exists"));
        assert!(lifecycle.calls().is_empty());

        seed_legacy_wish(
            &project,
            "wish-legacy-taken",
            "simple-flow",
            "completed",
            "Legacy",
        );
        let body = json!({"wish_id": "wish-legacy-taken", "title": "T", "flow_id": "simple-flow"});
        let (status, code) = error_code(&post(&state, "/api/wishes", body));
        assert_eq!((status, code.as_str()), (409, "already_exists"));
        assert!(lifecycle.calls().is_empty());
    }

    #[test]
    fn create_wish_reports_init_failure() {
        let project = TestProject::new("create-fail");
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_failure(2, "xdesign: invalid checks config"));
        let state = test_state(&project, lifecycle);
        let (status, code) = error_code(&post(&state, "/api/wishes", valid_create_body()));
        assert_eq!((status, code.as_str()), (500, "init_failed"));
    }

    // ------------------------------------------------------------------
    // Work and check
    // ------------------------------------------------------------------

    fn next_reply(wish_id: &str, flow_id: &str, stage: &str, semantic: bool) -> Value {
        let checkpoints = if semantic {
            json!([{"checkpoint_id": "wishAccepted", "kind": "semantic", "title": "wishAccepted", "checker": null}])
        } else {
            json!([{"checkpoint_id": "simpleDesignAccepted", "kind": "syntactic",
                    "title": "simpleDesignCheck", "checker": {"checker_id": "simpleDesignCheck"}}])
        };
        json!({
            "schema_version": 1,
            "wish_id": wish_id,
            "run_id": "run-1",
            "flow_id": flow_id,
            "status": "active",
            "completed": false,
            "state_token": "state-token-1",
            "store_revision": 1,
            "work": {
                "stage_id": stage,
                "visit_index": 0,
                "artifact_revision": 0,
                "artifact_snapshot_digest": hex('a', 64),
                "goal": "Goal",
                "instructions": "Instructions",
                "inputs": ["wish:model"],
                "input_artifacts": [format!("cache/wishes/{wish_id}.sysml")],
                "expected_outputs": ["result:artifacts"],
                "frozen_output_paths": [],
                "exit_checkpoints": checkpoints,
                "current_assessments": [],
                "correction_reasons": [],
                "collection_diagnostics": [],
                "candidate_artifacts": [],
                "candidate_snapshot_digest": hex('b', 64),
            },
        })
    }

    #[test]
    fn work_returns_the_next_response_and_raw_streams() {
        let project = TestProject::new("work");
        seed_store_wish(
            &project,
            "wish-work",
            "simple-flow",
            "active",
            Some(("Work", "Work")),
            None,
        );
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_reply(
            0,
            next_reply("wish-work", "simple-flow", "design", false),
        ));
        let state = test_state(&project, lifecycle.clone());
        let data = ok_data(&get(&state, "/api/wishes/wish-work/work"));
        assert_eq!(data["exit_code"], 0);
        assert_eq!(data["response"]["work"]["stage_id"], "design");
        assert_eq!(data["response"]["state_token"], "state-token-1");
        assert!(data["stdout"].as_str().unwrap().contains("\"design\""));
        assert_eq!(data["stderr"], "");
        assert_eq!(
            lifecycle.call_args(),
            vec![vec!["next".to_owned(), "wish-work".to_owned()]]
        );
    }

    #[test]
    fn work_failure_is_an_envelope_error() {
        let project = TestProject::new("work-fail");
        seed_store_wish(
            &project,
            "wish-work-fail",
            "simple-flow",
            "active",
            None,
            Some("Work"),
        );
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_failure(2, "xdesign: cannot load run"));
        let state = test_state(&project, lifecycle);
        let (status, code) = error_code(&get(&state, "/api/wishes/wish-work-fail/work"));
        assert_eq!((status, code.as_str()), (500, "work_failed"));
    }

    #[test]
    fn work_unknown_wish_does_not_spawn_a_process() {
        let project = TestProject::new("work-missing");
        let lifecycle = FakeLifecycle::default();
        let state = test_state(&project, lifecycle.clone());
        let (status, code) = error_code(&get(&state, "/api/wishes/wish-missing/work"));
        assert_eq!((status, code.as_str()), (404, "unknown_wish"));
        assert!(lifecycle.calls().is_empty());
    }

    #[test]
    fn check_writes_the_result_file_and_returns_the_cli_response() {
        let project = TestProject::new("check");
        seed_store_wish(
            &project,
            "wish-check",
            "wish-verify",
            "active",
            Some(("Check", "Check")),
            None,
        );
        project.write_text("docs/note.md", "# Evidence\n");
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_reply(
            0,
            next_reply("wish-check", "wish-verify", "wish", true),
        ));
        lifecycle.push(lifecycle_reply(
            0,
            json!({"schema_version": 1, "wish_id": "wish-check", "advance": "enter_next_stage",
                   "assessed": {"stage_id": "wish", "visit_index": 0}}),
        ));
        let state = test_state(&project, lifecycle.clone());
        let body = json!({
            "description": "Web check run for wish",
            "performed_by": "operator",
            "artifacts": [{"uri": "docs/note.md"}],
            "judgment": {
                "authority": "agent",
                "identity": "operator",
                "outcome": "passed",
                "rationale": "Reviewed the retained evidence set.",
            },
        });
        let data = ok_data(&post(&state, "/api/wishes/wish-check/check", body));
        assert_eq!(data["exit_code"], 0);
        assert_eq!(data["response"]["advance"], "enter_next_stage");
        assert!(data["stdout"]
            .as_str()
            .unwrap()
            .contains("enter_next_stage"));

        let calls = lifecycle.calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0, vec!["next", "wish-check"]);
        assert_eq!(calls[1].0[0], "check");
        assert_eq!(calls[1].0[1], "wish-check");
        assert_eq!(calls[1].0[2], "--result");
        let result_uri = &calls[1].0[3];
        assert!(
            result_uri.starts_with("cache/web/attempts/wish-check-"),
            "{result_uri}"
        );
        assert!(result_uri.ends_with(".json"), "{result_uri}");

        let result: Value = serde_json::from_str(&project.read_text(result_uri)).unwrap();
        assert_eq!(result["wish_id"], "wish-check");
        assert_eq!(result["run_id"], "run-1");
        assert_eq!(result["stage_id"], "wish");
        assert_eq!(result["visit_index"], 0);
        assert_eq!(result["base_revision"], 0);
        assert_eq!(result["state_token"], "state-token-1");
        assert_eq!(result["description"], "Web check run for wish");
        assert_eq!(result["performed_by"], "operator");
        assert_eq!(
            result["artifacts"],
            json!([{
                "uri": "docs/note.md",
                "sha256": crate::core::sha256_bytes(b"# Evidence\n"),
            }])
        );
        assert_eq!(result["judgment"]["artifact_snapshot_digest"], hex('b', 64));
        assert_eq!(result["judgment"]["authority"], "agent");
        assert_eq!(result["judgment"]["outcome"], "passed");
    }

    #[test]
    fn check_defaults_description_and_accepts_a_blocked_exit_code() {
        let project = TestProject::new("check-blocked");
        seed_store_wish(
            &project,
            "wish-blocked",
            "simple-flow",
            "active",
            None,
            Some("Blocked"),
        );
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_reply(
            0,
            next_reply("wish-blocked", "simple-flow", "design", false),
        ));
        lifecycle.push(lifecycle_reply(
            1,
            json!({"schema_version": 1, "wish_id": "wish-blocked",
                   "advance": "remain_blocked"}),
        ));
        let state = test_state(&project, lifecycle.clone());
        let data = ok_data(&post(
            &state,
            "/api/wishes/wish-blocked/check",
            json!({"artifacts": []}),
        ));
        assert_eq!(data["exit_code"], 1);
        assert_eq!(data["response"]["advance"], "remain_blocked");
        let calls = lifecycle.calls();
        let result_uri = &calls[1].0[3];
        let result: Value = serde_json::from_str(&project.read_text(result_uri)).unwrap();
        assert_eq!(result["description"], "Web check, stage design");
        assert_eq!(result["performed_by"], "xdesign web operator");
        assert!(result["judgment"].is_null());
    }

    #[test]
    fn check_engine_failure_carries_stdout_and_stderr_detail() {
        let project = TestProject::new("check-fail");
        seed_store_wish(
            &project,
            "wish-check-fail",
            "simple-flow",
            "active",
            None,
            Some("Fail"),
        );
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_reply(
            0,
            next_reply("wish-check-fail", "simple-flow", "design", false),
        ));
        lifecycle.push(lifecycle_failure(2, "xdesign: broken continuation"));
        let state = test_state(&project, lifecycle);
        let response = post(&state, "/api/wishes/wish-check-fail/check", json!({}));
        let (status, code) = error_code(&response);
        assert_eq!((status, code.as_str()), (500, "check_failed"));
        let value: Value = serde_json::from_slice(&response.body).unwrap();
        assert_eq!(value["error"]["detail"]["exit_code"], 2);
        assert!(value["error"]["detail"]["stderr"]
            .as_str()
            .unwrap()
            .contains("broken continuation"));
    }

    #[test]
    fn check_refuses_delivered_and_judgment_on_a_syntactic_stage() {
        let project = TestProject::new("check-refuse");
        seed_store_wish(
            &project,
            "wish-done-check",
            "simple-flow",
            "completed",
            None,
            Some("Done"),
        );
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_reply(
            0,
            json!({"schema_version": 1, "wish_id": "wish-done-check",
                   "completed": true, "status": "completed", "run_id": "run-1",
                   "state_token": "t", "work": null}),
        ));
        lifecycle.push(lifecycle_reply(
            0,
            next_reply("wish-done-check", "simple-flow", "design", false),
        ));
        let state = test_state(&project, lifecycle);
        let (status, code) = error_code(&post(
            &state,
            "/api/wishes/wish-done-check/check",
            json!({}),
        ));
        assert_eq!((status, code.as_str()), (409, "already_delivered"));

        // A fresh wish on a syntactic stage rejects a supplied judgment.
        seed_store_wish(
            &project,
            "wish-syntactic",
            "simple-flow",
            "active",
            None,
            Some("Syntactic"),
        );
        let (status, code) = error_code(&post(
            &state,
            "/api/wishes/wish-syntactic/check",
            json!({"judgment": {"authority": "agent", "identity": "op",
                                "outcome": "passed", "rationale": "ok"}}),
        ));
        assert_eq!((status, code.as_str()), (400, "invalid_judgment"));
    }

    #[test]
    fn check_rejects_missing_artifacts_before_invoking_the_cli() {
        let project = TestProject::new("check-artifact");
        seed_store_wish(
            &project,
            "wish-artifact",
            "simple-flow",
            "active",
            None,
            Some("Artifact"),
        );
        let lifecycle = FakeLifecycle::default();
        let state = test_state(&project, lifecycle.clone());
        let response = post(
            &state,
            "/api/wishes/wish-artifact/check",
            json!({"artifacts": [{"uri": "docs/missing.md"}]}),
        );
        let (status, code) = error_code(&response);
        assert_eq!((status, code.as_str()), (400, "invalid_artifact"));
        assert!(lifecycle.calls().is_empty());
    }

    // ------------------------------------------------------------------
    // Material endpoint
    // ------------------------------------------------------------------

    #[test]
    fn material_serves_head_store_materials() {
        let project = TestProject::new("material-store");
        seed_store_wish(
            &project,
            "wish-material",
            "simple-flow",
            "active",
            Some(("Material", "Material")),
            None,
        );
        let state = test_state(&project, FakeLifecycle::default());
        let response = get_query(
            &state,
            "/api/wishes/wish-material/material",
            "uri=cache%2Fwishes%2Fwish-material.sysml",
        );
        assert_eq!(response.status, 200);
        assert_eq!(response.content_type, "text/plain; charset=utf-8");
        let text = String::from_utf8(response.body).unwrap();
        assert!(text.contains("// xdesign-wish:"), "{text}");

        let response = get_query(
            &state,
            "/api/wishes/wish-material/material",
            "uri=cache%2Fxdesign%2Fwish-material%2Fplan.json",
        );
        assert_eq!(response.status, 200);
        assert_eq!(response.content_type, "application/json; charset=utf-8");

        let (status, code) = error_code(&get_query(
            &state,
            "/api/wishes/wish-material/material",
            "uri=docs%2Findex.md",
        ));
        assert_eq!((status, code.as_str()), (404, "material_not_found"));
        let (status, code) = error_code(&get(&state, "/api/wishes/wish-material/material"));
        assert_eq!((status, code.as_str()), (400, "missing_uri"));
        let (status, code) = error_code(&get_query(
            &state,
            "/api/wishes/wish-material/material",
            "uri=..%2F..%2Fetc%2Fpasswd",
        ));
        assert_eq!((status, code.as_str()), (400, "invalid_uri"));
    }

    #[test]
    fn material_rejects_payloads_above_the_limit() {
        let project = TestProject::new("material-large");
        let plan = flow_plan("simple-flow");
        let plan_bytes = frozen_plan_bytes(&plan);
        let plan_sha256 = crate::core::sha256_bytes(&plan_bytes);
        let run = seeded_run("wish-large", "simple-flow", &plan_sha256, "active");
        let directory = "cache/xdesign/wish-large";
        let plan_uri = format!("{directory}/plan.json");
        let work_uri = format!("{directory}/work.json");
        let run_uri = format!("{directory}/run.json");
        let registry_uri = format!("{directory}/registry.json");
        let wish_uri = "cache/wishes/wish-large.sysml".to_owned();
        let run_bytes = serde_json::to_vec_pretty(&run).unwrap();
        let work_bytes = b"{}".to_vec();
        let source = wish_source_text("wish-large", None, Some("Large"));
        let registry = json!({
            "schema_version": 1, "wish_id": "wish-large", "flow_id": "simple-flow",
            "run_id": run["run_id"], "wish_model_uri": wish_uri,
            "plan_uri": plan_uri, "plan_sha256": plan_sha256,
            "work_uri": work_uri, "work_sha256": crate::core::sha256_bytes(&work_bytes),
            "run_record_uri": run_uri,
        });
        let registry_bytes = serde_json::to_vec_pretty(&registry).unwrap();
        let large_uri = format!("{directory}/large.bin");
        let large_bytes = vec![b'x'; (MAX_MATERIAL_BYTES + 1) as usize];
        project.write(&plan_uri, &plan_bytes);
        project.write(&work_uri, &work_bytes);
        project.write(&run_uri, &run_bytes);
        project.write(&registry_uri, &registry_bytes);
        project.write_text(&wish_uri, &source);
        project.write(&large_uri, &large_bytes);
        let registry_uri_for_save = registry_uri.clone();
        let run_uri_for_save = run_uri.clone();
        save_wish_request(
            &project,
            "wish-large",
            "simple-flow",
            &registry_uri_for_save,
            &run_uri_for_save,
            vec![
                (registry_uri, "registration", registry_bytes),
                (plan_uri, "frozen-plan", plan_bytes),
                (work_uri, "frozen-work", work_bytes),
                (run_uri, "run-record", run_bytes),
                (wish_uri, "wish-source", source.into_bytes()),
                (large_uri, "checker-input", large_bytes),
            ],
        );
        let state = test_state(&project, FakeLifecycle::default());
        let response = get_query(
            &state,
            "/api/wishes/wish-large/material",
            "uri=cache%2Fxdesign%2Fwish-large%2Flarge.bin",
        );
        let (status, code) = error_code(&response);
        assert_eq!((status, code.as_str()), (413, "material_too_large"));
    }

    #[test]
    fn material_serves_only_legacy_run_uris_and_the_wish_prefix() {
        let project = TestProject::new("material-legacy");
        seed_legacy_wish(
            &project,
            "wish-legacy-mat",
            "simple-flow",
            "completed",
            "Legacy",
        );
        // Plan is under the wish prefix; evidence URI appears in the run record.
        project.write_text(
            "cache/xdesign/wish-legacy-mat/reports/requirements_1_report.json",
            "{\"ok\":true}",
        );
        project.write_text("docs/private.md", "private");
        let state = test_state(&project, FakeLifecycle::default());
        let response = get_query(
            &state,
            "/api/wishes/wish-legacy-mat/material",
            "uri=cache%2Fxdesign%2Fwish-legacy-mat%2Fplan.json",
        );
        assert_eq!(response.status, 200);
        assert_eq!(response.content_type, "application/json; charset=utf-8");

        let response = get_query(
            &state,
            "/api/wishes/wish-legacy-mat/material",
            "uri=cache%2Fxdesign%2Fwish-legacy-mat%2Freports%2Frequirements_1_report.json",
        );
        assert_eq!(response.status, 200);
        assert_eq!(response.content_type, "application/json; charset=utf-8");

        let (status, code) = error_code(&get_query(
            &state,
            "/api/wishes/wish-legacy-mat/material",
            "uri=docs%2Fprivate.md",
        ));
        assert_eq!((status, code.as_str()), (404, "material_not_found"));
    }

    // ------------------------------------------------------------------
    // Checkpoint library endpoints (round 2)
    // ------------------------------------------------------------------

    fn checkpoint_list_reply() -> Value {
        json!({
            "schema_version": 1,
            "library_uri": "checkpoints/library.json",
            "issues": [],
            "checkpoints": [
                {
                    "checkpoint_id": "energy-budget-positive",
                    "title": "Energy budget stays non-negative",
                    "kind": "syntactic",
                    "builtin": false,
                    "version": 2,
                    "updated_at": "2026-10-26T00:00:00Z",
                    "tags": ["energy", "design"],
                    "executor": {
                        "mode": "script",
                        "command_uri": "checkpoints/scripts/energy_budget_positive.py",
                    },
                    "rule_count": 2,
                    "criteria_count": 0,
                },
                {
                    "checkpoint_id": "builtin-simple-requirements",
                    "title": "SimpleFlow requirements checker",
                    "kind": "syntactic",
                    "builtin": true,
                    "version": 1,
                    "updated_at": null,
                    "tags": [],
                    "executor": {
                        "mode": "script",
                        "command_uri": "tools/checks/simple_requirements.py",
                    },
                    "rule_count": 2,
                    "criteria_count": 0,
                },
                {
                    "checkpoint_id": "verification-evidence-complete",
                    "title": "Verification evidence answers every requirement",
                    "kind": "semantic",
                    "builtin": false,
                    "version": 3,
                    "updated_at": "2026-10-26T00:00:00Z",
                    "tags": ["verification"],
                    "executor": null,
                    "rule_count": 0,
                    "criteria_count": 2,
                },
            ],
        })
    }

    fn checkpoint_show_reply(checkpoint_id: &str) -> Value {
        json!({
            "schema_version": 1,
            "checkpoint": {
                "checkpoint_id": checkpoint_id,
                "title": "Energy budget stays non-negative",
                "description": "Deterministic check that no declared energy budget is negative.",
                "kind": "syntactic",
                "builtin": false,
                "tags": ["energy"],
                "rules": [{
                    "rule_id": "budget-declared",
                    "condition": "Every actuator declares a numeric energyBudget.",
                    "reads": ["mbse/model/design.sysml"],
                }],
                "inputs": ["mbse/model/design.sysml"],
                "executor": {
                    "mode": "script",
                    "command_uri": "checkpoints/scripts/energy_budget_positive.py",
                },
                "version": 2,
                "created_at": "2026-10-26T00:00:00Z",
                "updated_at": "2026-10-26T00:00:00Z",
                "source": "generated",
                "history": [],
                "script_exists": true,
            },
        })
    }

    fn checkpoint_save_reply(checkpoint_id: &str) -> Value {
        json!({
            "schema_version": 1,
            "checkpoint": {
                "checkpoint_id": checkpoint_id,
                "title": "Energy budget stays non-negative",
                "kind": "syntactic",
                "builtin": false,
                "version": 2,
            },
            "script_uri": "checkpoints/scripts/energy_budget_positive.py",
            "created": true,
            "version": 2,
        })
    }

    fn valid_checkpoint_save_body() -> Value {
        json!({
            "checkpoint_id": "energy-budget-positive",
            "title": "Energy budget stays non-negative",
            "description": "Deterministic check that no declared energy budget is negative.",
            "kind": "syntactic",
            "tags": ["energy", "design"],
            "rules": [{
                "rule_id": "budget-declared",
                "condition": "Every actuator declares a numeric energyBudget.",
                "reads": ["mbse/model/design.sysml"],
            }],
            "inputs": ["mbse/model/design.sysml"],
            "executor": {
                "mode": "script",
                "command_uri": "checkpoints/scripts/energy_budget_positive.py",
            },
            "create_script": "scaffold",
        })
    }

    #[test]
    fn checkpoints_list_passes_the_cli_reply_through() {
        let project = TestProject::new("checkpoints-list");
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_reply(0, checkpoint_list_reply()));
        let state = test_state(&project, lifecycle.clone());
        let data = ok_data(&get(&state, "/api/checkpoints"));
        assert_eq!(data, checkpoint_list_reply());
        assert_eq!(
            lifecycle.call_args(),
            vec![vec!["checkpoint".to_owned(), "list".to_owned()]]
        );
        let calls = lifecycle.calls();
        assert_eq!(calls[0].1, project.root());
    }

    #[test]
    fn checkpoint_list_failure_is_checkpoint_failed_with_detail() {
        let project = TestProject::new("checkpoints-list-fail");
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_failure(
            2,
            "xdesign: invalid entry in checkpoints/library.json",
        ));
        let state = test_state(&project, lifecycle);
        let response = get(&state, "/api/checkpoints");
        let (status, code) = error_code(&response);
        assert_eq!((status, code.as_str()), (500, "checkpoint_failed"));
        let value: Value = serde_json::from_slice(&response.body).unwrap();
        assert_eq!(value["error"]["detail"]["exit_code"], 2);
        assert!(value["error"]["detail"]["stderr"]
            .as_str()
            .unwrap()
            .contains("invalid entry"));
    }

    #[test]
    fn checkpoints_show_passes_through_and_maps_unknown_ids() {
        let project = TestProject::new("checkpoints-show");
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_reply(
            0,
            checkpoint_show_reply("energy-budget-positive"),
        ));
        lifecycle.push(lifecycle_reply(
            2,
            json!({"schema_version": 1, "ok": false,
                   "error": {"code": "unknown_checkpoint",
                             "message": "unknown checkpoint: missing-id"}}),
        ));
        lifecycle.push(lifecycle_failure(
            2,
            "xdesign: unknown checkpoint: missing-id",
        ));
        lifecycle.push(lifecycle_failure(
            2,
            "xdesign: invalid checkpoints/library.json",
        ));
        let state = test_state(&project, lifecycle.clone());

        let data = ok_data(&get(&state, "/api/checkpoints/energy-budget-positive"));
        assert_eq!(
            data["checkpoint"]["checkpoint_id"],
            "energy-budget-positive"
        );
        assert_eq!(data["checkpoint"]["script_exists"], true);

        let (status, code) = error_code(&get(&state, "/api/checkpoints/missing-id"));
        assert_eq!((status, code.as_str()), (404, "unknown_checkpoint"));

        // A plain-text unknown-id failure still maps to 404.
        let (status, code) = error_code(&get(&state, "/api/checkpoints/missing-id"));
        assert_eq!((status, code.as_str()), (404, "unknown_checkpoint"));

        // Any other failure (here a malformed library) stays a server error.
        let response = get(&state, "/api/checkpoints/missing-id");
        let (status, code) = error_code(&response);
        assert_eq!((status, code.as_str()), (500, "checkpoint_failed"));
        let value: Value = serde_json::from_slice(&response.body).unwrap();
        assert!(value["error"]["detail"]["stderr"]
            .as_str()
            .unwrap()
            .contains("invalid checkpoints/library.json"));

        assert_eq!(
            lifecycle.call_args(),
            vec![
                vec![
                    "checkpoint".to_owned(),
                    "show".to_owned(),
                    "energy-budget-positive".to_owned()
                ],
                vec![
                    "checkpoint".to_owned(),
                    "show".to_owned(),
                    "missing-id".to_owned()
                ],
                vec![
                    "checkpoint".to_owned(),
                    "show".to_owned(),
                    "missing-id".to_owned()
                ],
                vec![
                    "checkpoint".to_owned(),
                    "show".to_owned(),
                    "missing-id".to_owned()
                ],
            ]
        );
    }

    #[test]
    fn checkpoint_routes_validate_ids_and_methods() {
        let project = TestProject::new("checkpoints-routes");
        let lifecycle = FakeLifecycle::default();
        let state = test_state(&project, lifecycle.clone());
        let (status, code) = error_code(&get(&state, "/api/checkpoints/UPPER-id"));
        assert_eq!((status, code.as_str()), (404, "not_found"));
        let (status, code) = error_code(&post(
            &state,
            "/api/checkpoints/energy-budget-positive",
            json!({}),
        ));
        assert_eq!((status, code.as_str()), (405, "method_not_allowed"));
        assert!(lifecycle.calls().is_empty());
    }

    #[test]
    fn checkpoint_save_writes_the_request_file_and_runs_the_cli() {
        let project = TestProject::new("checkpoint-save");
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_reply(
            0,
            checkpoint_save_reply("energy-budget-positive"),
        ));
        let state = test_state(&project, lifecycle.clone());
        let body = valid_checkpoint_save_body();
        let data = ok_data(&post(&state, "/api/checkpoints", body.clone()));
        assert_eq!(data["created"], true);
        assert_eq!(data["version"], 2);
        assert_eq!(
            data["script_uri"],
            "checkpoints/scripts/energy_budget_positive.py"
        );

        let calls = lifecycle.calls();
        assert_eq!(calls.len(), 1);
        let args = &calls[0].0;
        assert_eq!(args[0], "checkpoint");
        assert_eq!(args[1], "save");
        assert_eq!(args[2], "--request");
        let request_uri = &args[3];
        assert!(
            request_uri.starts_with("cache/web/checkpoint-requests/"),
            "{request_uri}"
        );
        assert!(request_uri.ends_with(".json"), "{request_uri}");
        let written: Value = serde_json::from_str(&project.read_text(request_uri)).unwrap();
        assert_eq!(written, body);
        assert_eq!(calls[0].1, project.root());
    }

    #[test]
    fn checkpoint_save_maps_invalid_builtin_and_internal_failures() {
        let project = TestProject::new("checkpoint-save-errors");
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_reply(
            2,
            json!({"schema_version": 1, "ok": false,
                   "error": {"code": "invalid_checkpoint",
                             "message": "syntactic checkpoint requires at least one rule"}}),
        ));
        lifecycle.push(lifecycle_reply(
            2,
            json!({"schema_version": 1, "ok": false,
                   "error": {"code": "builtin_checkpoint",
                             "message": "built-in checkpoints cannot be saved"}}),
        ));
        lifecycle.push(lifecycle_failure(
            2,
            "xdesign: cannot overwrite the built-in checkpoint builtin-simple-requirements",
        ));
        lifecycle.push(lifecycle_failure(1, "xdesign: internal failure"));
        let state = test_state(&project, lifecycle.clone());
        let body = valid_checkpoint_save_body();

        let response = post(&state, "/api/checkpoints", body.clone());
        let (status, code) = error_code(&response);
        assert_eq!((status, code.as_str()), (400, "invalid_checkpoint"));
        let value: Value = serde_json::from_slice(&response.body).unwrap();
        assert!(value["error"]["message"]
            .as_str()
            .unwrap()
            .contains("at least one rule"));

        let (status, code) = error_code(&post(&state, "/api/checkpoints", body.clone()));
        assert_eq!((status, code.as_str()), (409, "builtin_checkpoint"));

        // A plain-text built-in mutation is still classified as a conflict.
        let (status, code) = error_code(&post(&state, "/api/checkpoints", body.clone()));
        assert_eq!((status, code.as_str()), (409, "builtin_checkpoint"));

        let response = post(&state, "/api/checkpoints", body);
        let (status, code) = error_code(&response);
        assert_eq!((status, code.as_str()), (500, "checkpoint_failed"));
        let value: Value = serde_json::from_slice(&response.body).unwrap();
        assert_eq!(value["error"]["detail"]["exit_code"], 1);
        assert!(value["error"]["detail"]["stderr"]
            .as_str()
            .unwrap()
            .contains("internal failure"));
    }

    #[test]
    fn checkpoint_save_rejects_malformed_requests_before_running_the_cli() {
        let project = TestProject::new("checkpoint-save-bad");
        let lifecycle = FakeLifecycle::default();
        let state = test_state(&project, lifecycle.clone());

        let (status, code) = error_code(&post_raw(&state, "/api/checkpoints", b"{not json"));
        assert_eq!((status, code.as_str()), (400, "invalid_json"));

        let (status, code) = error_code(&post(&state, "/api/checkpoints", json!([])));
        assert_eq!((status, code.as_str()), (400, "invalid_checkpoint"));

        assert!(lifecycle.calls().is_empty());
        assert!(!project
            .root()
            .join("cache/web/checkpoint-requests")
            .exists());
    }

    // ------------------------------------------------------------------
    // Library bindings in create/detail (round 2)
    // ------------------------------------------------------------------

    fn library_create_body() -> Value {
        json!({
            "wish_id": "wish-library",
            "title": "Library bound wish",
            "description": "Uses a script library checkpoint and a semantic library checkpoint.",
            "flow_id": "simple-flow",
            "checks": [
                {"checkpoint_id": "simpleRequirementsAccepted", "mode": "library",
                 "library_id": "energy-budget-positive"},
                {"checkpoint_id": "simpleDesignAccepted", "mode": "default"},
                {"checkpoint_id": "simpleImplementationAccepted", "mode": "default"},
                {"checkpoint_id": "simpleRecordAccepted", "mode": "default"},
                {"checkpoint_id": "verificationAccepted", "mode": "library",
                 "library_id": "verification-evidence-complete"},
            ],
        })
    }

    #[test]
    fn create_wish_accepts_library_bindings_and_writes_them_to_the_checks_file() {
        let project = TestProject::new("create-library");
        let lifecycle = FakeLifecycle::default();
        // Best-effort pre-validation reads the library once.
        lifecycle.push(lifecycle_reply(0, checkpoint_list_reply()));
        lifecycle.push(lifecycle_reply(
            0,
            json!({"schema_version": 1, "wish_id": "wish-library",
                   "flow_id": "simple-flow", "store_revision": 1}),
        ));
        let state = test_state(&project, lifecycle.clone());
        let data = ok_data(&post(&state, "/api/wishes", library_create_body()));
        assert_eq!(data["wish_id"], "wish-library");

        let config: Value =
            serde_json::from_str(&project.read_text("cache/web/checks/wish-library.json")).unwrap();
        assert_eq!(config["schema_version"], 1);
        let checks = config["checks"].as_array().unwrap();
        assert_eq!(checks.len(), 5);
        let requirements = checks
            .iter()
            .find(|entry| entry["checkpoint_id"] == "simpleRequirementsAccepted")
            .unwrap();
        assert_eq!(requirements["mode"], "library");
        assert_eq!(requirements["library_id"], "energy-budget-positive");
        let verification = checks
            .iter()
            .find(|entry| entry["checkpoint_id"] == "verificationAccepted")
            .unwrap();
        assert_eq!(verification["mode"], "library");
        assert_eq!(verification["library_id"], "verification-evidence-complete");
        assert!(checks.iter().all(|entry| entry["mode"] != "cli"));

        let calls = lifecycle.call_args();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0], vec!["checkpoint", "list"]);
        assert!(calls[1]
            .windows(2)
            .any(|pair| pair == ["--checks", "cache/web/checks/wish-library.json"]));
    }

    fn create_body_with_checks(checks: Value) -> Value {
        json!({
            "wish_id": "wish-kind",
            "title": "Kind checked wish",
            "description": "Library binding pre-validation",
            "flow_id": "simple-flow",
            "checks": checks,
        })
    }

    #[test]
    fn create_wish_prevalidates_library_kind_and_existence() {
        let project = TestProject::new("create-library-prevalidate");
        let lifecycle = FakeLifecycle::default();
        for _ in 0..4 {
            lifecycle.push(lifecycle_reply(0, checkpoint_list_reply()));
        }
        lifecycle.push(lifecycle_reply(
            0,
            json!({"schema_version": 1, "wish_id": "wish-kind", "store_revision": 1}),
        ));
        let state = test_state(&project, lifecycle.clone());

        // Semantic entry bound to a syntactic gate.
        let response = post(
            &state,
            "/api/wishes",
            create_body_with_checks(json!([{
                "checkpoint_id": "simpleRequirementsAccepted",
                "mode": "library",
                "library_id": "verification-evidence-complete",
            }])),
        );
        let (status, code) = error_code(&response);
        assert_eq!((status, code.as_str()), (400, "invalid_checks"));
        let value: Value = serde_json::from_slice(&response.body).unwrap();
        let message = value["error"]["message"].as_str().unwrap();
        assert!(message.contains("is semantic"), "{message}");
        assert!(message.contains("requires syntactic"), "{message}");

        // Unknown library id.
        let response = post(
            &state,
            "/api/wishes",
            create_body_with_checks(json!([{
                "checkpoint_id": "simpleRequirementsAccepted",
                "mode": "library",
                "library_id": "ghost-gate",
            }])),
        );
        let (status, code) = error_code(&response);
        assert_eq!((status, code.as_str()), (400, "invalid_checks"));
        let value: Value = serde_json::from_slice(&response.body).unwrap();
        assert!(value["error"]["message"]
            .as_str()
            .unwrap()
            .contains("unknown library checkpoint: ghost-gate"));

        // Syntactic entry bound to the semantic gate.
        let response = post(
            &state,
            "/api/wishes",
            create_body_with_checks(json!([{
                "checkpoint_id": "verificationAccepted",
                "mode": "library",
                "library_id": "energy-budget-positive",
            }])),
        );
        let (status, code) = error_code(&response);
        assert_eq!((status, code.as_str()), (400, "invalid_checks"));
        let value: Value = serde_json::from_slice(&response.body).unwrap();
        let message = value["error"]["message"].as_str().unwrap();
        assert!(message.contains("is syntactic"), "{message}");
        assert!(message.contains("requires semantic"), "{message}");

        // Invalid bindings never wrote the wish source or ran init.
        assert!(!project.root().join("cache/web").exists());
        assert_eq!(lifecycle.call_args().len(), 3);

        // Kind-appropriate bindings still create the wish.
        let data = ok_data(&post(
            &state,
            "/api/wishes",
            create_body_with_checks(json!([
                {"checkpoint_id": "simpleRequirementsAccepted", "mode": "library",
                 "library_id": "energy-budget-positive"},
                {"checkpoint_id": "simpleDesignAccepted", "mode": "default"},
                {"checkpoint_id": "simpleImplementationAccepted", "mode": "default"},
                {"checkpoint_id": "simpleRecordAccepted", "mode": "default"},
                {"checkpoint_id": "verificationAccepted", "mode": "library",
                 "library_id": "verification-evidence-complete"},
            ])),
        ));
        assert_eq!(data["wish_id"], "wish-kind");
        let calls = lifecycle.call_args();
        assert_eq!(calls.len(), 5);
        assert_eq!(calls[3], vec!["checkpoint", "list"]);
        assert_eq!(calls[4][0], "init");
    }

    #[test]
    fn create_wish_maps_authoritative_init_binding_rejection_to_400() {
        let project = TestProject::new("create-library-fallback");
        let lifecycle = FakeLifecycle::default();
        // The library cannot be read during pre-validation: init decides.
        lifecycle.push(lifecycle_failure(
            2,
            "xdesign: cannot read checkpoints/library.json",
        ));
        lifecycle.push(lifecycle_failure(
            2,
            "xdesign: invalid check bindings: library checkpoint ghost is semantic but the gate requires syntactic",
        ));
        // A genuine engine/IO failure is still a server error.
        lifecycle.push(lifecycle_failure(
            2,
            "xdesign: cannot read checkpoints/library.json",
        ));
        lifecycle.push(lifecycle_failure(
            1,
            "xdesign: cannot write operational store: disk full",
        ));
        let state = test_state(&project, lifecycle.clone());
        let body = create_body_with_checks(json!([{
            "checkpoint_id": "simpleRequirementsAccepted",
            "mode": "library",
            "library_id": "ghost-gate",
        }]));

        let response = post(&state, "/api/wishes", body.clone());
        let (status, code) = error_code(&response);
        assert_eq!((status, code.as_str()), (400, "invalid_checks"));
        let value: Value = serde_json::from_slice(&response.body).unwrap();
        assert!(value["error"]["message"]
            .as_str()
            .unwrap()
            .contains("library checkpoint ghost"));

        let (status, code) = error_code(&post(&state, "/api/wishes", body));
        assert_eq!((status, code.as_str()), (500, "init_failed"));
    }

    #[test]
    fn create_wish_rejects_invalid_library_bindings() {
        let project = TestProject::new("create-library-bad");
        let lifecycle = FakeLifecycle::default();
        let state = test_state(&project, lifecycle.clone());
        let cases: Vec<Value> = vec![
            // Library mode on a gate that has no library binding.
            json!({"wish_id": "w1", "title": "T", "flow_id": "simple-flow",
                   "checks": [{"checkpoint_id": "someOtherGate", "mode": "library",
                               "library_id": "energy-budget-positive"}]}),
            // Missing library id.
            json!({"wish_id": "w1", "title": "T", "flow_id": "simple-flow",
                   "checks": [{"checkpoint_id": "simpleRequirementsAccepted",
                               "mode": "library"}]}),
            // Non-slug library id.
            json!({"wish_id": "w1", "title": "T", "flow_id": "simple-flow",
                   "checks": [{"checkpoint_id": "simpleRequirementsAccepted",
                               "mode": "library", "library_id": "Energy Budget"}]}),
            // Library bindings are not supported for wish-verify.
            json!({"wish_id": "w1", "title": "T", "flow_id": "wish-verify",
                   "checks": [{"checkpoint_id": "wishAccepted", "mode": "library",
                               "library_id": "verification-evidence-complete"}]}),
            // The semantic gate only accepts library mode, not default.
            json!({"wish_id": "w1", "title": "T", "flow_id": "simple-flow",
                   "checks": [{"checkpoint_id": "verificationAccepted", "mode": "default"}]}),
        ];
        for body in cases {
            let (status, code) = error_code(&post(&state, "/api/wishes", body.clone()));
            assert_eq!(status, 400, "case {body}: {code}");
            assert_eq!(code, "invalid_checks", "case {body}");
        }
        assert!(lifecycle.calls().is_empty());
        assert!(!project.root().join("cache/web/checks").exists());
    }

    fn find_checkpoint<'a>(data: &'a Value, checkpoint_id: &str) -> &'a Value {
        data["plan"]["checkpoints"]
            .as_array()
            .unwrap()
            .iter()
            .find(|checkpoint| checkpoint["checkpoint_id"] == checkpoint_id)
            .unwrap()
    }

    #[test]
    fn wish_detail_joins_the_current_library_and_exposes_library_bindings() {
        let project = TestProject::new("detail-library");
        let frozen = library_frozen_plan(
            "simple-flow",
            &[
                ("simpleRequirementsAccepted", "energy-budget-positive"),
                ("verificationAccepted", "verification-evidence-complete"),
            ],
        );
        seed_legacy_wish_with_plan(
            &project,
            "wish-library-detail",
            "simple-flow",
            "active",
            "Library detail",
            Some(frozen),
        );
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_reply(0, checkpoint_list_reply()));
        let state = test_state(&project, lifecycle.clone());
        let data = ok_data(&get(&state, "/api/wishes/wish-library-detail"));

        let requirements = find_checkpoint(&data, "simpleRequirementsAccepted");
        assert_eq!(requirements["library_id"], "energy-budget-positive");
        assert_eq!(
            requirements["checker"]["binding"],
            json!({"mode": "library", "library_id": "energy-budget-positive"})
        );
        assert_eq!(
            requirements["library"],
            json!({
                "checkpoint_id": "energy-budget-positive",
                "title": "Energy budget stays non-negative",
                "kind": "syntactic",
                "version": 2,
                "builtin": false,
            })
        );

        let verification = find_checkpoint(&data, "verificationAccepted");
        assert_eq!(verification["library_id"], "verification-evidence-complete");
        assert_eq!(verification["checker"], Value::Null);
        assert_eq!(
            verification["binding"],
            json!({"mode": "library", "library_id": "verification-evidence-complete"})
        );
        assert_eq!(
            verification["library"]["title"],
            "Verification evidence answers every requirement"
        );

        let design = find_checkpoint(&data, "simpleDesignAccepted");
        assert_eq!(design["library_id"], Value::Null);
        assert_eq!(design["library"], Value::Null);
        assert_eq!(design["checker"]["binding"], json!({"mode": "default"}));

        assert_eq!(data["warnings"], json!([]));
        assert_eq!(
            lifecycle.call_args(),
            vec![vec!["checkpoint".to_owned(), "list".to_owned()]]
        );
    }

    #[test]
    fn wish_detail_joins_library_for_store_backed_wishes() {
        let project = TestProject::new("detail-library-store");
        let frozen = library_frozen_plan(
            "simple-flow",
            &[
                ("simpleRequirementsAccepted", "energy-budget-positive"),
                ("verificationAccepted", "verification-evidence-complete"),
            ],
        );
        seed_store_wish_with_plan(
            &project,
            "wish-library-store",
            "simple-flow",
            "active",
            Some(("Store library", "Frozen library bindings in the store plan")),
            None,
            Some(frozen),
        );
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_reply(0, checkpoint_list_reply()));
        let state = test_state(&project, lifecycle.clone());
        let data = ok_data(&get(&state, "/api/wishes/wish-library-store"));
        assert_eq!(data["source"], "store");
        assert_eq!(data["head_revision"], 1);
        let requirements = find_checkpoint(&data, "simpleRequirementsAccepted");
        assert_eq!(requirements["library_id"], "energy-budget-positive");
        assert_eq!(
            requirements["checker"]["binding"],
            json!({"mode": "library", "library_id": "energy-budget-positive"})
        );
        assert_eq!(
            requirements["library"]["title"],
            "Energy budget stays non-negative"
        );
        let verification = find_checkpoint(&data, "verificationAccepted");
        assert_eq!(
            verification["binding"],
            json!({"mode": "library", "library_id": "verification-evidence-complete"})
        );
        assert_eq!(data["warnings"], json!([]));
        assert_eq!(
            lifecycle.call_args(),
            vec![vec!["checkpoint".to_owned(), "list".to_owned()]]
        );

        // The list projection of the same store-backed library wish must still
        // work (the store plan carries the frozen library member).
        let wishes = ok_data(&get(&state, "/api/wishes"));
        assert!(wishes["wishes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|wish| wish["wish_id"] == "wish-library-store"));
    }

    #[test]
    fn wish_detail_warns_when_a_frozen_library_id_no_longer_resolves() {
        let project = TestProject::new("detail-library-missing");
        let frozen = library_frozen_plan(
            "simple-flow",
            &[
                ("simpleRequirementsAccepted", "energy-budget-positive"),
                ("simpleDesignAccepted", "removed-checkpoint"),
            ],
        );
        seed_legacy_wish_with_plan(
            &project,
            "wish-library-missing",
            "simple-flow",
            "active",
            "Missing library",
            Some(frozen),
        );
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_reply(0, checkpoint_list_reply()));
        let state = test_state(&project, lifecycle);
        let data = ok_data(&get(&state, "/api/wishes/wish-library-missing"));

        let requirements = find_checkpoint(&data, "simpleRequirementsAccepted");
        assert_eq!(requirements["library_id"], "energy-budget-positive");
        assert!(requirements["library"]["title"].is_string());
        let design = find_checkpoint(&data, "simpleDesignAccepted");
        assert_eq!(design["library_id"], "removed-checkpoint");
        assert_eq!(design["library"], Value::Null);
        let warnings = data["warnings"].as_array().unwrap();
        assert!(
            warnings
                .iter()
                .any(|warning| warning.as_str().unwrap().contains("removed-checkpoint")),
            "{warnings:?}"
        );
    }

    #[test]
    fn wish_detail_never_fails_when_the_library_is_unreadable() {
        let project = TestProject::new("detail-library-unreadable");
        let frozen = library_frozen_plan(
            "simple-flow",
            &[("simpleRequirementsAccepted", "energy-budget-positive")],
        );
        seed_legacy_wish_with_plan(
            &project,
            "wish-library-broken",
            "simple-flow",
            "active",
            "Broken library",
            Some(frozen),
        );
        let lifecycle = FakeLifecycle::default();
        lifecycle.push(lifecycle_failure(
            2,
            "xdesign: invalid checkpoints/library.json",
        ));
        let state = test_state(&project, lifecycle);
        let data = ok_data(&get(&state, "/api/wishes/wish-library-broken"));
        let requirements = find_checkpoint(&data, "simpleRequirementsAccepted");
        assert_eq!(requirements["library_id"], "energy-budget-positive");
        assert_eq!(requirements["library"], Value::Null);
        assert_eq!(
            requirements["checker"]["binding"],
            json!({"mode": "library", "library_id": "energy-budget-positive"})
        );
        let warnings = data["warnings"].as_array().unwrap();
        assert!(
            warnings.iter().any(|warning| warning
                .as_str()
                .unwrap()
                .contains("Cannot read the checkpoint library")),
            "{warnings:?}"
        );
    }

    #[test]
    fn wish_detail_does_not_run_checkpoint_list_without_library_bindings() {
        let project = TestProject::new("detail-no-library");
        seed_legacy_wish(&project, "wish-plain", "simple-flow", "active", "Plain");
        let lifecycle = FakeLifecycle::default();
        let state = test_state(&project, lifecycle.clone());
        ok_data(&get(&state, "/api/wishes/wish-plain"));
        assert!(lifecycle.call_args().is_empty());
    }

    // ------------------------------------------------------------------
    // Static assets and the HTTP loop
    // ------------------------------------------------------------------

    static TEST_ASSETS: &[(&str, &[u8], &str)] = &[
        (
            "/index.html",
            b"<html>index</html>",
            "text/html; charset=utf-8",
        ),
        (
            "/app.js",
            b"console.log(1)",
            "text/javascript; charset=utf-8",
        ),
        ("/styles.css", b"body{}", "text/css; charset=utf-8"),
    ];

    #[test]
    fn asset_lookup_serves_files_and_falls_back_for_hash_routes() {
        assert!(matches!(
            lookup_asset(TEST_ASSETS, "/"),
            AssetLookup::Found(bytes, "text/html; charset=utf-8") if bytes == b"<html>index</html>"
        ));
        assert!(matches!(
            lookup_asset(TEST_ASSETS, "/app.js"),
            AssetLookup::Found(_, "text/javascript; charset=utf-8")
        ));
        assert!(matches!(
            lookup_asset(TEST_ASSETS, "/wishes/wish-1"),
            AssetLookup::SpaIndex(_, "text/html; charset=utf-8")
        ));
        assert!(matches!(
            lookup_asset(TEST_ASSETS, "/overview"),
            AssetLookup::SpaIndex(..)
        ));
        assert!(matches!(
            lookup_asset(TEST_ASSETS, "/missing.png"),
            AssetLookup::NotFound
        ));
        assert!(matches!(
            lookup_asset(TEST_ASSETS, "/js/unknown.js"),
            AssetLookup::NotFound
        ));
        assert!(matches!(
            lookup_asset(TEST_ASSETS, "/favicon.ico"),
            AssetLookup::NotFound
        ));
    }

    #[test]
    fn asset_route_serves_the_embedded_page_and_rejects_post() {
        let project = TestProject::new("assets");
        let state = test_state(&project, FakeLifecycle::default());
        let response = get(&state, "/");
        assert_eq!(response.status, 200);
        assert_eq!(response.content_type, "text/html; charset=utf-8");
        assert!(!response.body.is_empty());
        let (status, code) = error_code(&post(&state, "/", json!({})));
        assert_eq!((status, code.as_str()), (405, "method_not_allowed"));
        let response = get(&state, "/wishes/wish-1");
        assert_eq!(response.status, 200, "SPA fallback must serve the page");
    }

    fn raw_request(address: std::net::SocketAddr, request: &str) -> String {
        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }

    fn response_body(response: &str) -> &str {
        response.split_once("\r\n\r\n").unwrap().1
    }

    #[test]
    fn http_loop_serves_json_and_long_checks_do_not_block_accepts() {
        let project = TestProject::new("http-loop");
        seed_store_wish(
            &project,
            "wish-loop",
            "simple-flow",
            "active",
            None,
            Some("Loop"),
        );
        let lifecycle = FakeLifecycle::default();
        lifecycle.delay("next", Duration::from_millis(700));
        lifecycle.push(lifecycle_reply(
            0,
            next_reply("wish-loop", "simple-flow", "design", false),
        ));
        lifecycle.push(lifecycle_reply(
            0,
            json!({"schema_version": 1, "advance": "enter_next_stage"}),
        ));
        let state = Arc::new(test_state(&project, lifecycle.clone()));
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let shutdown = Arc::new(AtomicBool::new(false));
        let server = {
            let shutdown = Arc::clone(&shutdown);
            std::thread::spawn(move || {
                super::super::serve_listener(listener, state, Some(shutdown)).unwrap();
            })
        };

        let health = raw_request(
            address,
            "GET /api/health HTTP/1.1\r\nHost: localhost\r\n\r\n",
        );
        assert!(health.starts_with("HTTP/1.1 200 OK"), "{health}");
        let body: Value = serde_json::from_str(response_body(&health)).unwrap();
        assert_eq!(body["ok"], true);
        assert_eq!(body["data"]["store_present"], true);

        let asset = raw_request(address, "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
        assert!(asset.starts_with("HTTP/1.1 200 OK"), "{asset}");

        let unknown = raw_request(address, "GET /api/nope HTTP/1.1\r\nHost: localhost\r\n\r\n");
        assert!(unknown.starts_with("HTTP/1.1 404 Not Found"), "{unknown}");

        let bad_method = raw_request(
            address,
            "DELETE /api/wishes HTTP/1.1\r\nHost: localhost\r\n\r\n",
        );
        assert!(bad_method.starts_with("HTTP/1.1 405"), "{bad_method}");

        // A slow check (its `next` step sleeps 700 ms) must not block other
        // accepts: a health request completes while the check is in flight.
        let check = {
            let address = address;
            std::thread::spawn(move || {
                raw_request(
                    address,
                    "POST /api/wishes/wish-loop/check HTTP/1.1\r\n\
                     Host: localhost\r\n\
                     Content-Type: application/json\r\n\
                     Content-Length: 2\r\n\r\n{}",
                )
            })
        };
        std::thread::sleep(Duration::from_millis(150));
        let started = Instant::now();
        let health = raw_request(
            address,
            "GET /api/health HTTP/1.1\r\nHost: localhost\r\n\r\n",
        );
        let elapsed = started.elapsed();
        assert!(
            elapsed < Duration::from_millis(500),
            "health was blocked behind a check for {elapsed:?}"
        );
        assert!(health.starts_with("HTTP/1.1 200 OK"), "{health}");

        let check_response = check.join().unwrap();
        assert!(
            check_response.starts_with("HTTP/1.1 200 OK"),
            "{check_response}"
        );
        assert!(response_body(&check_response).contains("enter_next_stage"));

        shutdown.store(true, Ordering::Relaxed);
        // Wake the blocking accept so the loop observes the shutdown flag.
        let _ = TcpStream::connect(address);
        server.join().unwrap();
    }

    #[test]
    fn binding_derivation_covers_default_cli_and_command_runners() {
        let default_checker = json!({
            "checker_id": "simpleRequirementsCheck",
            "command_uri": "tools/checks/simple_requirements.py",
        });
        assert_eq!(derive_binding(&default_checker), json!({"mode": "default"}));

        let cli_checker = json!({
            "checker_id": "simpleDesignCheck",
            "command_uri": "tools/checks/simple_design.py",
            "custom": {"runner": "cli", "cli": "codex", "model": "gpt-6-astra"},
        });
        assert_eq!(
            derive_binding(&cli_checker),
            json!({"mode": "cli", "cli": "codex", "model": "gpt-6-astra"})
        );

        let command_checker = json!({
            "checker_id": "simpleImplementationCheck",
            "command_uri": "tools/checks/custom_implementation.py",
            "custom": {"runner": "process"},
        });
        assert_eq!(
            derive_binding(&command_checker),
            json!({
                "mode": "command",
                "command_uri": "tools/checks/custom_implementation.py",
            })
        );

        // A null custom member means the default deterministic checker.
        let null_custom = json!({
            "checker_id": "simpleRecordCheck",
            "command_uri": "tools/checks/simple_record.py",
            "custom": Value::Null,
        });
        assert_eq!(derive_binding(&null_custom), json!({"mode": "default"}));
    }

    #[test]
    fn utc_now_has_the_rfc3339_shape() {
        let now = utc_now();
        assert_eq!(now.len(), 20, "{now}");
        assert!(now.ends_with('Z'), "{now}");
        let year: i32 = now[..4].parse().unwrap();
        assert!((1970..=2100).contains(&year), "{now}");
        let month: u32 = now[5..7].parse().unwrap();
        assert!((1..=12).contains(&month), "{now}");
    }
}
