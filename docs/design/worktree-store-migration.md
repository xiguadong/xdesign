# Worktree store migration protocol

> Status: designed-only extension of schema-1 storage; no migration execution or acceptance
> claim. Allocation: `XDesignStorage::PlannedWorktreeStoreMigration` and
> `PlannedWorktreeStoreAccess`; selection and trust boundary: [D116](storage.md#d116).

<a id="d117"></a>

## 1. Authority and implementation boundary

This design preserves the accepted [R20 requirements](../requirements/requirements.md#r20)
and their allocations. [I21](../interfaces/worktree-store.md#i21) owns the public command
and record schemas; [D118](../verification/worktree-store.md#d118) owns executable test
procedures. D86's current behavior is not retroactively relabelled as implemented migration.

The existing `src/storage/db.rs` creates a closed version-1 root, checks `format.json`,
SQLite application/user versions and mode, and compares the root/database device/inode
fields in `store_identity` with live objects. `RsStore` retains a SQLite connection and
its original `RsIdentity`. `verify_live` repeats checks, but does not currently consult a
migration fence. A same-filesystem directory rename can preserve these inode checks;
a copy or database replacement does not. Therefore path selection, matching store IDs,
`cp`, `rename`, editing `format.json`, or ordinary `save` cannot implement this migration.
The new protocol must validate both existing physical checks and the separate binding.
For an old never-bound root, there is no historical path field with which to detect a
same-inode rename before its first upgraded access. This is an explicit observation limit,
not a supported restoration path. Such a move supplies no migration provenance and cannot
make a global binding authoritative. Existing explicit-root inspection retains physical
validation; it cannot certify the unknown prior location.

The source is exactly the canonical selected project's existing `.xdesign`, valid under
the current version-1 rules. Destination is the dedicated D116 binding's absent `store`
child. General backup/restore, moving broken stores, schema conversion, merge, legacy
cache import and relocating arbitrary explicit stores are excluded. Explicit selection
continues to choose an existing root without migrating it.

### 1.1 Preconditions and complete baseline

Before acquiring the migration lease, the operator must stop all older binaries, scripts
using direct SQLite, and other non-cooperating writers; attest that in the migration request.
There is no reliable PID census or OS lock that retroactively forces them to cooperate.
Quiescence must continue while the retained source exists. A normal upgraded lifecycle
command holds a lease through checker execution, publication and commit; migration reports
busy instead of killing it. Existing per-wish lock files remain a separate condition:
active, stale, malformed or unreadable locks all refuse migration. Custom checker children
that attempt reentrant access to the same guarded store receive busy; the parent lease is
not inherited as a child authorization. Resolve a stale lock only
using the existing operator procedure after establishing its owner has stopped. The migration
never deletes locks or reconciles pending saves implicitly.

Reject any unsupported/foreign/unsafe source, dirty doctor finding, pending operation,
nonempty `pending_blobs`, staging residue, orphan blob or inconsistent schema. Run the
supported original-root `outcome`, `request` and exact `save --request` recovery for pending
work before starting, or abort an unfinished preparation to permit that recovery. After
reconciliation take a new complete baseline; a prior hash, enumeration or prepared target
cannot be reused. A target with any unrelated content is occupied even if its store ID,
latest wish IDs or heads match. No target adoption based on resemblance is permitted.

Baseline enumeration runs under the exclusive access lease and one consistent SQLite read
transaction, with live root checks before and after. It validates integrity/foreign keys,
actual table/trigger/index definitions against the supported schema, strict stored JSON,
request digests and request/receipt links, revision contiguity, kinds and heads. Unknown
schema objects are refused for this privileged operation even if ordinary open tolerated
them. Enumerate all rows, not only wish heads or rows reachable from the latest manifest:

| Content | Preserved and verified value |
| --- | --- |
| `items` | Exact ID, kind, head; head zero without committed history is refused as unresolved state. |
| `revisions` | Every item/revision, exact stored `metadata_json` text, producing operation; metadata decoded with original compatibility validation. |
| `materials` | Every revision's complete URI/role/digest/length rows, including old artifacts, shared bytes, reports, checker bundles and exact run/registry/plan/work bytes. |
| `blobs` | Every row and file; hash and length of all files, including blobs not referenced by the current head. Orphan files without rows refuse. |
| `operations` | Every operation ID, exact BLOB `request_bytes`, digest, item, expected revision, terminal outcome/revision and exact stored `result_json` text, including conflicts without revisions. |
| `pending_blobs`, staging and locks | Must be empty/clean at the baseline and cutover; no transient authority copied. |
| Format and identity | Source format/store ID, physical identities, root/DB creation evidence and source binding; destination physical identities recorded separately. |

Compute a versioned SHA-256 inventory from binary length-prefixed row streams: row count,
field count, then each field's SQLite storage-class tag, length and exact bytes (signed i64
big-endian integers, UTF-8 text, raw BLOB, tagged null), ordered by primary key using binary
collation. Each table digest binds its length-prefixed table/column names and row-stream
digest. The aggregate binds its version tag, exact format bytes, supported DDL digest and
ordered table names/digests. Blob digest/length rows participate after every physical blob
has independently passed hash and length verification. Retain row counts, column names,
per-table digests, blob count/byte count and verifier version alongside the aggregate;
physical source/destination observations are retained separately because they intentionally
differ. I21 records the nested baseline/verification representation. This implementation
refinement is a migration fingerprint, not a new item/revision snapshot.
WAL/SHM bytes and page layout are not logical history; include their validation findings,
not their changing byte layout, in the baseline. The source may checkpoint without changing
logical history. Re-enumerate fully immediately before decision and on every resumed
preparation; no timestamp-only or `data_version` shortcut is sufficient.

### 1.2 Verified destination construction

A private `MigrationLease` capability is the only way to call the constructor. Construct
at the final destination path, behind its durable preparing fence; do not create a normally
openable temporary store and rename it. Use exclusive creation, owner-only directories,
the attempt-owned empty root allocated after fencing, a new `store.db`, the pinned
schema/PRAGMAs and **exact validated source `format.json` bytes**, including valid
noncanonical whitespace/member order. Only the new root/database identity rows differ;
format bytes and their digest remain in logical equality, never a regenerated marker.
Insert the new root/database identity **once**, at creation, rather than updating or disabling
the existing immutable identity trigger. Source physical identity remains unchanged.

Publish every blob via bounded streaming, independent destination files (no hard links or
shared mutable database), checksum comparison, `0400`, flush and directory flush. Import in
one database transaction after blob publication: insert exact terminal operations, blob rows,
items at head zero, then each item's ascending revisions and manifest rows, advancing that
item's head once per imported revision. This ordering satisfies the existing triggers;
there are no calls to public `save`, new operation IDs, synthetic receipts or extra revisions.
No stored request is passed through today's *new reservation* limits; original schema-1
validation remains the compatibility test. Do not canonicalize retained request or JSON text.

Commit with existing FULL/fullfsync durability, close/checkpoint the destination, flush its
files and directories, then reopen it through the private migration capability. Independently
enumerate and verify all logical rows, projections and bytes. The destination's inventory
must equal the source's excluding the deliberately rebound physical fields. Persist the full
source/destination physical mapping, inventories and verifier version in the migration
record. Preserving `store_id` expresses common logical origin; it never alone proves authority.
Only the durable migration decision does. Validate both flows through their existing strict
registry/plan/run loaders without materializing or overwriting workspace files.

A failure leaves a fenced partial target. Exact retry can retain verified published blobs
but must start a fresh database import from a newly checked baseline; no general partial
SQLite repair. Removing a partial database is allowed only under the private lease when its
recorded attempt owns the exact object, before decision. Unrecognized or replaced objects
are preserved and cause an integrity error. Abort may retain the entire partial directory
for inspection; no auto-delete of source or unrelated bytes is permitted.

## 2. Durable coordination and held handles

### 2.1 Discoverability without foreign root entries

Each canonical store parent can contain a reserved sibling `.xdesign-access-v1/` directory,
with a strict owner/version marker, `roots/<path-key>.json` tombstones and migration records.
It is outside the closed store root; `.DS_Store` remains the only inert extra root entry.
For source `<project>/.xdesign`, this sibling contains coordination only, never operational
history. Destination coordination is beside `v1/projects/<key>/store` inside the global
namespace. Unrelated existing content at the reserved sibling is refused and never adopted.
Roots nested inside another initialized store are refused to avoid foreign entries there.

The access-root path key and authenticated route bytes are defined by
[I21 §2.2](../interfaces/worktree-store.md#i21). Parent aliases use resolved filesystem
names; a final root symlink is refused. Ordinary explicit access to an unbound root needs
no account-home lookup, birth-time enrollment or coordination writes. It retains native
`Path` support, including non-UTF-8 names, and the existing physical validation. Only
participation in linked binding/migration requires I21's stronger identity capabilities.

A root-directory access-reference attribute (`user.xdesign.access.v1` on Linux,
`org.xdesign.access.v1` on macOS) points to the external root pointer. It survives rename;
a missing/mismatched referenced record refuses. A removed/recreated root is still found
through its parent tombstone and permanent route. Controlled coordination bytes must remain
outside closed stores. Bind/overlay/multiple-mount aliases without a unique parent are
unsupported. Creation and external output retain all-ancestor no-symlink rules.

#### 2.1.1 One discovery and acquisition algorithm

All entry points use these steps, including ordinary APIs, held handles, enrollment,
migration/status/abort/discard, baseline checks, pure legacy commands and Web preflight.
A probe is never an authority check or a lease retained into acquisition.

1. **Probe with no lock held.** Pin close-on-exec descriptors for the requested existing
   parent and, when supplied, selected project. Read bounded, no-follow snapshots of root
   pointers, access attributes, permanent routes, adjacent recognized bindings, initialization
   records, and exact-ID reservation/terminal records. An absent root is probed through its
   existing parent. Record both absence and present content digests/generations. Untrusted
   pointers provide candidates only: validate paths/owner/shape before opening descriptors;
   do not write, open SQLite, or follow an unchecked arbitrary journal. A disappearing or
   torn generation restarts; a stable malformed/foreign record refuses.
2. **Collect closure, release probes.** Each recognized binding/route contributes its
   namespace directory, canonical selected-project backend directory,
   and source/destination parents, even for an explicitly selected global root.
   A local source parent is also its project/backend directory. Peer records contribute
   their descriptors recursively until the set closes; supplied migration context adds all
   these before any acquisition. Enrollment includes every directory receiving a nonce as
   an object-directory lock. Pure legacy classification contributes the project/backend and
   existing discoverable persistent candidate parents, without requiring an unrelated home.
   If enrollment needs a missing namespace/binding parent, discovery contributes its
   nearest existing safe creation-parent descriptor and an exact absent-child predicate.
   Only enrollment may create that directory under this set, publish/flush its owned
   metadata phase, then release and restart discovery to include the new descriptor
   before any further use. It never acquires the new directory while retaining old locks.
   A normal access or recovery finding a formerly recorded parent missing refuses; it
   cannot recreate it. Ordinary explicit init still requires its final parent to exist.
   The first probe takes **no exclusive lock**; any diagnostic helper that used one must
   release it and every other probe lock here. Retain only descriptors and observations.
3. **Deduplicate, order, acquire.** Deduplicate by kernel `(device,inode)` after proving
   aliases refer to the same canonical directory; merge all roles without assigning priority.
   Conflicting canonical identities refuse. Sort unique descriptors by
   `(unsigned device, unsigned inode, canonical native path bytes)` and acquire
   exclusive nonblocking `flock` in that total order. This one order includes namespace, backend, nonce-object and parent descriptors alike.
   No acquisition is permitted while holding a descriptor outside this collected set.
   On a busy lock release **all** acquired locks in reverse order before retry; never wait
   holding a prefix, promote a lock, or acquire any newly discovered descriptor. Roles do
   not affect order: one directory may be a backend for one caller and a parent for another;
   a namespace-first hierarchy would allow inversion in that case.
4. **Revalidate under the complete set.** Rerun selection/classification; compare every
   pinned descriptor against the current path, all probed presences/absences, full binding,
   attributes, route, pointer/reservation/decision digests and generations. Reread the complete
   graph to prove no added namespace, backend or peer. A generation/pointer change with
   unchanged pinned identities restarts from step 1 after release. A replaced/moved pinned
   root/parent, changed alias or binding/incarnation mismatch refuses `identity_unprovable`
   (or `source_changed` for a post-decision source); never retarget a held handle. A missing
   peer root is legal only in its specified phase; missing/changed peer parent refuses.
5. **Operate and release.** Only now open SQLite, take snapshots, restore cache or create
   bytes. Revalidate identity/fences before output/commit and the complete source baseline
   at the final destination commit guard (§3.3). An unexpected graph change while the lease
   is held fails closed without an effect; do not release and resume a partly executed
   mutation. Owned pointer updates advance the lease's recorded generation explicitly.
   Drop leases on every return/error/unwind; a process exit releases kernel locks but never
   clears a durable fence. Nested guarded helpers borrow the same private owned lease.

Each invocation permits at most 8 acquisition/discovery attempts and a 250 ms monotonic
acquisition budget, whichever is reached first. Between attempts use bounded jitter
(1–16 ms), with no locks held; exhaustion returns `migration_busy`. Stable invalid data
returns its I21 reason immediately. The budget covers acquisition, not a checker, full
inventory or filesystem syscall latency. There is no starvation/fairness guarantee under
continuous contention; after competing operations stop and identities stabilize a fresh
call can acquire the fixed set. Tests use barriers and lock traces, not timing luck.
Unsupported directory locking returns `coordination_unavailable`; no deletable lock file
substitute is allowed. Ordinary absent-root init still requires an existing safe parent.

#### 2.1.2 Reservation discovery before the first root pointer

Before publishing **any** reservation or retained request bytes, install matching immutable
`routes/<access-root-key>/<binding-key>.json` records under both parents' access directories.
These permanent, namespace-HMAC-authenticated routes name the binding, source/destination
paths and parents, and the deterministic destination journal base. They do not name a
current attempt or grant authority. Validate their full I21 payload against the live
binding, namespace/key/owner and both paths; they survive root deletion and every abort.
A route key collision compares full payloads; foreign, contradictory or invalid-MAC
candidates return `coordination_corrupt`, never fall through to ordinary init. Normal access
scans its route bucket with bounded per-record reads even when no root pointer exists. Multiple
valid bindings for one path are diagnosed, not selected by recency. A prior incarnation may
be inspected by exact ID but cannot authorize a new attempt for the live project. For
ordinary access and a new apply ID, enumerate **all** ID slots in the authenticated journal
base (streaming bounded records), verify each reservation/abort/decision and digest the
complete slot set; reread that set under the leases. A prior nonterminal reservation blocks
a different ID before any write, even with no root pointer. A valid aborted attempt requires
its discard conditions before reuse of target space; a committed attempt retires the source.
Unexpected entries/partial final records are not ignored as empty history. The separate
non-authoritative draft pool below is not an ID slot. No mutable active index or
mtime shortcut replaces this scan. Discovery over budget returns busy without holding locks.
Exact-ID status/request can use direct lookup, but abort/apply still check the full attempt
set for contradictory decisions. IDs/pointers never let a new attempt overwrite an old one.

Publish each access-directory marker and route using exclusive creation, synced temporary
file, no-replace rename and directory sync, including each newly created ancestor's entry.
Reopen and verify **both** routes and their directory durability under the full lease set
before creating the journal slot. A crash after only one route is recoverable by matching
and completing the missing counterpart from the authenticated existing route, under both
parents' locks; no request has yet been accepted and status is `not_found`. Status never
creates the missing route. Exact apply can finish this setup. Unknown bytes are preserved.
A failed route sync returns known I/O failure before reservation, not cutover uncertainty.

Given an ID, either route computes exactly `<destination-parent>/.xdesign-access-v1/
migrations/<ID>/reservation.json`; no arbitrary journal option, glob over the home directory,
or original request file is required. The reservation is a single authenticated envelope
containing **all exact request bytes**, their digest, binding digest, fresh 32-byte attempt
nonce, both route digests, journal parent identity and source observation. It is published
before the derivative raw `request.json` and before any root pointer. The ID is unique
within this binding's journal; `(binding-key, migration-id, attempt-nonce)` identifies the
attempt. All state, pointer, abort and decision records carry that nonce and reservation
digest, and must match the envelope. Reused ID with changed request bytes refuses;
terminal IDs never receive a new nonce, even after discarded target residue.

**Atomic first publication, with no accepted partial request.** Under the full leases,
create/validate the safe empty `migrations/<ID>/` slot and sync it and its parent; observe
its identity for the existing reservation payload. Write the envelope to a freshly
exclusive-created random file in the separate owner-only `reservation-drafts/` directory
beside `migrations/`, on the same filesystem. Draft names are opaque random values, never
IDs or discoverable attempts. No `reservation.pending` is written in an ID slot. Before
publication, finish and strictly validate the whole envelope, sync the file, close its
writable descriptor and recheck its pinned identity/bytes. Atomically rename it **no-replace**
to that slot's `reservation.json`, then sync both affected directories and the slot's
ancestors and reread the final envelope. Unsupported atomic/no-replace/durability primitives
refuse; ordinary rename that can overwrite an existing final is forbidden. Existing final
bytes must be validated and compared, never replaced. No pointer, destination root, baseline
or source fence may be created until the final reservation is durably confirmed.

The first-publication boundary has only absence or a complete immutable final record on
supported filesystems; the file is never opened for writing after publication. A crash
during a draft write cannot leave an incomplete accepted attempt. Before any complete final
was published, status/request/abort/discard for that ID return `not_found` with no effect,
from either route, also with either or both roots absent and lost request/cache. No bytes
or nonce can be recovered from an incomplete draft, and none are claimed to have been
accepted. A surviving caller with the full request may start afresh at the same ID; after
losing it, an operator may explicitly supply a new request at that unused ID or a new ID.
Both require normal source/context validation, an eligible absent destination, and the full
slot scan. Absent source refuses apply with `identity_unprovable`; it does not prevent the
no-effect recovery commands. A nonce generated only in a draft was never an accepted attempt
nonce and may be abandoned; a durably accepted ID always retains its exact bytes and nonce.

Draft cleanup is not inferred from a filename, UID or valid-looking partial JSON. The live
creator may unlink its own unpublished partial only while retaining its exclusive-create
descriptor, matching path identity and full leases, then sync the draft directory. After a
crash loses that proof, leave the partial **quarantined in place**, without reading it as an
attempt, promoting it, renaming it or deleting it. Unknown foreign draft bytes are likewise
preserved and refused for reuse/cleanup; new creation chooses a fresh exclusive name. No
command adopts a draft, including a complete signed one, after restart. This pool never
authorizes history and does not block a clean ID slot/new ID; its retained bytes can consume
space and cause ordinary I/O refusal. An unsafe/replaced pool directory refuses creation.
Unexpected content in an ID slot (including an old `reservation.pending`), a malformed final,
or a foreign route/namespace remains `coordination_corrupt`, preserved and blocking. Such a
partial final cannot be produced by this publication sequence. A safe empty slot is reusable
without adopting history; neither it nor the draft pool requires a recursive intent record.
Their creation has no migration effect, which ends the first-publication regress.

If publication or sync was interrupted, resolve the final path under the full leases before
any new request/ID: confirm the exact complete final and all required directory flushes,
or conclusively confirm absence with stable directory identities, successful flushes and
no pointers/effects in the full slot scan. The former returns `reserved` with the original
bytes/nonce; the latter is the no-accepted-request case above. If neither can be established,
return `migration_outcome_unknown` (exit 7), preserving all bytes and forbidding new IDs.
Invalid final bytes are corruption, never absence. A fully durable accepted reservation is
not lost by deleting cache/drafts; unexplained removal of a referenced final fails closed.
Pure location inspection reports metadata without resolving publication. Request recovery
shares the resolving status path, then exports exact final-envelope bytes to a new file.

Request write/sync and first-pointer crashes are now downstream of a durable reservation.
From either supplied root, including an absent source or absent destination with parents
intact, status/request/replay/abort locate the same envelope. An absent source before decision
allows request recovery and reservation-only abort, but apply refuses `identity_unprovable`;
post-decision source absence is allowed. Complete route/namespace loss remains outside this
recoverability guarantee and fails closed when a surviving witness refers to it. Discovering
a reservation or authenticated router **never** establishes cutover: only a validated,
durably confirmed `decision.json` does. For ordinary access, any valid in-progress reservation
blocks the destination immediately; the source is writable only before its first fence.

### 2.2 Coverage and lifetime

| Entry class | Required lease interval and rule |
| --- | --- |
| Fresh `RsStore::init/open` | Before existence/probe/creation or SQLite open, through validated handle construction. Prepared destination refuses even if it happens to contain a complete database. Existing retained source can open for inspection; `init` on it refuses. |
| `save`, `save_bytes` | From guard discovery before retry lookup/reservation through blob publication, final live/fence check and SQLite commit, receipt return and cleanup. Retired-source save, including exact terminal retry, refuses with destination guidance; retry at destination returns the original receipt. |
| `outcome`, `pending_item_operations`, `list`, `get`, `history`, `read_material` | Entire SQLite snapshot and byte verification; retained-source inspection allowed, prepared target ordinary reads refused. |
| `checkout`, CLI `read/request` | Through verification and atomic external output publication. Guarded internal helpers avoid nested reacquisition. |
| `doctor`, CLI root doctor, `info`, request-byte helper | Same discovery; diagnostic-only inspection can report unsafe/partial roots, but cannot present partial data as authority or bypass a busy migration. |
| Lifecycle init/next/check | Root selection, legacy classification, cache restoration, per-wish lock, checker execution, save/commit and return share one command lease; retain existing stale-token/head guards. |
| Web | Each read or lifecycle child acquires its own complete lease; parent must release preflight lease before spawning and child revalidates, to avoid self-deadlock. |

A held `RsStore` is not a lease. Each public call reacquires from its pinned absolute root,
parent and physical identity, loads current records, and checks live identity under that
lease. A pre-migration handle is therefore fenced after cutover and cannot write through its
old SQLite connection. Deleting/recreating the source while retaining that handle fails its
identity/creation checks; a new open through the original path finds the tombstone. A root
renamed with its access attribute is refused for mutation (`source_moved`) and can be
inspected only if physical validation still passes. Use private `*_guarded` helpers and a
borrow of a private shared-owned lease internally; do not recursively lock the public API. Opening a SQLite
connection does not bypass the root guard, and no public raw connection accessor is added.

Cooperating clients must not remove coordination directories, tombstones, incarnation
attributes or their parent directories while references exist. Deleting the store root is
distinct from deleting this independent witness. Loss of a known record/pinned parent fails
closed. After erasure of *all* independent records and attributes, a fresh root-only API
call cannot distinguish a never-migrated root from recreated history: automatic fencing in
that case is impossible, and is explicitly outside this cooperative protocol. Do not claim
otherwise. Default selection still has its independent home binding/path registry. Removing
both that registry and local witnesses is an operator data-loss event with no supported
adoption or recovery in this slice.

## 3. Cutover and recovery

### 3.1 Ordered durable protocol

All record writes use strict bounded JSON, fresh same-directory temporary files, file sync,
atomic no-replace publication for immutable records (or atomic replace for generation
pointers), and parent-directory sync. The implementation must validate on its target hosts
that directory locks, atomic publication, extended-attribute flush and directory flush work;
errors are refusals. macOS uses the existing `F_FULLFSYNC` policy where applicable. Multiple
files cannot be atomically committed together; the protocol chooses one decision record and
makes every other pointer recoverable. No cross-filesystem atomic rename is assumed.

1. `inspect` validates context without changing it. Explicit enrollment establishes the
   destination binding when migration is needed. Migration accepts an exact request with a
   caller-supplied migration ID, full binding digest, expected source identity, destination
   and old-client quiescence attestation. IDs are separate from store save operation IDs.
2. Acquire the complete set using §2.1.1, validate context and require an absent eligible
   destination. Complete and flush both permanent routes, then publish the authenticated
   reservation using §2.1.2. Derive immutable exact `request.json` from that envelope and
   write the reserved state; missing derivative files can be reconstructed only from it. Write and
   flush the **destination** root pointer first, before creating any destination root bytes.
   Then create the final destination directory exclusively, record its physical identity
   and flush it. A crash between mkdir and identity recording can resume only the exact
   reservation at a safe empty owner-only directory with no foreign entries/attributes;
   there is no history to adopt. Any content or contradictory identity is preserved and
   refused. The private constructor subsequently uses this same attempt-owned directory.
   Crash here cannot expose a partial target. Source writes can still occur before its
   pointer is published; no source baseline is yet relied upon.
3. Publish and flush the source pointer, then both access-reference attributes and their
   directories. Mark `fenced`. From this point cooperative ordinary access returns busy
   even after the migrating process dies. A kernel lock's release does not clear the durable
   fence. Revalidate source identity; refuse existing item locks and pending/doctor findings.
4. Record the full baseline, construct and verify destination as §1.2, and record `prepared`.
   Revalidate source, all locks/pointers, binding, absent pending work and full inventory.
   Source disappearance/recreation, changed bytes or unexpected destination identity aborts
   progress with no decision; never treat disappearance during preparation as permission.
5. Publish one immutable `decision.json` at the reserved destination journal, containing
   request digest, both identity tuples, binding, inventory digests and destination authority.
   Flush it and its directory **before** reporting committed. Existence of a valid fully
   published decision is the only cutover fact; a generation/status pointer is not authority.
   File visible but flush failed is `migration_outcome_unknown`; keep both fences and resolve
   under locks. No caller may infer rollback from an interrupted response.
6. Update derived status pointers to `committed`, release leases and return the decision.
   If these updates fail, the decision still wins. Every decision resolver, including ordinary access, confirms the
   decision file and directory flush under the leases before permitting mutation; flush
   failure preserves `migration_outcome_unknown`. Ordinary clients discovering an old
   `prepared` pointer read the decision; they do not resume source writes. Source history is
   retained. No post-decision rollback or delete command exists.

### 3.2 Recovery state table

| Durable observation under leases | Default/ordinary access | Exact retry/status | Abort |
| --- | --- | --- | --- |
| No accepted reservation (routes/empty slot/unpublished quarantined drafts allowed) | Existing selection rules; local candidate blocks default new authority | Status/request `not_found`; full explicit request may start afresh; no reconstruction of incomplete bytes | No effect; abort/discard report not found |
| Valid reservation, with no root pointers or destination pointer only | Target unavailable; source explicit operations allowed until fenced | Status `reserved`; retry verifies identities and publishes missing fences before taking a baseline | Persist `aborted`, retain destination tombstone/residue; no source baseline exists |
| Source pointer present, missing attribute or `fenced` | Both sides busy; doctor/status diagnostic only | Finish metadata only if exact objects/record identities match; fully enumerate afresh | Verify no decision, persist abort and invalidation before releasing source |
| Copying/partial target | Both sides busy; partial target never an authority | Verify request; rebuild owned partial database and re-enumerate source; status `preparing` | Abort as above; preserve partial bytes for inspection |
| Verified target, no decision | Both sides busy | Status `prepared`; full source and target revalidation required before decision | Abort allowed only under all leases after proving decision absent |
| Decision publication outcome unknown | Both sides refuse ordinary mutation | Resolve decision/durability **before source-presence refusal**: confirmed decision returns its original committed result even without source; unresolved remains exit 7; conclusive absence alone permits fresh preparation or absent-source `identity_unprovable` | Only after absence is conclusively observed under all leases and durability errors resolved |
| Valid decision, stale derived pointers | Destination authoritative; source read-only after identity checks | Return original decision, repair only derived coordination pointers; never replay import or add receipts | `migration_committed` refusal |
| Decision with missing source | Destination allowed after binding/target checks | Original decision remains inspectable; absence is recorded as observation only | Refused |
| Decision with changed/recreated source | Default mutation blocked; explicit eligible source and destination read-only inspection | `source_changed`; original decision retained, never merged/replaced | Refused; reconciliation/merge unsupported |
| Missing/invalid journal, pointer, key, target or mismatched identity | Fail closed; diagnostic-only inspection | `coordination_corrupt`/`identity_unprovable`; no automatic reconstruction | No destructive cleanup; unavailable until evidence can be restored externally |
| Durable aborted request | Source access can resume; target permanently non-authoritative | Same request returns `aborted`; changed bytes same ID is mismatch | Idempotently returns same abort |

Abort first durably records `aborted` with invalidated baseline/prepared-target generation,
then updates source pointer to permit source access. A crash between them is reconciled from
the authoritative abort record. Never unlink a source pointer to “unlock” it. A new migration
ID after abort may reserve a new destination attempt only after explicit disposal of owned
partial state via `migrate discard-prepared`; this command validates aborted state, full object
ownership and identities, never touches source, and refuses unknown bytes. Tombstones and
old requests/outcomes remain. Each old ID always replays its own terminal result through the permanent route and exact ID; the pointer's retained attempts list is a redundant checked index. Seeing both a decision and abort for one ID is coordination corruption; it
cannot be a legitimate terminal state.

For migration request recovery, a durably confirmed reservation suffices even when decision
flush is uncertain; exporting the request makes no statement about cutover.

Specifically, an exact apply/status with `decision_uncertain` first validates the reservation
and possible decision under both routes' complete lease set, even with lost request/cache
and absent source. A valid decision whose file and directory durability can be confirmed
returns the same committed result/mapping, with no import or new receipt. An unreadable
decision path, indeterminate publication, or failed confirmation stays
`migration_outcome_unknown`; source absence does not downgrade that uncertainty. Only a
conclusive absent-decision observation with successful directory synchronization allows
apply to test preparation preconditions and return `identity_unprovable` for absent source.
With source present it resumes full preparation. Known malformed/contradictory records still
refuse `coordination_corrupt`; absent/replaced destination after confirmed decision still
refuses identity validation. No uncertainty permits abort/discard/new-ID bypass.

Request equality means exact bytes, not only digest. Unknown field, duplicate key, reused ID
with changed bytes or mismatch with selected binding is rejected. Status/inspect need no
original cache files; request bytes, decisions, aborts and mapping are durable external
records authenticated by the reservation/route chain. A retry lost after decision returns the same decision with no data write. An old
client's later source write cannot be prevented; full baseline comparison before **every
default mutation** detects it while stopped. Explicit destination selection can inspect
conflict; it must also honor an attached decision's source-baseline guard for writes, so it
cannot be used to bypass a known split history. A restored stale whole coordination registry
is not authenticated backup recovery; this slice offers no such facility.

### 3.3 Source validation at the destination commit boundary

Every new or resumed destination mutation, including explicit destination access, validates
the retained source twice under the common lease: before work and again **immediately before
SQLite COMMIT**, after staging and just before the final live/fence check. The latter fully
enumerates source logical history, exact format bytes and physical mapping against the
cutover baseline; it checks current absence again rather than caching it. No timestamp or
`data_version` shortcut suffices. Observed changes roll back the destination transaction and
return `source_changed`, retaining existing operation uncertainty semantics if a store
commit was already attempted. An exact terminal receipt replay does not commit anew but
still validates the source before returning; an unfinished operation repeats both checks.

The final check is the detection point, **not** atomic fencing of an old writer. A prohibited
noncooperating writer can commit after this observation and before/after destination COMMIT.
This cannot be prevented by these locks or SQLite transactions in different databases;
keep old writers stopped. A later destination mutation must detect that changed baseline
before any further commit; never claim a concurrently raced acknowledged update was prevented.
WTSTORE-07 uses separate barriers before the last source check and after it to distinguish
observed refusal from this explicitly unsupported race.

## 4. Lifecycle, Web and compatibility

The current Web `ProcessLifecycle::run(args, cwd)` passes only arguments and cwd: an
explicit `--store-root` affects Web reads but the child inherits a different
`XDESIGN_STORE_ROOT` or defaults locally. The signature accepts an optional root parameter (I21);
production `Command` sets that **absolute** effective root in the child's environment on
all persistent init/next/check paths. A validated legacy-only session without an explicit override or known persistent candidate
passes no root and clears that environment variable; its child reclassifies before continuing without enrollment.
An explicit root is retained even for a legacy child so a newly conflicting item at that
root is checked under the child lease. Do not mutate the server process's environment per request.
The child's default-context safety policy must not be lost by turning the path into an
override: registered access records enforce the same binding/source baseline at the root.
Health returns the chosen root and selection status; it does not imply an initialized store.
`open_store` must stop translating unsupported/corrupt roots to an empty list.

Centralize backend classification in the shared internal `project_backend` source module
**before default resolution/enrollment**, cache restoration, reads or writes; both library Web and binary stage adapters compile this same pure
module, while existing flow validators retain their responsibilities. Validate the strict legacy
registry and its plan/work/run linkage; `wish_model_uri` in `mbse/wishes/` is a legacy
candidate, not by itself sufficient validation. A cache registration is persistent only
when its identities and exact operational materials can be proven against a selected store
revision. Old cache from a different store or unmatched `cache/wishes/` registry is
`backend_ambiguous`, never automatically deleted or treated as legacy. Missing persistent
cache is regenerable from the store. Genuine legacy history stays legacy, including
active/completed behavior and Git traces. First strictly inspect the local registry and
known persistent candidates under the project/backend discovery algorithm: explicit root
if supplied, local candidate, and global candidate named by existing project access routes
or persistent provenance. This read-only classification does not enroll or require an
unrelated missing/unsafe home. A genuine legacy-only ID can be read/continued with the
pure project lease even when no global binding/home is available. An existing locator or
persistent cache proves that a candidate must be checked: an unreadable/unsafe candidate
returns `backend_ambiguous`, never a claim of absence. No locator, no persistent cache and
no local/explicit candidate means no selected persistent history for this legacy ID; do
not search inaccessible unrelated homes. Reachable same-ID local/global candidates yield
`backend_conflict` before any mutation. The full under-lock rescan includes candidate
absence so a cooperating enrollment cannot race this decision. Only new or proven
persistent operations proceed to U114; explicit-root failure is not ignored. Read-only
`--backend legacy` can inspect validated legacy bytes despite an unresolved store, but
unqualified mutation cannot bypass a known unresolved candidate.

If a genuine legacy registration and persistent item share an ID, unqualified detail/work
and all lifecycle mutations refuse `backend_conflict`. Read-only CLI `inspect` and Web
`backend=legacy|store` select either history without materialization, writes or checkout
overrides. Listing surfaces both sources and a conflict marker rather than skipping a legacy
row. These inspection selectors never authorize mutation. For store material reads the
revision is pinned; legacy inspection preserves existing files exactly. If legacy identity
cannot be validated, report the obstacle rather than hiding the row. Take a project backend
lease (exclusive lock on the canonical selected-project directory, included in the same
physical-identity total order as namespace and parents, deduplicating identical descriptors) for classification through lifecycle commit to prevent supported legacy/persistent
writers from racing the check; legacy files stay on their original backend.

Core plan/work/registry/run schemas, state tokens, existing save requests and terminal
receipts are unchanged. Baseline copies include both flows' all visits, artifact revisions,
assessments, bundles and evidence. No migration judgment or new flow assessment is invented.
After migration unchanged-head tokens agree; checker replay and cache restoration use the
existing byte/digest and workspace-preservation rules. Completed next/check stay read-only.
Older clients can read schema-1 destinations explicitly but do not understand the fence;
using them concurrently or against retained source is prohibited operationally, not
technically prevented. The requirement gate and independent semantic design review remain
separate from this protocol's future verification and user acceptance.

## 5. Exact implementation file plan and review obligations

The binary's `stage_cli` cannot call library `pub(crate)` helpers. Therefore
`xdesign::storage::cli::resolve_store_root` and `with_project_store` are additive, documented
bridge functions ([root resolution](api/storage/resolve-store-root.md#u114),
[command access](api/storage/with-project-store.md#u115)). The latter lends a guarded
`&RsStore` to the whole lifecycle operation; stage helpers must use that handle instead of
reopening. Its private shared-owned lease lets existing methods borrow and revalidate without
recursively locking; U115 specifies the generic action-error wrapper and binary conversion. No lease or raw SQLite connection escapes the callback.

The following is the allowed implementation surface for this design. No Rust application
source is changed by this design stage. Adding another production path requires an explicit
source-plan correction before its implementation, not silent expansion. Existing public
Rust method signatures and enum variants remain; additive CLI bridge functions cross
the library/binary boundary. Adapter diagnostics extend behavior as
specified in I21/U81. Private capability types do not become a raw rebind public API.

The implementation adds `validate_project_write(&RsStore)` beside the two originally
planned CLI bridges. Cache restoration and checker scratch writes occur in the binary
before a public store save, so the binary must explicitly validate mutation authority
on its borrowed command handle before those writes. The helper returns the original
typed storage error and exposes no lease, connection or import capability. This is a
material adapter refinement for implementation review; read-only inspection does not call it.
The companion `read_completed_project` adapter validates a retained terminal head and
builds the existing completed response without materializing cache. This lets completed
next/check remain read-only at a retained source while cache-restoring active commands
and exact operation replays still require mutation authority.
`project_candidate_contains` repeats legacy collision checks across the complete known
candidate set under that same lease. It opens only parents already in the acquired closure
and returns an existence fact; it cannot acquire or export a new lease. This closes the
race between initial candidate discovery and legacy continuation when more than one root
is known to the project.

| Planned path | Exact responsibility and validation |
| --- | --- |
| `src/storage/location.rs` (new) | D116 selection, sanitized Git probe, account-home seam, nonce/creation evidence, namespace schema/HMAC, immutable binding and enrollment recovery. Map selection/migration/access responsibilities to the existing planned model elements. |
| `src/storage/location/linux.rs` (new) | Linux descriptor/statx/fstatfs and mount-topology checks, no-replace publication, and create/replace-only xattr operations; fail closed on missing capabilities. |
| `src/storage/access.rs` (new) | Directory leases, alias canonicalization, external root pointers, attributes, sorted peer discovery, held-handle checks and diagnostic classification. A private unforgeable lease parameter gates helpers. |
| `src/storage/migration.rs` (new) | Strict request/status/abort protocol, complete inventory and comparison, private verified constructor, single decision publication, pre-decision discard and exact replay. No general restore. |
| `src/storage/{mod,db,ops,blobs,cli}.rs` | Private modules/handle context plus the public CLI bridges described below; guarded init/open and every public operation/helper; constructor sharing DDL/trigger rules without relaxing ordinary open; fsync/attribute primitives; location/migrate command parsing and errors. `unicode.rs` unchanged. |
| `src/project_backend.rs` (new), `src/{lib,main}.rs` | Shared strict registry/backend classification compiled as an internal module in both library and binary; declarations only in lib/main. Pure byte/identity classification with no store writes, shared by stage and Web. |
| `src/core/{plan,restore}.rs` | Private byte-based entry points reuse the existing structural plan and historical run validators during migration. Existing public file-based loaders keep their signatures and live-file checks. This implementation refinement avoids materializing retained historical files into the workspace merely to validate them; it does not change flow rules or judgments. |
| `src/stage_cli/{mod,store,persistent}.rs` | Shared root resolution and command lease, classification/inspection, root-pinned existing lock/cache/save paths and typed diagnostic propagation. No flow engine/checker changes. |
| `web/server/{mod,api}.rs` | One selected root; child root propagation; strict selection/open errors, conflict listing and backend inspection; lifecycle trait/test doubles updated. |
| `web/assets/{app.js,styles.css,js/views/wishes.js,js/views/wish-detail.js,js/api.js}` | Display root/conflict diagnostics and read-only backend choices with disabled mutation controls. No automatic migration button. |
| `tools/records.py` | Stop hard-coding `.xdesign` as an implicit default; delegate default resolution to store CLI; preserve explicit `--root`. Binary lookup honors the existing build/cache convention. |
| `tests/{worktree_store,worktree_migration,worktree_access,worktree_web}.rs` (new) | Isolated Rust integration tests and exhaustive operation/alias/barrier matrix. |
| `tests/test_worktree_store.py` (new) | Real-binary CLI, two-flow, old-version fixtures, cache-loss and Web child end-to-end tests. |
| `tests/test_worktree_public_cli.py` (new) | Public CLI inspect/enroll/prepare/apply/status/request/exact-retry with a separately built old binary, isolated effective-account lookup, independent history/blob comparisons and unchanged-state assertions. |
| `Cargo.toml`, `Cargo.lock` | Only if tests expose a necessary dependency/API capability gap; prefer existing libc/sha2/SQLite and a small HMAC implementation with known-answer tests. No online/unpinned dependency update. |

Doc changes during implementation are limited to the owning D116/D117/D118/I21 pages,
store API pages, stage/Web contracts and `web/SPEC.md` as behavior becomes available;
immutable design evidence copies retain the designed baseline. Requirements and traceability
files stay byte-identical. The model's four planned allocations remain truthful even if code
is added later; turning a planned allocation into an implementation claim requires its own
review and evidence, not renaming frozen definitions.

Before implementation acceptance, reviewers must establish: every callable route owns a
lease through its effect; initial pointer ordering excludes partial adoption; direct handles
and aliases find the same fence; target import respects all triggers/history; state transitions
have a single decision/abort authority; incarnation capability probes fail closed; and each
WTSTORE assertion has actual evidence. Directory-lock/xattr durability on supported macOS
and Linux filesystems is a feasibility checkpoint to test early. If it fails, stop this
mechanism and report the contract gap instead of weakening the accepted requirements.
Process termination tests do not prove physical power-loss safety, hostile-owner security,
old-client fencing, full backup/restore or final engineering acceptance.
