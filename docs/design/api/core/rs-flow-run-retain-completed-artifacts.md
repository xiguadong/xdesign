# `RsFlowRun::retain_completed_artifacts`

> `xdesign::core::run` · method · Rust-side retention configuration; no additional SysML operation or gate authority.

<a id="u114"></a>

## retain_completed_artifacts

```rust
pub fn retain_completed_artifacts(&mut self, directory: String) -> Result<(), String>
```

Configure a normalized project-relative retention directory before any activity.
New persistent registrations use their run-owned `artifact-snapshots/` directory.
The setting is serialized in the run record and restored without changing history;
an absent setting preserves legacy workspace-bound checks.

On authorized advancement the engine copies each assessed artifact to
`DIRECTORY/SHA256` and records the original-to-retained URI map before completing
the visit. Existing bytes must match; links and conflicting files are refused.
The persistent adapter archives these copies in the same logical commit. Cache
copies alone are not durable operational authority.

Invalid directories or configuration after work return an error. Retention errors
prevent completion. This method does not reopen stages or judge later source edits;
the [revision contract](../../evidence-and-revisions.md#e03) defines that boundary.
