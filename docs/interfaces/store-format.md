# Operational store format and CLI

> Interface: local operational store | Schema: 1 | Status: implemented and exercised by
> storage acceptance evidence; not a frozen compatibility baseline.

This document owns the physical layout, JSON identities, command envelopes, and process
exit contract for the selected store. [Storage design D86](../design/storage.md#d86) owns
behavior and stage integration; the [Rust API set](../design/api/storage/index.md#u79) owns
typed calls. Direct SQLite access is not a public compatibility interface.

<a id="i11"></a>

## 1. Scope, roots, and versioning

The default store root is `.xdesign/` beneath the project root. `XDESIGN_STORE_ROOT`
overrides that default; an explicit `--root PATH` overrides the environment for one store
command. Root paths may be absolute or relative to the invocation's project root, but must
name a local POSIX directory satisfying the identity and permission checks below. The
public API receives the root explicitly and does not read the environment itself.

Version 1 has two related identities:

- request and record `schema_version = 1` govern JSON and logical table compatibility;
- root marker `format = "xdesign.store"` and `schema_version = 1` govern physical format.

`init` opens a valid existing store with its unchanged `store_id` and no record effect;
otherwise it creates a store only at an absent root or safe empty directory. Partial,
incompatible, or other nonempty content is refused rather than adopted. `open` creates
nothing. A missing store is `not_found`; a foreign, truncated, unsupported, or newer format
is refused without adoption, fallback, downgrade, or automatic migration. No command repairs
a format to make it openable.

<a id="i12"></a>

## 2. Physical format and durability units

### 2.1 Root layout

```text
.xdesign/
  format.json
  store.db
  store.db-wal                 # SQLite sidecar when present
  store.db-shm                 # SQLite sidecar when present
  blobs/sha256/ab/cdef...      # first 2 / remaining 62 digest characters
  staging/<operation-id>/      # transient, never authoritative bytes
  locks/items/<item-id>.lock   # transient stage-writer exclusion; dead-owner residue is a doctor finding
```

A closed allowlist of OS-generated root entries — exactly `.DS_Store` — may be present:
the store never reads or writes those entries, they are not doctor findings, and they do
not make a root nonempty for `init`. Every other entry is foreign content that refuses
`open`. A stage-writer lock whose recorded process is no longer running is crash residue:
doctor reports it as a `stale_lock` finding (best effort — a reused pid can mask residue),
and removing the named lock file is the supported operator recovery; doctor never removes
it itself.

`format.json` is strict JSON with exactly:

```json
{
  "format": "xdesign.store",
  "schema_version": 1,
  "store_id": "7fbbec45-8847-45b2-a5f1-e0f9316e2a45"
}
```

`store_id` is a random canonical UUID for example purposes; the contract requires a
nonempty opaque ID matching the general ID rule, not a particular UUID version. SQLite has
`application_id = 0x58445331` (`XDS1`), `user_version = 1`, `foreign_keys = ON`,
`journal_mode = WAL`, and `synchronous = FULL`. A finite SQLite busy wait is an
implementation choice, not a latency guarantee. `init` flushes the root marker, database,
and required directories before returning success.

### 2.2 Logical tables

The following names and constraints define schema 1. Implementations may add private
indexes, but may not change these public meanings or silently weaken the invariants.

| Table | Essential fields | Invariants |
| --- | --- | --- |
| `store_identity` | singleton `store_id`, `root_device`, `root_inode`, `database_device`, `database_inode` | Matches `format.json` and live `stat` identity; device/inode values are losslessly represented, not floating-point JSON numbers |
| `items` | `item_id` PK, `kind`, `head_revision` | `kind` immutable; head is 0 only before first commit and otherwise equals the greatest revision |
| `revisions` | `item_id`, `revision`, `metadata_json`, `operation_id`; PK `(item_id, revision)` | Contiguous from 1; immutable; metadata is a JSON object; producing operation is committed |
| `materials` | `item_id`, `revision`, `uri`, `role`, `sha256`, `byte_length`; PK `(item_id, revision, uri)` | Complete manifest for every revision; immutable; every row references a committed `blobs` row and published file |
| `blobs` | `sha256` PK, `byte_length` | Immutable content identity; file path derives only from digest; a row is inserted only after publication |
| `operations` | `operation_id` PK, exact `request_bytes`, `request_sha256`, `outcome`, `item_id`, `expected_revision`, `revision`, `result_json` | Exact request is immutable; pending may become committed or conflict; terminal receipts never change; revision is nonnull exactly for committed |
| `pending_blobs` | `operation_id`, `sha256`; PK pair | Transient reservation associated with an exact pending operation; not a committed blob-availability row |

`items.head_revision` and `operations.outcome/revision` are the only mutable authority
fields in this design. Historical revisions, manifest rows, blob rows, exact requests, and
terminal receipts reject update/delete through implementation guards and, where practical,
SQLite triggers. Pending reservations may be consumed by their own retry; transient staging
and locks are not engineering history. There is no public delete/GC operation.

### 2.3 Identity, permissions, and publication

At initialization/open and before each public operation and commit, the store checks:

1. root and database are at their recorded device/inode identities, are owned by the
   effective user, and are regular objects of the expected type;
2. store-controlled directories are `0700`, private mutable files are `0600`, and blobs are
   `0400`, with no group/other permission bits and no symlinks;
3. `format.json`, SQLite `application_id/user_version`, and `store_identity.store_id`
   agree, and the schema is exactly supported version 1;
4. all store-controlled path components remain beneath the opened root without symlink
   escapes. A filesystem unable to provide stable local identity is unsupported.

A source blob is first written to a same-filesystem staging file with owner-only access,
flushed with `fsync`, and published at `blobs/sha256/<first-2>/<remaining-62>` using an
atomic no-replace operation such as a POSIX hard link. Plain overwriting `rename` is not a
valid substitute. Parent directories are flushed after publication. Existing final files
are checked for regular-file type, length, and SHA-256 through end of file before reuse.
Staging is never returned as published material.

### 2.4 Crash-boundary meanings

| Boundary | Durable state | Required recovery |
| --- | --- | --- |
| Before pending reservation | No operation effect | Retry the same exact operation |
| After reservation, before/during source reads | Pending receipt and reservations; possibly staging | `outcome` reports pending; exact retry resumes; doctor reports pending/staging |
| After publication, before commit | Pending receipt; possibly orphan final blobs; no committed revision referencing them | Exact retry verifies/reuses or completes; conflict produces no revision; doctor reports pending/orphans |
| After commit, before confirmed response | Committed or conflict receipt and, for committed, complete revision/head | `outcome` returns the terminal receipt; never start a fresh operation |
| After response, with staging residue | Terminal operation remains authoritative; residue is untrusted transient data | Doctor reports staging; no silent cleanup is required by this slice |

No committed revision row may exist without all its blobs published. External deletion or
alteration after commit is a doctor/read integrity failure, not an event that rewrites the
manifest. Cleanup of a pending operation's own staging or published residue is ordered
after a confirmed terminal commit, so the recovery attribution of blobs published by that
operation is not lost while its outcome is unknown. A digest that exists only as an
unrelated published or orphan blob is never adopted as a source-less material.

<a id="i13"></a>

## 3. Strict JSON and save request schema

### 3.1 Common parser and identity rules

- Every accepted operation/request, registry, run, plan/work, report, and other protocol
  JSON document is at most **16 MiB** and nests to at most **64** levels, counting its root
  container as level 1. CLI envelopes obey the same limits; an oversized response fails
  explicitly and is never truncated.
- JSON must be UTF-8 with one top-level value and no trailing content. Duplicate object
  member names are rejected at every depth, including inside open `metadata` and `result`
  values. Unknown members are rejected in every protocol-defined object. Metadata keys and
  result contents are otherwise extensible unless constrained by `kind = "wish"` below.
- Numbers must be parsed losslessly; floating-point rounding is not accepted for revisions,
  lengths, or other integral fields. Strict JSON validation and storage use one pinned
  `serde_json` numeric parser/serializer, and open `metadata` and `result` numbers are
  stored as exact 64-bit integers or IEEE-754 doubles only when the stored representation
  is an exact fixed point: decoding that representation and re-encoding it yields the same
  value. Validation checks that fixed point against the actual representation the revision
  and receipt will persist before the operation is reserved. Three token classes are
  rejected: an integer token beyond the u64/i64 range (for example
  `18446744073709551617`); a decimal token whose canonical stored form differs from the
  supplied value (for example `0.30000000000000000004`); and a token whose extreme exponent
  range the exact representation does not support. Each is rejected with `invalid_input`;
  they are never silently rounded, and no token may panic or abort the caller. Unknown enum
  values, null in place of a required value, and `source: null` are invalid. Optional fields
  are omitted rather than nulled unless their response schema explicitly permits null.
- `item_id` and `operation_id` match `[A-Za-z0-9][A-Za-z0-9_-]{0,127}`. `kind` matches
  `[a-z][a-z0-9-]{0,63}` and is immutable for an item. `role` uses the same token rule.
  SHA-256 is exactly 64 lowercase hexadecimal characters.
- A logical URI or `source` is a normalized project-relative slash path of at most 4096
  UTF-8 bytes, with at most 255 bytes per segment. Reject empty paths/segments, absolute or
  scheme/authority paths, drive prefixes, backslashes, NUL/control characters, `.`/`..`,
  trailing slash, query/fragment syntax, and non-normal filesystem object types. Percent
  signs are literal; there is no percent-decoding or second path interpretation. URIs are
  unique by exact string within one request. A manifest must also be materializable on a
  filesystem: no URI may be both a file and an implied directory of another URI, and every
  URI entry and every implied directory entry is compared under host-independent Unicode
  canonical decomposition (NFD) and Unicode case folding — a conservative superset of simple
  case folding. Two distinct names whose canonical decomposed, case-folded forms
  collide are rejected, so `a` with `a/x`, `File.txt` with `file.txt`, or `A/x` with `a/y`
  are all rejected, as are canonically equivalent forms — for example, a precomposed
  `é` filename and the same name written as `e` plus a combining acute accent. The same
  equivalence check is repeated at checkout materialization before any destination file is
  written, so a stored manifest that cannot be materialized fails explicitly. ASCII case
  and file-versus-directory prefix rules are unchanged. Source containment is decided on
  resolved filesystem identity: the longest existing prefix of the joined source path is
  canonicalized, so case-insensitive aliases or symlinks resolving onto the store root are
  refused, and a store never ingests its own root under any spelling.
- A save must reserve room for its own receipt before reservation: the `result` and
  `metadata` nesting and serialized sizes, plus the manifest echo, must leave a fixed
  margin below the envelope limits (6 nesting levels and 2048 bytes in schema version 1).
  Manifest entries are budgeted at their JSON-escaped serialized length — quote-heavy URIs
  nearly double when the encoder escapes them —
  so a committed operation always has a receipt the CLI can return and the
  `response_too_large` envelope error is never the only report of committed work.
- Every material URI in a **new** reservation must be at most 900 UTF-8 bytes in total:
  the smallest supported `PATH_MAX` is 1024 and checkout destinations add ancestry of
  their own, so a longer URI could commit yet never materialize. The budget gates new
  reservations only; operations already reserved under schema 1 keep their original
  validation and remain retryable and readable.
- `expected_revision` and all revisions fit the nonnegative SQLite/JSON integer contract;
  the next revision must also fit. `expected_revision = 0` creates. One material is at most
  **256 MiB** (268,435,456 bytes); a 256 MiB-plus-one read proves excess rather than
  truncating. Total request size and count are bounded by the 16 MiB request cap.

For a typed `RsStore::save`, deterministic request bytes are UTF-8 compact JSON with
recursively sorted object members and no insignificant whitespace. Primitive values use the
pinned `serde_json` serializer's forms; this is not an assertion of RFC 8785 interoperability
across arbitrary JSON numbers. `save` is equivalent to `save_bytes` over those deterministic
bytes. `save_bytes` performs no canonicalization: its request SHA-256 is over the exact raw
bytes supplied. Exact bytes are also retained in `operations` so equality does not rely only
on a hash comparison.

### 3.2 `SaveRequest`

Required top-level members are exactly `schema_version`, `operation_id`, `item_id`, `kind`,
`expected_revision`, `metadata`, `materials`, and `result`. `metadata` is an object;
`result` is any strict JSON value. Each material has required `uri`, `role`, `sha256` and
optional `source`; no other member is allowed.

```json
{
  "schema_version": 1,
  "operation_id": "check-00000042",
  "item_id": "wish-012345",
  "kind": "wish",
  "expected_revision": 0,
  "metadata": {
    "flow_id": "simple-flow",
    "registry_uri": "cache/xdesign/wish-012345/registry.json",
    "run_uri": "cache/xdesign/wish-012345/run.json"
  },
  "materials": [
    {
      "uri": "cache/wishes/wish-012345.sysml",
      "role": "wish-source",
      "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      "source": "incoming/wish-012345.sysml"
    },
    {
      "uri": "cache/xdesign/wish-012345/run.json",
      "role": "run-record",
      "sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
      "source": "cache/xdesign/wish-012345/run.json"
    }
  ],
  "result": {
    "command": "init",
    "response": {"schema_version": 1, "status": "active"},
    "exit_code": 0
  }
}
```

The digest strings are shape examples, not evidence or hashes of real artifacts. On an
append, the request restates the complete desired manifest. An unchanged entry omits
`source` to carry its verified blob by digest:

```json
{
  "uri": "docs/design/storage.md",
  "role": "artifact",
  "sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
}
```

A source path is resolved beneath the supplied `project_root`, never beneath the store
root, and source bytes must equal the declared digest. Source omission is a carry, not a
waiver: it is accepted only when the digest has a committed blob row and a verified
published file, or when this exact operation's earlier attempt verifiably published that
digest and the attribution is still durable at recovery time. A digest that exists only
as an unrelated published or orphan blob is never adopted by omission. Omitting a URI from
the new complete manifest does not delete it from older revisions.

### 3.3 Wish metadata and projection input

For `kind = "wish"`, metadata requires exactly the semantics of these three members (other
metadata members may carry non-authoritative extension data):

| Member | Constraint |
| --- | --- |
| `flow_id` | Nonempty supported flow identifier; equals the selected revision registry's `flow_id` |
| `registry_uri` | Valid URI present in the same complete manifest; identifies a strict version-1 registry JSON material |
| `run_uri` | Valid URI present in the same complete manifest; equals registry `run_record_uri` and identifies the current run JSON at this revision |

The registry binds `wish_id = item_id`, `run_id`, plan/work URIs and digests, wish model URI,
and `run_record_uri`. The run binds that run ID and the registry's plan identity/digest.
These records use their existing strict schemas. Output `status` and `current_stage` are
computed from the run and are not accepted as independent wish metadata authorities.
Projection failure is an integrity error, not an empty/default status.

### 3.4 Receipt, revision, and query shapes

`OperationReceipt` JSON has exactly `operation_id`, `request_sha256`, `outcome`, `revision`,
and `result`. `outcome` is lowercase `pending`, `committed`, or `conflict`; `revision` is a
positive integer for committed and null otherwise. `result` is the exact JSON value from
the request. A conflict result is not evidence that its proposed response was applied.

`ItemRevision` JSON contains `item_id`, `kind`, `revision`, `metadata`, `materials`,
`operation_id`, and derived `projection`. Materials expose `uri`, `role`, `sha256`, and
`byte_length`; request-only `source` is not exposed. `projection` is null for kinds without
a defined projection; otherwise it contains `revision`, `flow_id`, `registry_uri`, `run_uri`,
`status`, and `current_stage`. Projection `revision` equals the containing revision, and
`current_stage` is null only for a completed run. `list` returns current item summaries;
`history` returns complete revisions in ascending order. CLI list/history data includes
`complete: true` only after all selected rows were read successfully.

<a id="i14"></a>

## 4. Public store command line

### 4.1 Commands

The store command group is separate from the existing stage `init/next/check` commands.
Run it from the project root unless `--project-root` is shown. Every command accepts
`--root PATH`; if absent, `XDESIGN_STORE_ROOT` and then `.xdesign` are used.

```text
xdesign store init [--root PATH]
xdesign store info [--root PATH]
xdesign store save --request FILE [--project-root PATH] [--root PATH]
xdesign store outcome OPERATION_ID [--root PATH]
xdesign store request OPERATION_ID --output FILE [--root PATH]
xdesign store list [--kind KIND] [--status received|active|blocked|delivered] [--root PATH]
xdesign store get ITEM_ID [--revision N] [--root PATH]
xdesign store history ITEM_ID [--root PATH]
xdesign store read ITEM_ID --uri URI [--revision N] --output FILE [--root PATH]
xdesign store checkout ITEM_ID --destination DIR [--revision N] [--root PATH]
xdesign store doctor [--verify-bytes] [--root PATH]
```

- `init` creates version 1 and returns store identity; `info` opens without mutation and
  returns identity plus item, revision, blob, pending-operation, and finding-independent
  counts. `info` does not claim integrity is clean.
- `save` reads exact request bytes from `FILE` (`-` means standard input) and validates
  them. It inspects the exact operation and returns an existing terminal receipt for an
  identical exact retry before it canonicalizes `--project-root` (default current project
  root) or resolves any source path; sources are needed only for bytes that no committed
  blob row or same-operation publication already supplies. The request file itself is not
  implicitly a material.
- `outcome`, `list`, `get`, and `history` return committed records/receipts in one read
  snapshot. `outcome` may explicitly return a pending receipt. `get/history` return full
  manifests; list returns heads only. No query silently truncates.
- `request` writes the exact retained request bytes of `OPERATION_ID` to a previously
  absent `FILE` under the same destination rules as `read` (all ancestors existing
  non-symlink directories, no overwrite, outside the store). The envelope reports the
  request digest, byte length, receipt state, and output path. It is the recovery input
  for `unknown_outcome` and pending operations when the original request file is gone:
  feeding the recovered file to `save --request FILE` is the exact retry.
- `read` verifies bytes before atomically creating the previously absent `FILE`; stdout is
  reserved for the JSON envelope, never mixed with binary content. The envelope reports
  URI, digest, length, revision, and output path. With `--revision N` the reported
  metadata and the written bytes are both pinned to revision N even if the head advances
  concurrently. Every ancestor of `FILE`, up to the filesystem root, must be an existing
  non-symlink directory — a symlink anywhere in the destination chain is refused, not
  resolved. An existing output or any failed write is an error, not a partial successful
  read.
- `checkout` verifies the entire selected manifest in a bounded streaming pass before
  exposing a previously absent destination directory containing every URI; materialization
  uses bounded buffers, so peak memory does not grow with manifest size or alias count. It
  returns revision, destination, and material count. Destination ancestry obeys the same
  all-components, up-to-root symlink rule as `read`. The stage adapter's cache restoration
  uses the same digest/no-overwrite rules for selected operational materials.
- `doctor` uses the read-only inspection core even when normal `open` would reject a root;
  it reports format/identity findings instead of adopting that root. `--verify-bytes` adds
  full blob hashing. Doctor never repairs. The public `RsStore::doctor` method uses the
  same core after a successful open and detects subsequent replacement/corruption.

Unknown options, duplicate options, missing required options, extra positional arguments,
and `--help` mixed with a command are input errors. `--help` alone is ordinary help text,
not a JSON command result.

### 4.2 JSON envelopes

A successful command emits exactly one envelope on stdout:

```json
{
  "schema_version": 1,
  "ok": true,
  "command": "outcome",
  "data": {
    "receipt": {
      "operation_id": "check-00000042",
      "request_sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      "outcome": "pending",
      "revision": null,
      "result": {"command": "check", "response": {}, "exit_code": 1}
    }
  }
}
```

An expected command failure emits exactly one envelope on stderr and no success envelope:

```json
{
  "schema_version": 1,
  "ok": false,
  "command": "save",
  "error": {
    "code": "operation_mismatch",
    "message": "The operation ID is reserved for different request bytes.",
    "operation_id": "check-00000042"
  },
  "data": null
}
```

When a save has reserved pending work before a later error, the error envelope includes its
`operation_id` and `"receipt_state": "pending"` when known. A durable revision conflict
uses `error.code = "revision_conflict"` and returns the stored receipt in `data` so the
proposed result cannot be mistaken for a committed response. `unknown_outcome` always
includes `operation_id`; the caller must use `outcome` or exact retry. Unexpected process
failures still need a diagnostic and nonzero status; they must not be translated into an
empty successful data envelope.

### 4.3 Exit codes

| Code | Meaning | Representative conditions |
| --- | --- | --- |
| 0 | Command succeeded and its explicit data is usable | Save committed, query returned (including an explicitly pending `outcome`), clean doctor, verified read/checkout |
| 2 | Invalid input or usage | Malformed/duplicate/unknown JSON, unsupported request schema, invalid ID/URI/digest, source mismatch or excess size, invalid option |
| 3 | Not found | Missing store, operation, item, revision, or manifest URI |
| 4 | Conflict | `operation_mismatch`, stale expected head/create or kind conflict, per-wish writer conflict |
| 5 | Corrupt, unsupported, unsafe, or integrity finding | Foreign root/database format, root/inode/permission/symlink defect, altered or missing blob, any doctor finding |
| 6 | Local I/O or SQLite failure whose commit outcome is known not to have succeeded, or which occurred before mutation | Read/write/open/busy failure outside the uncertain commit window; no false success |
| 7 | Unknown mutation outcome after reservation/publication or around commit/response | Reconcile the named operation through `outcome`; never substitute a fresh operation ID |

`save` returning a conflict receipt is exit 4, not 0. Doctor with any finding is exit 5 and
returns the complete report in `data`; clean doctor is the only exit-0 doctor result. Store
commands do not use exit 1. The existing stage CLI retains its separate exit 1 for a checked
visit that remains blocked.

<a id="i15"></a>

## 5. Compatibility and failure limits

Schema 1 preserves the existing frozen plan, work, registry, run, checker input, and report
JSON contracts as materials; it does not add fields to the strict run record. New store
commands do not change legacy stage storage or migrate existing Git traces. Future store
format or request versions require explicit design and version negotiation, never implicit
interpretation as version 1.

This is a local single-project operational format, not an authenticated or encrypted
exchange. Whole-store backup/snapshot/baseline, deletion/GC, model-claim audit, service
transport, nested/arbitrary flows, automatic migration, and Windows behavior are outside
this interface. Performance targets are unassigned. The executed [STORE acceptance
matrix](../records/plans/storage.md#p38) records actual recovery, concurrency, checkout/replay,
and two-flow behavior with their limits; it does not freeze compatibility or replace final
human acceptance.

New store-backed script registrations add the optional strict `checker.bundle` member to frozen plans, with executor materials retained from revision 1. The stage adapter checks run confinement and file identities; failed checker output is retained as unvalidated attempt materials. See [checker execution](../design/checker-execution.md#d111). New executables accept earlier plans without the member; older executables may refuse plans carrying it. This adds no store migration or restore operation.

The linked-worktree location extension is [I21](worktree-store.md#i21). Its public inspection, enrollment, request preparation and migration commands retain this schema-1 root/table/request format, add coordination outside the closed store root, and use a privileged verified construction path; ordinary open never adopts copied bytes. See its operator example and exact recovery procedure before migration.
