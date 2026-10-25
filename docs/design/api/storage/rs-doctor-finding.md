# `RsDoctorFinding`

> `xdesign::storage` · struct · implemented; STORE-01–STORE-11 evidence executed within disclosed limits · realizes
> `XDesignStorage::DoctorFinding` in [D86](../../storage.md#d86); implementation
> projection `src/storage/mod.rs`.

<a id="u96"></a>

## RsDoctorFinding

```rust
pub struct RsDoctorFinding {
    pub kind: RsDoctorFindingKind,
    pub subject_uri: String,
    pub detail: String,
}
```

One integrity or recoverable-state observation produced by a completed doctor inspection.

**Fields**

| Field | Contract |
| --- | --- |
| `kind` | Stable [`RsDoctorFindingKind`](rs-doctor-finding-kind.md#u95) category for programmatic handling. |
| `subject_uri` | Identifies the store-relative blob/staging path, operation ID, item/revision, or record involved. It is a diagnostic subject, not proof that the subject is a valid readable URI. |
| `detail` | Human-oriented explanation of the observed condition; not a stable machine-readable code. In particular it distinguishes absent, truncated, and altered byte conditions. |

**Errors and return boundary**

The finding type has no independent fallible operation. An individual defect belongs in a
finding rather than aborting inspection. [`RsStore::doctor`](rs-store-doctor.md#u105) returns
`Err` only when the requested inspection cannot complete safely; such an error cannot be
represented as a supposedly comprehensive set of findings.

**Persistence**

Constructing, cloning, or dropping a finding has no effect. Findings are read-only report
output. They do not schedule or perform deletion, cleanup, repair, adoption, migration, or
trust of affected bytes.

**Logic**

Doctor classifies each discovered condition, records its exact subject and diagnostic detail,
and retains it with the other findings. A nonempty finding list is not a clean result; any
finding maps to store CLI exit 5. A pending operation is reported here without changing the
separate successful `outcome` query's explicit pending receipt.

**Example** (illustrative, not executed):

```rust
for finding in report.findings {
    let RsDoctorFinding { kind: _, subject_uri, detail } = finding;
    eprintln!("{subject_uri}: {detail}");
}
```

Persisted CLI envelopes use the format contract, not Rust debug text.

**See also:** [`RsDoctorFindingKind`](rs-doctor-finding-kind.md#u95),
[`RsDoctorReport`](rs-doctor-report.md#u97), and
[doctor CLI rules I14](../../../interfaces/store-format.md#i14).
