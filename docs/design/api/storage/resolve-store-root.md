# `resolve_store_root`

> Layer: `xdesign::storage::cli` | Status: planned additive library/binary adapter bridge.
> Language-side rationale: the stage CLI binary cannot use library-private selection helpers.
> Allocated responsibility: `PlannedWorktreeStoreSelection` and `PlannedWorktreeStoreAccess`;
> planned source: `src/storage/{cli,location}.rs`, not an implemented API claim.

<a id="u116"></a>

## Root resolution

```rust
pub fn resolve_store_root(
    project_root: &std::path::Path,
    requested: Option<&std::path::Path>,
    enroll: bool,
) -> Result<std::path::PathBuf, RsStoreError>;
```

Canonical selected project, explicit option, environment and contextual precedence follow
[D118](../../storage.md#d118). A present empty request/environment is invalid; a relative
root uses `project_root`. Explicit selection never enrolls or migrates and bypasses Git/home
lookup. Existing binding selection performs full comparison; no local/global ambiguity is
resolved by preference. The returned absolute path is a selection, not permission to use it:
all access still goes through the common guard.

`enroll=false` is read-only and returns `NotFound` with `location_reason=binding_missing;`
when a valid linked project has no binding. CLI `location inspect` uses the richer private
inspection result to report that candidate state without inventing a root. `enroll=true`
may create only a wholly new eligible linked binding through the durable enrollment protocol;
it never creates store history, adopts a local candidate or repairs an existing nonce.
Only new/persistent lifecycle initialization and eligible persistent Web session setup use
true; persistent reads use false. The adapter first performs D119's strict legacy/persistent
classification under the complete discovery algorithm. A genuine legacy-only read or
continuation bypasses this resolver and takes the pure project/backend lease, even with
unrelated unavailable/unsafe home or absent global binding. Known candidate conflicts or
unreadable persistent provenance cannot be bypassed. A legacy-only Web session without an explicit override or known persistent candidate can report
`store_root=null`, `selection_state=legacy_only` and continue those runs without enrollment;
a later new persistent action must resolve successfully before it runs. Persistent Web setup's
binding allocation is explicit session setup, disclosed here; local-candidate conflict is
served only as a diagnostic state. Its persistent root is fixed before any persistent lifecycle child runs; a legacy child
retains an explicit root for conflict checks when supplied, otherwise reruns strict legacy
classification without inventing a fallback store root.

The [I22 diagnostic mapping](../../../interfaces/worktree-store.md#i22) applies through the
existing `RsStoreError` variants. Direct `RsStore::init/open` remain explicit-root calls and
do not invoke this resolver. Unit seams inject account/Git/identity observations without
adding production environment overrides; WTSTORE-01–04 verify selection and failure effects.
