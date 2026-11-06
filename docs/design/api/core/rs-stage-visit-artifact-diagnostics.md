# `RsStageVisit::artifact_diagnostics`

> `xdesign::core::record` · method · Rust-side integrity diagnostics; no additional SysML operation.

<a id="u115"></a>

## artifact_diagnostics

```rust
pub fn artifact_diagnostics(&self) -> Vec<String>
```

Check the canonical original-URI/digest snapshot and hash each resolved artifact.
Completed snapshot-enabled visits use retained URIs; current and legacy visits
use live paths. Return an empty list on success. Failures identify the stage,
visit, original URI, retained URI, expected/actual digest or file-read error.

The method reads files and changes no state. `verify_artifacts()` is its boolean
projection. Stage CLI preflight uses these diagnostics before further checker
execution, and work responses expose them as correction reasons.

See [artifact verification](rs-stage-visit-verify-artifacts.md#u55) and the
[stage interface](../../stage-cli.md#d89).
