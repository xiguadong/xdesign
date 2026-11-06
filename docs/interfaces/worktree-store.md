# Worktree store location and migration interface

> Interface version: 1 | Store format remains schema 1. The CLI implements the commands
> below; platform capability and engineering acceptance require their own retained evidence.

<a id="i22"></a>

## 1. Selection and inspection

[Storage D118](../design/storage.md#d118) owns the exact context and precedence algorithm;
[D119](../design/worktree-store-migration.md#d119) owns leases and recovery. This extension
does not change existing store request/receipt fields, the strict run schema or per-flow
judgment authority. Existing `--root` and `XDESIGN_STORE_ROOT` retain their explicit meaning.
Store commands are invoked from the selected project; existing `store save --project-root`
continues to change material-source resolution only. Migration/location commands' explicit
`--project-root` instead names the context being inspected/enrolled, resolving relative to cwd.

```text
xdesign store location inspect [--project-root DIR] [--root PATH]
xdesign store location enroll --operation-id ID [--project-root DIR]
        [--new-incarnation --previous-binding KEY]
xdesign store migrate prepare MIGRATION_ID --output FILE --older-clients-stopped [--project-root DIR]
xdesign store migrate apply --request FILE
xdesign store migrate status MIGRATION_ID --root SOURCE_OR_DESTINATION
xdesign store migrate request MIGRATION_ID --root SOURCE_OR_DESTINATION --output FILE
xdesign store migrate abort MIGRATION_ID --root SOURCE_OR_DESTINATION
xdesign store migrate discard-prepared MIGRATION_ID --root SOURCE_OR_DESTINATION
xdesign inspect WISH_ID --backend legacy|store [--revision N]
```

All commands are local and explicit. `FILE` is exact UTF-8 JSON (stdin `-` allowed for apply).
Unknown/duplicate options, empty values, invalid IDs, `--revision` for legacy, and an enrollment
acknowledgement without `--new-incarnation` are usage errors. Enroll never creates store
history; new-incarnation enrollment never binds old history to a recreated worktree. It
refuses a still-present unexpected nonce rather than erasing it. Inspection is the recovery
path when there is no supported reenrollment. IDs use the existing operation-ID grammar.
An exact repeated enrollment ID resumes only its own journaled context; changing arguments
is `operation_mismatch`. Enrollment normalizes its arguments once into a retained request.

`location inspect` performs no creation, nonce installation, SQLite writable open or repair.
It returns `selected_by` (`explicit`, `environment`, `local_default`, `linked_default`),
canonical project, context kind, candidate root (null before binding exists), local candidate,
namespace/binding identifiers when known, authority state, failed predicates and safe next
commands. Errors return the partial inspection in `data`; a conflict is not an empty success.
With `--root`, inspect that exact root and its coordination pointers even when home or Git
is unavailable. Ordinary `store info/get/history/read/checkout/request/outcome/doctor --root`
remain available for eligible retained-source inspection. During preparation, `location
inspect`, migration status and root doctor report metadata/findings; they never expose the
partial destination as committed history.

Status/request/abort discovery uses the authenticated permanent route bucket under the
supplied root's **parent**, plus checked pointers/attributes when present. D119 §2.1.2
requires routes at both parents before the first reservation is accepted. The route derives
one exact-ID journal; status validates its authenticated reservation, binding and full
root/path observations under the complete lease set. Root absence does not erase the route.
No original request/cache file or default home selection is required. Invalid/foreign route,
reservation, nonce or mismatched pointers refuse without adopting a journal. Before a valid
reservation exists, routes alone produce `not_found`. Ordinary access/new attempts scan the
full journal slot set through the route, so an unindexed prior attempt cannot be bypassed
with a new ID; exact-ID discovery is not an adoption or active-index shortcut. D119 publishes
only a complete, synced reservation atomically into the final slot. Unpublished partial
drafts live in a separate non-authoritative pool, are never recovered as requests, and are
quarantined in place after creator proof is lost. Before any complete final was published,
status/request/abort/discard return `not_found` without effects; an explicitly supplied full
request may start at the unused ID or a new ID after the full absence/slot checks. Unknown
drafts are preserved and refused for reuse/deletion, not adopted; unexpected partial/final
bytes inside an ID slot still produce `coordination_corrupt`. A possibly published final
must be durably resolved before retry/new ID; failed confirmation produces
`migration_outcome_unknown`. These outcomes apply from both roots, including absent roots
with intact parents. Incomplete bytes cannot reconstruct the original request or nonce.

`migrate request` and ordinary read/request outputs require a previously absent file with
existing all-ancestor/no-symlink rules. Recovery extracts exact bytes from the authenticated
reservation; `request.json` is a checked derivative, never the sole discovery record.
Status reports observed phase, attempt nonce, request/reservation digests, original mapping,
inventories and exact terminal decision/abort. It returns `reserved` even without a root
pointer; apply with those identical bytes is the retry interface. Same-ID byte mismatch
refuses. Abort never rolls back a decision; reservation-only abort requires no source
baseline and works with an absent root if the parents and route chain remain valid.
Discard-prepared discovers through either root but targets only the destination. It removes
only proven owned residue of a durably aborted attempt and retains
routes, reservation, requests and tombstones. Only a fully validated and durably confirmed
decision is cutover authority; no route, MAC, directory name or status generation is one.

### 1.1 Operator migration and recovery

Run these commands from the selected linked project. `location inspect` is read-only;
a migration-required refusal includes `data.source.store_id`, `physical_identity`, the
candidate roots, authenticated binding key/digest when enrolled, failed predicates and
suggested commands. `authority_state` distinguishes absent, unmigrated, retained source,
committed destination and non-authoritative preparation. Explicit-root inspection bypasses
contextual Git/account lookup; it still validates any existing coordination chain.

```sh
xdesign store location inspect
xdesign store location enroll --operation-id enroll-project-1
# Stop all older/non-cooperating xdesign processes and keep them stopped.
xdesign store migrate prepare migrate-project-1 --output migration-request.json --older-clients-stopped
xdesign store migrate apply --request migration-request.json
xdesign store location inspect
xdesign store migrate status migrate-project-1 --root .xdesign
```

`prepare` is a public request generator. Under the common lease it validates the enrolled
binding, canonical source/destination, physical source identity, complete schema-1 history
and absence of an occupied destination. It writes a previously absent external file with
all exact request fields; the operator supplies an operation ID and explicit quiescence
assertion, never physical IDs or a binding digest. It does not reserve an attempt, create
store history, or cut over authority. The request contains no namespace secret key.
The `--older-clients-stopped` flag is required; it records an operator assertion, not a
process-fencing claim. `apply` repeats identity/history checks before accepting the request.

Keep the request bytes unchanged. If an apply response is lost, run status through either
original root, including an absent root whose original parent remains intact. Recover the
accepted bytes to a **new** external file and replay them:

```sh
xdesign store migrate status migrate-project-1 --root .xdesign
xdesign store migrate request migrate-project-1 --root .xdesign --output recovered-request.json
xdesign store migrate apply --request recovered-request.json
```

After reservation, repeating `prepare` with that ID also exports the authenticated original
bytes. Before reservation, retain the original prepared file; status/request correctly return
`not_found` and cannot recover an unaccepted request. A different byte sequence for an accepted
ID refuses `operation_mismatch`, including whitespace-only changes. Never replace an occupied
request output. Use the reported destination for explicit inspection after a successful
cutover, and leave the retained local source unchanged.

Before a decision, an operator can instead run `migrate abort ID --root SOURCE` followed by
`migrate discard-prepared ID --root SOURCE`. Discard removes only proven owned destination
residue after a durable abort; it preserves the source and recovery records. A committed
migration cannot be aborted. `migration_outcome_unknown` requires status/exact retry;
`history_conflict`, changed source, unsafe ancestry or unprovable identity requires inspection
and correction of the reported predicate, never deletion, merge, rekeying or automatic adoption.

`xdesign inspect` is a new read-only lifecycle inspection command: validates the selected
backend and returns registry, frozen plan/work/run, current/completed status and persistent
revision/history metadata, without executing `next`, checker evaluation or cache restoration.
Its root selection follows lifecycle precedence. Persistent `--revision` is pinned; legacy
has no store revision. Materials remain inspectable with store read or existing legacy
material endpoints; the inspection does not rewrite legacy files. Unqualified next/check
and all registration mutations still refuse a same-ID legacy/store collision.

## 2. Namespace and coordination records

Every JSON record is strict UTF-8, versioned, bounded by 16 MiB/64 levels, rejects duplicate
and unknown fields, and stores filesystem integer fields as decimal strings, never JSON
floating point. Paths in these internal records are absolute canonical native paths encoded
as UTF-8; non-UTF-8 contextual identity is unsupported. Timestamps/UIDs/IDs are observations,
not authorization. Native paths are never evaluated as shell strings. HMAC input is the
canonical binary grammar in §2.2, never serialized JSON; request equality is byte-for-byte.

```text
<account-home>/.xdesign/worktree-stores/v1/
  namespace.json                 # format, version, owner, epoch
  key                            # 32 random bytes, 0600
  enrollments/<id>/request.json   # immutable enrollment request
  enrollments/<id>/state.json     # durable phase and installed object observations
  objects/<nonce>.json            # authenticated object binding
  paths/<path-key>.json           # retained incarnations; never silently reset
  projects/<binding-key>/binding.json
  projects/<binding-key>/initialization.json  # root/DB identity and verified creation phase
  projects/<binding-key>/store/   # unchanged schema-1 store layout
  projects/<binding-key>/.xdesign-access-v1/...
<source-parent>/.xdesign-access-v1/
  format.json
  routes/<access-root-key>/<binding-key>.json # permanent authenticated routing
  roots/<path-key>.json
<destination-parent>/.xdesign-access-v1/
  format.json
  routes/<access-root-key>/<binding-key>.json # permanent authenticated routing
  roots/<path-key>.json
  reservation-drafts/<random>     # unaccepted scratch; never an ID or recovery authority
  migrations/<id>/reservation.json # immutable authenticated envelope incl. exact request bytes
  migrations/<id>/request.json    # derivative exact bytes, recoverable from reservation
  migrations/<id>/state.json
  migrations/<id>/baseline.json
  migrations/<id>/verification.json
  migrations/<id>/decision.json   # sole irrevocable authority when present
  migrations/<id>/abort.json      # terminal only if decision absent under leases
```

No key, binding or journal is stored inside the closed actual store root. Extended attributes
hold incarnation/access witnesses as specified in D118/D119. Root/namespace records and
attributes are operational coordination data, not Git products, item revisions or save
receipts. A partial or foreign namespace is not silently initialized. Publishing bindings,
indexes, tombstones or decisions is exclusively locked, atomic and durably flushed; an
incomplete set is reconciled from its journal, never inferred from matching IDs.

| Record | Exact required fields in version 1 |
| --- | --- |
| Namespace | `format="xdesign.worktree-namespace"`, `schema_version=1`, `owner_uid`, `epoch` (64 hex random) |
| Object evidence | `path`, `device`, `inode`, `volume_id`, `birth_seconds`, `birth_nanoseconds`, `nonce` (64 hex); filesystem capability observations are recorded in enrollment state |
| Binding payload | `schema_version=1`, `namespace_epoch`, `owner_uid`, `home`, `common`, `admin`, `worktree`, `project` (object evidence), `project_relative`, `key`, `store_root` |
| Authenticated binding/object record | `payload`, `hmac_sha256`; object payload has `schema_version`, `namespace_epoch`, `owner_uid`, `object`, `roles` (sorted subset of home/common/admin/worktree/project) |
| Path index | `schema_version`, `path`, `generation`, `binding_keys` (append-only ordered incarnations), `active_key` or null, `enrollment_id`; validate against immutable bindings, never trust index alone |
| Access namespace marker | `format="xdesign.store-access"`, `schema_version=1`, `owner_uid`, `parent_path`, `parent_identity`, `epoch` |
| Permanent route | Authenticated payload kind 7 in §2.2; identical bytes at both root-parent route buckets; neither current attempt nor authority |
| Reservation | Authenticated payload kind 8 in §2.2; exact request bytes plus one never-reused attempt nonce |
| Root pointer | `schema_version`, `generation`, `root_path`, `parent_identity`, `root_identity` (null before creation), `role` (source/destination), `migration_id`, `journal_path`, `journal_identity`, `request_sha256`, `attempt_nonce`, `reservation_sha256`, `attempts` (append-only prior ID/attempt-nonce/reservation-digest/request-digest/journal-path/journal-identity entries); after abort it still points to that terminal record |
| Access-reference attribute | `schema_version`, `pointer_path`, `access_epoch`, `migration_id`, `request_sha256`, `attempt_nonce`, `reservation_sha256`; validates external pointer plus physical root evidence |
| Initialization journal | `schema_version`, `binding_key`, `binding_sha256`, `initialization_id`, `phase` (reserved/creating/verified), `root_identity` or null, `database_identity` or null, `store_id` or null; created only under the binding and parent leases, verified before default use |
| Migration state | `schema_version`, `generation`, `migration_id`, `attempt_nonce`, `reservation_sha256`, `request_sha256`, `phase`, `source_identity`, `destination_identity` (null until created), `baseline_sha256` (null until available), `prepared_generation` (null until verified), `diagnostics` |
| Baseline | `schema_version`, `migration_id`, `attempt_nonce`, `reservation_sha256`, `request_sha256`, `inventory`, `source_identity` |
| Verification | `schema_version`, `migration_id`, `attempt_nonce`, `reservation_sha256`, `request_sha256`, `source_inventory`, `destination_inventory`, `destination_identity`, `owned_objects` |
| Decision | `schema_version`, `migration_id`, `attempt_nonce`, `reservation_sha256`, `request_sha256`, `binding_key`, `binding_sha256`, `source_identity`, `destination_identity`, `source_baseline_sha256`, `verification_sha256`, `authority_root`, `outcome="committed"` |
| Abort | `schema_version`, `migration_id`, `attempt_nonce`, `reservation_sha256`, `request_sha256`, `invalidated_generation`, `outcome="aborted"`, `residue_identity` or null |

Implementation refinement: baseline and verification nest their inventories instead of
flattening them into each journal record. Each inventory has `inventory_version=1`,
`verifier_version`, preserved `store_id`, `ddl_sha256`, `format_sha256`, `tables`
(name/columns/count/sha256), `blobs` (count/byte_count/inventory_digest), and `logical_sha256`.
The two complete inventory objects must compare equal before publishing a decision.
The reservation, baseline and verification together retain the physical mapping: exact
source/destination paths and root/DB device/inode/creation evidence. `owned_objects` records
each observed relative path, complete object identity and directory flag, including SQLite
sidecars when present; absent/replaced/unrecognized objects never acquire ownership merely
by matching a name. This representation is a material implementation-review refinement,
not a change to cutover authority or preserved store history. `parent_identity`/
`journal_identity` are the corresponding object evidence without a nonce where no incarnation
attribute is required, with null nonce; access epoch is a separate marker/reference field, not an overloaded nonce. The home object is observation-only: home
nonce is the namespace epoch, not an attribute written to the user's home directory.
An enrollment state records the exact request, per-object nonce installation and observed
physical evidence; phase is `reserved|installing|published`, monotonic by generation. Readers
of partially published bindings wait/refuse until published and verified. `store_id` is added
to the migration mapping or normal initialization's external status, never mutates an immutable
binding. No unbounded file is submitted as one material; large inventories may be partitioned
into digest-addressed bounded records with a manifest and streamed equality check.

The implementation also needs an initialization witness before a new global root is
usable. A root with no migration has no reservation to cite, so its access-reference
attribute uses a separate closed shape: `schema_version=1`, `initialization_path`,
`binding_key`, `binding_sha256`, and `initialization_id`. These fields identify the
external verified initialization journal and immutable binding. This is an implementation
refinement requiring review: overloading migration nonces/digests for initialization would
misrepresent their authority. The root and database must match the journal; missing or
mismatched attributes refuse access. A moved initialized root retains this attribute and
refuses `source_moved`. It never authorizes migration or request recovery.

### 2.1 Exact migration request

Required fields are exactly:

```json
{
  "schema_version": 1,
  "migration_id": "example-migration-1",
  "project_root": "/example/worktree/project",
  "binding_key": "<64 lowercase hex>",
  "binding_sha256": "<64 lowercase hex>",
  "source_root": "/example/worktree/project/.xdesign",
  "source_store_id": "<observed source store ID>",
  "source_identity": {
    "root_device": "<decimal>", "root_inode": "<decimal>",
    "database_device": "<decimal>", "database_inode": "<decimal>"
  },
  "destination_root": "/example/home/.xdesign/worktree-stores/v1/projects/<key>/store",
  "older_clients_stopped": true
}
```

Values in angle brackets are placeholders, not valid requests. `source_identity` deliberately
uses the existing four physical fields; migration adds observed creation evidence/baseline
under the exclusive lease. False/missing quiescence attestation refuses; true is an operator
assertion and cannot prove old processes stopped. The request binds an enrolled linked
project; source must be its canonical local candidate, destination its exact binding root.
A mismatched explicit environment never redirects it. Status discovery works without the
original request file, and `migrate request` recovers its exact bytes for retry.

### 2.2 Canonical authenticated bytes and key grammar

This section is normative for version 1; JSON key order/whitespace is not signed. Strict
parsing precedes encoding. Each required field appears exactly once; unknown/duplicate or
omitted fields fail, including nullable fields. No unlisted optional fields exist. Integer
JSON strings use canonical decimal (`0` or nonzero digit followed by digits; negative signed
values have one leading `-`, never `-0`); `schema_version` alone is JSON number `1`.
Hex is lowercase, even length, with no prefix. Hashes, epoch, nonce and MAC decode to 32 raw
bytes; all-zero random values are refused. Path text is exact valid UTF-8 of filesystem-
resolved names, without Unicode normalization, case folding or trailing slash except `/`.
No NUL, empty absolute path, `.`/`..` component or ambiguous filesystem resolution is valid.
`project_relative` is slash-separated resolved relative text, empty at the toplevel, never
an absolute path or `.`. Non-UTF-8 enrollment/migration context refuses `identity_unprovable`.
Explicit unbound APIs keep native byte paths; they use the distinct raw path key below,
without serializing unsupported context into binding JSON.

Binary integers/lengths are **big-endian**. `C(kind, fields)` is ASCII `XDWS`, byte `01`,
unsigned 8-bit kind, unsigned 16-bit field count, then fields in increasing numeric tag order.
A field is `tag:u16 || type:u8 || payload_length:u32 || payload`. Length is bytes, not Unicode
scalars; no padding, terminators, omitted lengths, variable integers or trailing bytes.
Nested `O` carries a complete `C`. A list `L` is `count:u32` then each
`type:u8 || length:u32 || payload`; count/length overflow refuses. Total encoded object and
JSON envelopes remain bounded by 16 MiB/64 nesting levels. All fields below use tags 1, 2,
... in the exact listed order; every kind is closed and has exactly that field count.

| Type byte | Symbol | Payload |
| --- | --- | --- |
| `00` | N | Zero bytes, explicit null; allowed only where stated |
| `01` | U | Exactly 8 bytes, unsigned u64 |
| `02` | S | Exactly 8 bytes, signed two's-complement i64 |
| `03` | T | Exact UTF-8 bytes |
| `04` | B | Raw bytes, fixed width where stated |
| `05` | O | Complete nested object |
| `06` | L | Typed list as above |
| `07` | Q | One byte `00` false or `01` true |
| `08` | X | Exact native Unix path/component bytes (not lossy UTF-8) |

| Kind | Ordered fields, with types and widths |
| --- | --- |
| 1 — object evidence | `path:T, device:U, inode:U, volume_id:B(17), birth_seconds:S, birth_nanoseconds:U, nonce:B(32) or N` |
| 2 — binding-key tuple | `schema_version:U, namespace_epoch:B(32), owner_uid:U, home:O(1), common:O(1), admin:O(1), worktree:O(1), project:O(1), project_relative:T` |
| 3 — binding payload | Kind-2 fields in the same order, then `key:B(32), store_root:T` |
| 4 — object registry payload | `schema_version:U, namespace_epoch:B(32), owner_uid:U, object:O(1), roles:L(T)` |
| 5 — project path-key tuple | `schema_version:U, namespace_epoch:B(32), owner_uid:U, path:T` (canonical selected project) |
| 6 — access-root path-key tuple | `schema_version:U, owner_uid:U, parent:X, child:X` (canonical parent and resolved single child component) |
| 7 — route payload | `schema_version:U, namespace_epoch:B(32), owner_uid:U, binding_key:B(32), binding_sha256:B(32), source_root:T, destination_root:T, source_parent:O(1), destination_parent:O(1), journal_base:T, namespace_path:T` |
| 8 — reservation payload | `schema_version:U, namespace_epoch:B(32), owner_uid:U, migration_id:T, attempt_nonce:B(32), binding_key:B(32), binding_sha256:B(32), request_bytes:B, request_sha256:B(32), route_sha256:B(32), journal_identity:O(1), source_root_identity:O(1), source_database_identity:O(1)` |
| 9 — incarnation MAC payload | `schema_version:U, namespace_epoch:B(32), owner_uid:U, object:O(1)` |

Kind 1 has `0 <= birth_nanoseconds < 10^9`. Null nonce is mandatory for observation-only
parent/journal/root/database evidence; bound common/admin/worktree/project objects require
a nonzero nonce, and home uses the epoch. Repeated bound objects carry byte-identical
observations and nonce. Roles are nonempty, unique and ASCII-byte-sorted from
`admin, common, home, project, worktree`; unsorted JSON roles are refused, not silently sorted.
JSON object member reordering is accepted and produces identical canonical bytes; reordering
binary fields, redundant integer zeros, forged lengths, unknown tags/types and duplicate
roles are rejected. Registry roles must match the role being verified, not just a valid MAC.
`request_bytes` is lowercase hex in reservation JSON and decodes to the original UTF-8
request (maximum 1 MiB for this new migration command); no JSON normalization is applied to
those embedded bytes. Parent observations in a route omit nonces even if that directory
also has an incarnation attribute. Validate attributes separately under the leases.

Define `D(label, value) = ASCII("xdesign.worktree.v1/") || ASCII(label) || 00 || value`.
Key files contain exactly 32 raw random bytes. An authenticated record is strict JSON
`{payload, hmac_sha256}` and uses HMAC-SHA256 with that key over:

| Object/purpose | Exact input |
| --- | --- |
| Binding MAC | `D("binding", C(3))` |
| Registry object MAC | `D("object", C(4))` |
| Route MAC | `D("route", C(7))` |
| Reservation MAC | `D("reservation", C(8))` |
| Incarnation attribute MAC | `D("incarnation", C(9))` |
| Binding key | SHA-256 of `D("binding-key", C(2))` |
| Project `paths/<path-key>` | SHA-256 of `D("project-path-key", C(5))` |
| Access `roots/<path-key>` / route bucket | SHA-256 of `D("access-root-key", C(6))` |

`binding_sha256`, `route_sha256` and `reservation_sha256` hash the **exact immutable record
file bytes**, including JSON formatting/MAC; both route copies must therefore be identical.
They are distinct from canonical MAC input and binding key. Validate a record's MAC, schema,
full tuple and context before trusting that digest. State/pointer/decision/abort JSON is not
independently HMAC-signed: its identity/digests must match the authenticated reservation, and
strict owner/physical/durable-decision checks apply. MACs establish consistency with the
local namespace key, not permission to mutate history or trust an arbitrary path.

The incarnation xattr is exactly 101 bytes:
`ASCII("XDIN") || 01 || namespace_epoch[32] || nonce[32] || mac[32]`.
Compute its MAC over kind 9 using the live kind-1 object including that nonce, canonical path,
owner and creation/volume observations. Attribute names are D118's platform-specific names.
Read/create via pinned descriptors, install create-only, flush, reread and compare; no JSON,
hex text, NUL padding, role list or native struct layout is stored in the xattr. Existing
unknown/wrong-size/wrong-epoch/MAC attributes refuse. Home has no xattr. Access-reference
xattrs remain strict UTF-8 JSON of the exact table fields (not this incarnation format),
checked against the route/pointer/reservation rather than signed as incarnation evidence.

These hashes/MACs do not authenticate against a hostile owner controlling the namespace key.
Copied valid bytes at a different live path/inode/birth/volume, foreign epoch/key or wrong
role fail context comparison even when their original MAC verifies. Replaying every key,
attribute and physical observation is outside this cooperative-owner trust boundary.

The versioned [known-answer vector file](assets/worktree-store-vectors.json) supplies full
input objects, encoded hex, SHA-256, five domain-separated HMACs and the exact incarnation
xattr, using key bytes `00 01 ... 1f`. It includes UTF-8 multibyte paths, nested objects,
large file IDs, signed birth seconds, null nonce and Linux volume words; fixtures are
synthetic, not live identity evidence. Selected independently reproducible answers:

| Input | Expected digest |
| --- | --- |
| `binding-key` | `0b64ac1d5e116fc496adc169bd8edd7186408d7c908d00bdfcfadde794394f03` |
| `project-path-key` | `0bb9f7a3e8bc2cdb2955f8db674c1847cfd9599f5e135106ce62605e6c4cf176` |
| `access-root-key` | `f7d571bde2048ae6b9be71d0b97b2c20a1da54cba372efce106aa3319328d9e7` |
| `incarnation` | `93817d5c6d124ad0ac3f36cc6d2346073913a24439865f03bdf019074298832d` |

### 2.3 Platform observation profile

All identity reads are descriptor-based and repeated under the complete lease; do not sign
raw native structs or use ctime/mtime as a substitute for birth time. `device` is the native
`fstat.st_dev` bit pattern zero-extended to u64; `inode` is `fstat.st_ino` as u64, checked
against the platform's second observation below. UIDs are effective UID as u64. Unsupported
width/negative invalid values or inconsistent descriptor/path observations refuse.

| Platform | Exact creation, volume and file sources |
| --- | --- |
| macOS | `fstat` 64-bit `st_ino`, `st_birthtimespec.tv_sec/tv_nsec`; object-descriptor `fgetattrlist` with `ATTR_CMN_RETURNED_ATTRS`, `ATTR_CMN_FILEID` and `ATTR_CMN_CRTIME`, checking support bits, file ID and birth equality. Separately pin the volume-root descriptor from `fstatfs.f_mntonname`, verify same device/fsid and mount membership, and request `ATTR_CMN_RETURNED_ATTRS` plus `ATTR_VOL_INFO | ATTR_VOL_UUID` there; do not combine FILEID with volume attributes. `volume_id = 01 ||` the 16 UUID bytes in returned byte order (not host integer layout). All-zero UUID or unsupported attributes refuse. |
| Linux | `statx(fd, "", AT_EMPTY_PATH | AT_SYMLINK_NOFOLLOW, STATX_BASIC_STATS | STATX_BTIME | STATX_MNT_ID, ...)`; require returned INO/BTIME/MNT_ID and UID/mode support, compare `stx_ino` and device major/minor with `fstat`, use `stx_btime.tv_sec/tv_nsec`. `fstatfs(fd)` gives `volume_id = 02 || u64(f_type) || u32(f_fsid.__val[0]) || u32(f_fsid.__val[1])`, integer words encoded big-endian as unsigned bit patterns. An all-zero fsid refuses. Mount ID is a capability observation, not a persisted incarnation field. |

macOS uses the local mounted-volume flags and volume UUID, resolves the actual component
names, and refuses duplicate volume mount aliases. Linux cross-checks `stx_mnt_id` with
`/proc/self/mountinfo`, mount root `/`, unique device/fsid mount mapping and local filesystem
type; bind, overlay, network, FUSE or unreadable/ambiguous mount topology refuses enrollment.
A filesystem type without independently established local birth/xattr/lock/flush behavior is
not admitted. Linux fsid is a filesystem-supplied discriminator, **not a cryptographic UUID**;
changed device/fsid after remount/reboot fails continuity and requires inspection, never
automatic rebinding. Zero/unsupported birth data, truncation or failed xattr/directory flush
is `identity_unprovable` (or `coordination_unavailable` for locking/flush capability).
The Linux implementation uses O_PATH descriptors for identity observation so closing an
observation descriptor does not release an open SQLite connection's POSIX locks. It admits
ext4, XFS and Btrfs only with the stated topology and capabilities; access and default POSIX
ACLs outside the admitted profile both refuse. No-replace publication uses `renameat2` with
`RENAME_NOREPLACE`, attributes use create/replace-only descriptor operations with flush and
readback, directory leases use nonblocking `flock`, and file/directory durability uses `fsync`.
Missing kernel/filesystem capabilities refuse rather than switching to weaker primitives.
Host capability tests remain required before claiming support on either platform. This
specifies observations; it does not claim Linux tests or power-loss certification occurred.

API bases: [Apple attribute definitions](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/attr.h)
, [stat fields](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/stat.h)
and [attribute-call constraints](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2);
[Linux statx](https://man7.org/linux/man-pages/man2/statx.2.html) and
[fstatfs](https://man7.org/linux/man-pages/man2/statfs.2.html). The versioned byte profile above,
rather than future header layout changes, governs encoding. Old unbound explicit roots need
only their existing physical checks and the common directory lease; lack of birth/UUID/xattr
capability does not alone disable their read/write compatibility. Such roots cannot enroll
or migrate until the stronger capability contract is satisfied.

## 3. Errors and compatibility

Store commands use the existing schema-1 stdout success/stderr error envelopes and exits
0/2/3/4/5/6/7. New command strings are `location.inspect`, `location.enroll`, `migrate.prepare`, `migrate.apply`,
`migrate.status`, `migrate.request`, `migrate.abort`, `migrate.discard-prepared`. Migration
responses identify `migration_id`, not a synthetic save `operation_id`. Status exit 0 means
its observation is valid, including a nonterminal or aborted phase; it never implies migration
success. A valid decision is required for apply exit 0 with `outcome=committed`; replay of an
aborted apply returns conflict with the terminal abort in `data`. Diagnostic data includes
candidate roots, failed predicate, observed phase, request identity and safe next commands;
no “delete both roots and retry” recovery advice is allowed.

| Diagnostic code | Exit | Required action |
| --- | --- | --- |
| `invalid_input` | 2 | Correct empty paths, malformed request/flags, false quiescence assertion or invalid context arguments. |
| `not_found` (including `binding_missing`) | 3 | Inspect the named missing root/request/item; absence is not initialization permission. |
| `migration_required`, `history_conflict`, `backend_conflict`, `backend_ambiguous` | 4 | Inspect both candidates/backends, then explicitly migrate if eligible; never choose by ID/head. |
| `migration_busy` | 4 | Bounded retry/status after lease owner finishes; do not remove locks. |
| `source_retired`, `source_changed`, `source_moved`, `migration_committed`, `migration_aborted`, `operation_mismatch` | 4 | Inspect durable decision/abort and roots; retry only exact request at authority where allowed. |
| `home_unavailable`, `identity_unprovable`, `binding_collision`, `coordination_corrupt`, `unsafe_path`, `integrity`, `unsupported_format` | 5 | Preserve bytes; inspect failed identity/ownership/format; repair external prerequisites only with proof. |
| `coordination_unavailable`, `io`, `database` | 6 | Known pre-mutation failure; fix supported filesystem/I/O capability and inspect status before retry. |
| `migration_outcome_unknown` | 7 | Query same migration ID or recover/exactly replay its request; never start a new migration. |

Existing public `RsStore` signatures and `RsStoreError` variants remain. To preserve exhaustive
Rust client matches, private `LocationDiagnostic` converts to `Integrity`/`UnsafePath` for
unsafe/conflict states and `Io { operation_id: None, ... }` for busy/unavailable access.
Messages begin with a stable ASCII token `location_reason=<code>;` followed by details;
adapters share a parser for that prefix to recover the detailed codes above. They do not
parse free-form English. Direct callers can inspect the token, or retain existing variant
handling. Save `UnknownOutcome { operation_id }` still means an actual store operation;
it is never reused for a migration ID. The CLI's private migration error type carries its
own ID/state. This adds no public unchecked import/rebind API.

Safety-tightened behavior can reject previously tolerated ambiguity, unsafe parents,
unprovable filesystems, or concurrent old-client usage. Initial explicit reads use no home
lookup and create no guard files. Existing stores and exact requests remain schema 1; request
numeric/path/size checks for *new* saves are not retroactively applied to old operations.
Persistent material source and external output path policies stay unchanged. CLI clients
must handle the new error codes; old executables do not understand new commands or fences.

## 4. Web and lifecycle delegation

Web selects one absolute root at startup using D118 (a new eligible linked binding may be
enrolled as persistent session setup, after legacy classification and without initializing store history); `AppState` retains the selection and
root. Root-changing metadata is revalidated on access, not silently redirected. Health
returns `project_root`, `store_root`, initialization/selection state and any location
diagnostic. A genuine legacy-only session uses `store_root=null`, `selection_state=legacy_only` and
requires no unrelated home/binding; it must resolve before adding persistent work.
Unsafe/unresolvable required persistent startup selection fails before binding a listener; conflicts
with inspectable candidates can serve a read-only diagnostic state. All actual operations
still guard/revalidate their root and source relationship.

Every `Lifecycle::run` invocation receives `store_root: Option<&Path>`; `Some` is required
for persistent work and also retains any explicit root context during legacy classification;
`ProcessLifecycle` sets
`Command.env("XDESIGN_STORE_ROOT", absolute_root)` and `current_dir(project_root)` before
executing the same binary. For a validated legacy-only session with no explicit option/environment root and no known
persistent candidate, `None` removes that environment variable and the
child reruns strict classification before continuation; newly discovered candidates can
refuse the child. No persistent creation runs with `None`. It never relies on inherited
selection or process-global `set_var`.
This fixes the current explicit-Web-root/child-root mismatch. Init, next, check and all
auxiliary work-query paths use the same parameter, including test doubles. Parent leases
are not held while waiting for a child; the child owns its command lease and reruns all
conflict checks. An error between preflight and child execution causes refusal, not fallback.

Web lists both same-ID rows with `source=legacy|store`, `conflict=true` and disabled actions.
`GET /api/wishes/{id}?backend=legacy|store` and the material endpoint with the same selector
provide read-only inspection of both histories; omit the selector for ordinary unique IDs.
Selectors are rejected on mutation and `/work` routes. Ambiguous unqualified detail/work
returns HTTP 409 with both inspection links. No implicit backend preference remains.
Unsupported store format/integrity is not converted into “no store” or an empty wish list.
Location conflict/busy/retired errors map to 409; missing entities to 404, invalid input to
400, unsafe/corrupt/home/capability/I/O errors to 503 with the detailed code. Unknown migration
outcome is a CLI concern; Web performs no migration writes. Lifecycle command errors retain
exit 2, checked blocked work retains exit 1, and success/completed retains exit 0. stderr
includes the stable reason token so Web can map it without altering plan/run JSON schemas.
