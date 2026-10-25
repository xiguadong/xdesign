# `RsDoctorReport`

> `xdesign::storage` · struct · implemented; STORE-01–STORE-10 evidence executed within disclosed limits · realizes
> `XDesignStorage::DoctorReport` in [D86](../../storage.md#d86); implementation
> projection `src/storage/mod.rs`.

<a id="u97"></a>

## RsDoctorReport

```rust
pub struct RsDoctorReport {
    pub verify_bytes: bool,
    pub findings: Vec<RsDoctorFinding>,
}
```

The result of one completed read-only inspection at the requested byte-verification level.

**Fields**

| Field | Contract |
| --- | --- |
| `verify_bytes` | Echoes whether full published-blob hashing through EOF was included. `false` still checks presence and recorded lengths; `true` additionally checks SHA-256. |
| `findings` | All [`RsDoctorFinding`](rs-doctor-finding.md#u96) values discovered by the completed inspection. Empty exactly when that inspection found no defect. |

`Ok(report)` means the requested inspection completed; it does not imply `findings.is_empty()`.
A nonempty report is not clean. If inspection itself fails, the method returns `Err` and no
report rather than claiming comprehensive partial findings.

**Errors**

The report has no independent error channel. Doctor uses `RsStoreError::Io` or `Database` for
local inspection failure and `UnsafePath` when a live path cannot be inspected safely.
Individual format, identity, record, pending, orphan, staging, permission, and byte defects
are report findings when the scan can complete.

**Persistence**

Constructing or dropping the report has no effect. Its producing inspection never updates,
deletes, repairs, migrates, or cleans store or staging state. The report is a snapshot of
observed findings, not a continuing guarantee against later corruption or replacement.

**Logic**

Doctor performs format/schema, identity, owner/path, record/receipt, manifest/reference,
pending/orphan/staging, and blob presence/length checks. When `verify_bytes` is true it also
hashes every published blob through EOF. Only after the selected inspection completes does it
return this report with every finding; the store CLI maps any finding to exit 5 and a clean
completed report to exit 0.

**Example** (illustrative, not executed):

```rust
let inspection = store.doctor(true)?;
assert!(inspection.verify_bytes);
if !inspection.findings.is_empty() {
    // Surface every finding; do not treat the stored bytes as a clean authority.
}
```

**See also:** [`RsStore::doctor`](rs-store-doctor.md#u105),
[`RsDoctorFindingKind`](rs-doctor-finding-kind.md#u95), and
[doctor acceptance STORE-07](../../../records/plans/storage.md#p38).
