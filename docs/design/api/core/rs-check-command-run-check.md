# `RsCheckCommand::run_check`

> `xdesign::core::ports` · trait method · implemented xdesign slice · realizes the `CheckCommandExchange` invocation ([D10](../../flows/simple-flow/views.md#d10))

<a id="u22"></a>

## run_check

```rust
fn run_check(
    &self, checker: &RsCheckerSpec,
    plan_id: &str, plan_uri: &str, plan_sha256: &str,
    wish_model: &str, run_record: &str,
    stage: &str, artifact_revision: u32, snapshot_digest: &str,
) -> Result<RsGateReport, RsGateError>
```

Invoke the checker's fixed command with the independent wish model identity,
the runtime JSON record, and the stage being judged;
answer with the report that becomes the assessment's evidence and the verdict that
decides it.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `checker` | `&RsCheckerSpec` | from the validated plan | Identifies the command and its exact rule set |
| `plan_id` | `&str` | `designedSimpleFlow` | Modeled usage identity |
| `plan_uri` | `&str` | frozen plan JSON path | Immutable plan bytes to read |
| `plan_sha256` | `&str` | SHA-256 of exact plan bytes | Rejects a changed or mismatched plan |
| `wish_model` | `&str` | repository-relative SysML wish trace path | Independent trace identity |
| `run_record` | `&str` | repository-relative JSON run record path | Executable record input |
| `stage` | `&str` | a stage id of the plan | Which stage's artifacts the command judges |
| `artifact_revision` | `u32` | current visit revision | Revision the report must identify |
| `snapshot_digest` | `&str` | current visit's canonical artifact snapshot SHA-256 | Expected exact artifact bytes |

**Returns**

| Case | Type | Guarantee |
| --- | --- | --- |
| `Ok(report)` | `RsGateReport` | revision and snapshot digest echo checked inputs; rules cover every declared rule exactly once; `holds` equals their conjunction; `report_uri` names a retained artifact |
| `Err(e)` | `RsGateError` | The check boundary could not produce a trustworthy report; `RsProcessCheck` returns `CommandFailed` for process, input, and report failures and `CheckerTimeout` when the checker process tree is killed at its deadline |

**Errors**

| Variant | Raised when | Caller action |
| --- | --- | --- |
| `RsGateError::CommandFailed` | the provided `RsProcessCheck` could not produce a trustworthy report | Record `Unevaluated`; inspect the command's own diagnostics |
| `RsGateError::CheckerTimeout` | the bounded checker deadline (whole process tree; default 600 s, `XDESIGN_CHECKER_TIMEOUT_SECS`) expired and the process group was killed with output drained | Record `Unevaluated`; inspect the retained partial logs and timeout note |

Panics: none. A different `RsCheckCommand` implementation can return another
`RsGateError`; [`evaluate_checkpoint`](evaluate-checkpoint.md#u20)
propagates that variant unchanged.

**Logic** (implementor side)

```
1. read plan_uri; require sha256(exact bytes) == plan_sha256 and
   decoded plan ID == plan_id
2. read run_record; require its plan ID, URI, SHA-256, stage and current
   revision to agree with these inputs
3. recompute each current artifact's SHA-256 from its file bytes;
   require the canonical artifact snapshot digest == snapshot_digest
4. locate checker.command_uri and run its fixed rules with these inputs
   in an isolated process group under the checker deadline; at the
   deadline kill the group and drain its bounded output
5. parse its per-rule output; require the rule IDs to equal checker.rules
   exactly once each, `input_sha256` to match re-read files, and each
   requirements/design input other than the wish identity, run record, and
   frozen plan to belong to the visit artifact set; holds := every(rule.holds)
6. retain the raw report; return Ok(GateReport {
       holds, artifact_revision,
       artifact_snapshot_digest: snapshot_digest,
       report_uri, rules })
7. on any failure of steps 1–5: return Err(CommandFailed), or
   Err(CheckerTimeout) when the deadline expired
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Executes an external command; retains its JSON report and command stdout/stderr logs |
| Complexity | Bound by the command itself and by the process adapter's hard deadline, which covers the whole process tree including descendants on supported Unix hosts; the trait itself imposes no deadline ([D50](../../flows/simple-flow/implementation.md#d50), [stage CLI contract](../../stage-cli.md#d85)) |
| Thread safety | `&self`; implementations must be `Send` and internally synchronized |

**See also:** [`RsCheckCommand`](rs-check-command.md#u21), [`RsGateReport`](rs-gate-report.md#u37), [`RsRuleReport`](rs-rule-report.md#u38).
