# `RsStore::doctor`

> `xdesign::storage` · method · implemented; STORE-01–STORE-11 evidence executed within disclosed limits · realizes
> `XDesignStorage::DoctorOperation` in [D86](../../storage.md#d86); implementation
> projection `src/storage/{ops,blobs}.rs`.

<a id="u105"></a>

## doctor

```rust
impl RsStore {
    pub fn doctor(
        &self,
        verify_bytes: bool,
    ) -> Result<RsDoctorReport, RsStoreError>;
}
```

Complete a read-only integrity inspection and report every finding without repair.

**Parameters and inspection level**

- `verify_bytes = false` checks format and schema, live root/database identity,
  owner/permissions and path types, item/head/revision continuity, operation/receipt rules,
  complete manifests and blob references, pending operations, orphan final blobs, staging
  residue, dead-owner stage-writer locks, and blob presence/recorded length.
- `verify_bytes = true` additionally streams every published blob through EOF and verifies
  recorded length and SHA-256. A length mismatch can be `MissingBlob` for absent/truncated
  content or `AlteredBlob` for changed content; the finding detail states the observed
  condition. An unreferenced file is `OrphanBlob` even when its name matches its bytes.

**Returns**

`Ok(RsDoctorReport)` means the requested inspection completed and includes all findings.
`findings` is empty exactly when no defect was found; a nonempty report is not a clean result.
`Err` means inspection could not complete and cannot claim comprehensive findings. Individual
defects remain findings rather than aborting a scan that can safely continue.

**Persistence**

Doctor never updates, deletes, repairs, adopts, migrates, or cleans anything. It does not
finalize pending operations, remove staging/orphans, rewrite records, or make altered bytes
trusted. It reports a stage-writer lock whose recorded process is no longer running as a
`StaleLock` finding without removing it, so clearing that crash residue remains an operator
action. The report is observation output only.

**Errors**

`Io`/`Database` reports local inspection failure; `UnsafePath` reports that a live path cannot
be inspected safely. Format, identity, record, permission, pending, orphan, staging, lock, and
byte defects use [`RsDoctorFindingKind`](rs-doctor-finding-kind.md#u95) when the inspection can
complete. Any finding maps to store CLI exit 5 in
[I14](../../../interfaces/store-format.md#i14).

The store CLI uses the same inspection core through a private degraded-open path so a foreign
format can be reported instead of adopted. The public method starts from a successfully opened
store and detects subsequent replacement/corruption. Direct `outcome` on a pending operation
remains a separate successful query with explicit pending data.

**Logic**

1. Revalidate safe access and inspect format/schema, live and recorded identity, ownership,
   permissions, and path types.
2. Inspect item/head/revision continuity, operations/receipts, complete manifests, blob
   references, pending operations, orphan final blobs, staging residue, and dead-owner
   stage-writer locks.
3. Check blob presence and recorded lengths; when requested, stream and hash every published
   blob through EOF.
4. Accumulate every safely discoverable finding, return the completed report, and perform no
   cleanup or repair.

**Example** (illustrative, not executed):

```rust
let inspection = store.doctor(true)?;
assert!(inspection.verify_bytes);
if !inspection.findings.is_empty() {
    // Surface every finding; stored bytes are not a clean authority.
}
```

**See also:** [`RsDoctorReport`](rs-doctor-report.md#u97),
[`RsDoctorFinding`](rs-doctor-finding.md#u96), and
[physical/doctor contracts I12–I14](../../../interfaces/store-format.md#i12).
