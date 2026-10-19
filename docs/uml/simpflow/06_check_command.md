# `RsCheckCommand`

> ports · trait · designed-only · realizes `CheckerCommand` ([D10](../../details/simple_flow/03_structure_graphs.md#d10))

<a id="u21"></a>

## U21 RsCheckCommand

```rust
pub trait RsCheckCommand: Send {
    fn run_check(
        &self, checker: &RsCheckerSpec,
        plan_id: &str, plan_uri: &str, plan_sha256: &str,
        wish_model: &str, run_record: &str,
        stage: &str, artifact_revision: u32, snapshot_digest: &str,
    ) -> Result<RsGateReport, RsGateError>;
}
```

The boundary every fixed (syntactic) check is invoked through. An implementation
supplies one dispatcher for the plan's checker specs; the run knows only the trait, so a checker
can be a subprocess, an in-process rule engine, or a test double without the run
changing.

**Required methods**

| Method | Page |
| --- | --- |
| [`run_check`](07_run_check.md#u22) | full entry |

**Implementor contract** — an implementation must guarantee, and a review checks:

1. **Reproducibility:** the same checker, frozen plan bytes, wish trace,
   run record, stage, revision, and artifact snapshot bytes
   yields the same report — the command runs its fixed rules unchanged.
2. **Rule coverage:** `RsGateReport.rules` answers every rule the gate's
   `RsCheckerSpec` declares, one `RsRuleReport` per rule, none missing, none extra.
   The command verifies the frozen plan file's SHA-256 and recomputes the
   artifact snapshot digest from exact file bytes before reporting a verdict.
3. **Read-only:** the command never mutates the wish model or the run's records.
4. **Honest failure:** when no report can be produced, return `Err`, never a
   fabricated `holds: true`.

**See also:** [`evaluate_checkpoint`](05_evaluate_checkpoint.md#u20), [`RsGateReport`](22_gate_report.md#u37), [interface D10](../../details/simple_flow/03_structure_graphs.md#d10), [ICD I01](../../icd/01_interfaces.md#i01).
