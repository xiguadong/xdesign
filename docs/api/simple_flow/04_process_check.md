# `RsProcessCheck`

> `xdesign::simple_flow::check` · struct and trait implementation · implemented SimpleFlow slice · Rust-side addition with no matching UML class; invokes the fixed Python commands

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

**Why this adapter sits in the flow layer** — its read-coverage policy encodes
this flow's fixed four-checker contract: the stage-specific binding rules below
(`flow-is-supported`'s retained-plan read, the design rules' `tools/checks/`
coverage, and the artifact-snapshot membership of requirements and design inputs)
name the designed flow's checkers and their declared reads. A different flow
provides its own `RsCheckCommand` adapter; `xdesign::core` owns only the trait
and the report types.

**Contract** — `run_check` first verifies the frozen plan file and the current JSON
run record identity, stage, revision, and artifact snapshot bytes. It then runs
`python3 <command_uri>` with the wish, record, checker, plan, stage, revision,
snapshot digest, and report path arguments. The raw report must identify the same
checker/stage/revision/digest, contain each declared rule once with a verdict,
nonempty input and observation data, and have an aggregate verdict equal to the
conjunction. The wish, frozen plan, and run record must have matching
`input_sha256` entries; all other entries are rehashed. Every passed rule input
must have a matching entry and cover its declared file or runtime read.
For `flow-is-supported`, the declared `retained plan serialization` read is
covered by the frozen `plan_uri` in that rule's hashed input list.
For design rules that read `tools/checks/`, the report must name all four
checker command files from the frozen plan.
Requirements and design source inputs, other than the wish, plan, and record,
must belong to the current visit's artifact snapshot. Any process or validation
failure yields `CommandFailed`.

**Logic** — validate inputs; choose a UUID-suffixed JSON report path; execute the
command; retain stdout/stderr; parse and validate its JSON and input hashes; map
`passed`/`failed` rules to `RsRuleReport` and return the report. The gate evaluator
also checks the resulting report and its file before accepting an assessment.

**Limit** — it checks fixed structural conditions. A passing report does not
establish engineering adequacy or replace the semantic gate.

**See also:** [`RsCheckCommand`](../core/29_check_command.md#u21), [`run_check`](../core/30_run_check.md#u22), [D06](../../details/simple_flow/02_gates_and_rules.md#d06).
