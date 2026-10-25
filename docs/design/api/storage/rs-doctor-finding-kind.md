# `RsDoctorFindingKind`

> `xdesign::storage` · enum · implemented; STORE-01–STORE-11 evidence executed within disclosed limits · realizes
> `XDesignStorage::DoctorFindingKind` in [D86](../../storage.md#d86); implementation
> projection `src/storage/mod.rs`.

<a id="u95"></a>

## RsDoctorFindingKind

```rust
pub enum RsDoctorFindingKind {
    OrphanBlob,
    PendingOperation,
    MissingBlob,
    AlteredBlob,
    StagingArtifact,
    StaleLock,
    InvalidRecord,
    FormatDefect,
    IdentityMismatch,
    PermissionDefect,
}
```

The machine-readable category of one completed read-only inspection finding. Findings are
diagnostics, not repairs or successful trusted reads. The corresponding logical variants are
lower camel case in SysML.

**Variants**

| Variant | Finding contract |
| --- | --- |
| `OrphanBlob` | A published final blob is unreferenced, even when its filename matches its bytes. |
| `PendingOperation` | An exact operation remains reserved without a terminal receipt; direct `outcome` still returns pending data successfully. |
| `MissingBlob` | Required content is absent or truncated; a length mismatch may select this category when the detail says truncated. |
| `AlteredBlob` | Published content differs from its recorded length/digest identity. |
| `StagingArtifact` | Transient staging residue remains; it is not authoritative published material. |
| `StaleLock` | A stage-writer lock under `locks/items/` records a process that is no longer running (or has no readable owner). A lock whose recorded process is alive is normal concurrency and is not reported; the liveness check is best effort, and removing the named lock file stays an operator action. |
| `InvalidRecord` | Item/head/revision continuity, operation/receipt, manifest, reference, or projection records are defective. |
| `FormatDefect` | Root marker, request/record format, application ID, or schema format is foreign, truncated, or unsupported. |
| `IdentityMismatch` | Live or recorded root/database/store identity values disagree. |
| `PermissionDefect` | Owner, permission bits, object type, or related safe-path expectations are defective. |

**Errors**

The enum itself has no parameters or fallible operation. A defect represented by a finding
does not abort a completed scan. `RsStoreError::Io`, `Database`, or `UnsafePath` instead means
the inspection could not complete safely and therefore cannot claim comprehensive findings.

**Persistence**

Constructing or matching a finding kind has no effect. Doctor never updates, deletes, repairs,
migrates, or cleans the condition it reports. The enum is stored only as report output, not as
a second defect authority or a cleanup queue.

**Logic**

[`RsStore::doctor`](rs-store-doctor.md#u105) checks record/format/identity/path conditions and
blob presence/length; with `verify_bytes = true`, it also hashes every published blob through
EOF. It collects all discovered findings. Any finding maps to store CLI exit 5, while an empty
completed report is clean.

**Example** (illustrative, not executed):

```rust
let byte_defect = matches!(
    finding.kind,
    RsDoctorFindingKind::MissingBlob | RsDoctorFindingKind::AlteredBlob
);
if byte_defect {
    // Do not trust reads or checkouts of the affected material.
}
```

**See also:** [`RsDoctorFinding`](rs-doctor-finding.md#u96),
[`RsDoctorReport`](rs-doctor-report.md#u97), and
[`RsStore::doctor`](rs-store-doctor.md#u105).
