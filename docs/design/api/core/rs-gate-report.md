# `RsGateReport`

> `xdesign::core::ports` · struct · implemented xdesign slice · realizes the output side of `CheckCommandExchange` ([D10](../../flows/simple-flow/views.md#d10))

<a id="u37"></a>

## RsGateReport

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsGateReport {
    pub holds: bool,
    pub artifact_revision: u32,
    pub artifact_snapshot_digest: String,
    pub report_uri: String,
    pub rules: Vec<RsRuleReport>,
}
```

RsGateReport is a check command's answer: the overall verdict, the retained report, and the rule verdicts that become assessment evidence. Use it when recording a syntactic assessment or auditing what a command established.

**Fields**

| # | Field | Type | Visibility | Constraint | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `holds` | `bool` | `pub` | equals the conjunction of `rules` verdicts | Overall gate verdict |
| 2 | `artifact_revision` | `u32` | `pub` | equals the evaluated visit revision | Freshness anchor |
| 3 | `artifact_snapshot_digest` | `String` | `pub` | SHA-256 of the current visit's canonical artifact set; plan, run record, wish identity, and report exchange files are validated separately | Byte identity of the visited artifacts |
| 4 | `report_uri` | `String` | `pub` | repository-relative path to an existing retained report | Evidence anchor |
| 5 | `rules` | `Vec<RsRuleReport>` | `pub` | covers every declared rule exactly once, none extra | Per-rule verdicts |

**Invariants** — checked by the caller ([`evaluate_checkpoint`](evaluate-checkpoint.md#u20)
step 5): `artifact_revision` equals the visited revision,
`artifact_snapshot_digest` equals the visit's current snapshot digest after
the checker recomputes each file hash,
`holds == rules.iter().all(|r| r.holds)`, every failed rule has a nonempty
diagnostic, the rule ID set equals the gate's declared rule ID set with no
duplicates, and `report_uri` names an existing file. A report violating
any of these is treated as `CommandFailed`.

**See also:** [`RsRuleReport`](rs-rule-report.md#u38), [`RsCheckCommand::run_check`](rs-check-command-run-check.md#u22), [`RsCheckerSpec`](rs-checker-spec.md#u35).
