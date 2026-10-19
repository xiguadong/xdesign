# `RsCheckCommand::run_check`

> ports · trait method · designed-only · realizes the `CheckCommandExchange` invocation ([D10](../../details/simple_flow/03_structure_graphs.md#d10))

<a id="u22"></a>

## U22 run_check

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
| `Err(e)` | `RsGateError::CommandFailed` | The command could not produce a report (process error, timeout, unreadable input) |

**Errors**

| Variant | Raised when | Caller action |
| --- | --- | --- |
| `RsGateError::CommandFailed` | no trustworthy report exists | Record `Unevaluated`; inspect the command's own diagnostics |

Panics: none.

**Logic** (implementor side)

```
1. read plan_uri; require sha256(exact bytes) == plan_sha256 and
   decoded plan ID == plan_id
2. read run_record; require its plan ID, URI, SHA-256, stage and current
   revision to agree with these inputs
3. recompute each current artifact's SHA-256 from its file bytes;
   require the canonical artifact snapshot digest == snapshot_digest
4. locate checker.command_uri and run its fixed rules with these inputs
5. parse its per-rule output; require the rule IDs to equal checker.rules
   exactly once each; holds := every(rule.holds)
6. retain the raw report; return Ok(GateReport {
       holds, artifact_revision,
       artifact_snapshot_digest: snapshot_digest,
       report_uri, rules })
7. on any failure of steps 1–5: return Err(CommandFailed)
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Executes an external command; retains its report |
| Complexity | Bound by the command, not the caller; no timeout is designed ([D50](../../details/simple_flow/06_rust_uml_design.md#d50)) |
| Thread safety | `&self`; implementations must be `Send` and internally synchronized |

**See also:** [`RsCheckCommand`](06_check_command.md#u21), [`RsGateReport`](22_gate_report.md#u37), [`RsRuleReport`](23_rule_report.md#u38).
