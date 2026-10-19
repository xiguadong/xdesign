# `RsProcessCheck`

> ports · struct and trait implementation · implemented SimpleFlow slice · invokes the fixed Python commands

<a id="u58"></a>

## U58 RsProcessCheck

```rust
pub struct RsProcessCheck {
    pub report_directory: String,
}

impl RsCheckCommand for RsProcessCheck { /* run_check */ }
```

The production command adapter for the four designed syntactic gates. It writes
reports and command stdout/stderr logs to `report_directory`, invokes the command
URI declared by the checker spec, and converts the raw JSON rule report to
`RsGateReport`. It never supplies a semantic judgment. The caller selects
`report_directory`; the project CLI places it under ignored `cache/`.

**Contract** — `run_check` first verifies the frozen plan file and the current JSON
run record identity, stage, revision, and artifact snapshot bytes. It then runs
`python3 <command_uri>` with the wish, record, checker, plan, stage, revision,
snapshot digest, and report path arguments. The raw report must identify the same
checker/stage/revision/digest, contain each declared rule once with a verdict,
nonempty input and observation data, and have an aggregate verdict equal to the
conjunction. Its `input_sha256` entries are rehashed; requirements and design
inputs, other than the wish identity, must belong to the current visit's artifact
snapshot. Any process or validation failure yields `CommandFailed`.

**Logic** — validate inputs; choose a UUID-suffixed JSON report path; execute the
command; retain stdout/stderr; parse and validate its JSON and input hashes; map
`passed`/`failed` rules to `RsRuleReport` and return the report. The gate evaluator
also checks the resulting report and its file before accepting an assessment.

**Limit** — it checks fixed structural conditions. A passing report does not
establish engineering adequacy or replace the semantic gate.

**See also:** [`RsCheckCommand`](06_check_command.md#u21), [`run_check`](07_run_check.md#u22), [D06](../../details/simple_flow/02_gates_and_rules.md#d06).
