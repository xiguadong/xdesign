# `RsSaveMaterial`

> `xdesign::storage` · struct · implemented; STORE-01–STORE-11 evidence executed within disclosed limits · realizes `XDesignStorage::SaveMaterial` in
> [D86](../../storage.md#d86); implementation projection `src/storage/mod.rs`.

<a id="u85"></a>

## RsSaveMaterial

```rust
use std::path::PathBuf;

pub struct RsSaveMaterial {
    pub uri: String,
    pub role: String,
    pub sha256: String,
    pub source: Option<PathBuf>,
}
```

One requested entry in a save's complete manifest. It declares the logical identity and exact
bytes to retain, plus an optional project-relative input location.

**Fields**

| Field | Contract |
| --- | --- |
| `uri` | Unique logical URI within the request; a normalized project-relative slash path under the strict URI rules. The containing manifest is validated for NFD-plus-case-fold name equivalence, including implied directories. |
| `role` | Material role token; it describes use without changing content identity. |
| `sha256` | Exactly 64 lowercase hexadecimal characters naming the required SHA-256 of the exact bytes. |
| `source` | `Some(path)` reads a regular file relative to the supplied project root. `None` carries a verified blob by digest: either a committed blob row or a digest verifiably published by this exact operation's earlier attempt. An unrelated published or orphan blob is never adopted. JSON omits `source` for `None`; `source: null` is invalid. |

A `PathBuf` that cannot be represented by the strict UTF-8 JSON source contract is invalid.
A source must remain beneath the project root, must not resolve into the store, and must have
no symlink components. Omitting `source` never waives blob existence or integrity checks.

**Errors**

Invalid URI/role/digest/source declarations and source digest or size mismatches are
`InvalidInput`; unsafe source resolution is `UnsafePath`. A digest that was never stored is
`NotFound`; a committed blob whose file is missing or altered is `Integrity`. The struct
itself has no fallible constructor; these errors arise when save validates or resolves it.

**Persistence**

Constructing or dropping this value has no effect. During save, `Some(source)` can cause
staging and no-replace publication of the declared bytes. `None` reuses only a verified
blob: a committed blob row or a digest this exact operation verifiably published on an
earlier attempt. The resulting committed manifest exposes [`RsMaterial`](rs-material.md#u86),
not this request-only source path.

**Logic**

For each entry, save validates the declarations, then reuses a blob already committed or
verifiably published by this operation — digest-verified through EOF, no source required —
and only otherwise reads the source with the 256 MiB cap and checks length/digest. A digest
that exists only as an unrelated orphan or another operation's publication is never adopted.
A digest with neither an attributable stored blob nor a readable matching source is
refused. After all required bytes are published, the complete manifest can commit atomically
with its revision. Every unchanged entry must still be restated.

**Example** (illustrative, not executed):

```rust
let material = RsSaveMaterial {
    uri: "cache/wishes/wish-012345.sysml".into(),
    role: "wish-source".into(),
    sha256: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into(),
    source: None,
};
assert_eq!(material.source, None); // carry a verified blob; never skip verification
```

**See also:** [`RsSaveRequest`](rs-save-request.md#u80),
[`RsMaterial`](rs-material.md#u86), and [save request schema I13](../../../interfaces/store-format.md#i13).
